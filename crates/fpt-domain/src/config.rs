use fpt_core::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env, fs, path::PathBuf, str::FromStr};

const DEFAULT_API_VERSION: &str = "v1.1";
const CONFIG_FILE_NAME: &str = "config.json";
const CONFIG_DIR_NAME: &str = "fpt-cli";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    Script,
    UserPassword,
    SessionToken,
}

impl AuthMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::UserPassword => "user_password",
            Self::SessionToken => "session_token",
        }
    }

    pub const fn grant_type(self) -> &'static str {
        match self {
            Self::Script => "client_credentials",
            Self::UserPassword => "password",
            Self::SessionToken => "session_token",
        }
    }
}

impl FromStr for AuthMode {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "script" | "client_credentials" => Ok(Self::Script),
            "user-password" | "user_password" | "password" | "user" => Ok(Self::UserPassword),
            "session-token" | "session_token" | "session" => Ok(Self::SessionToken),
            other => Err(AppError::invalid_input(format!(
                "unsupported auth mode `{other}`; expected one of: `script`, `user-password`, or `session-token`"
            ))
            .with_operation("parse_auth_mode")
            .with_invalid_field("auth_mode")
            .with_received_value(other)
            .with_allowed_values(["script", "user-password", "session-token"])
            .with_hint("Use one of the supported auth mode names in CLI flags, environment variables, or persisted config.")),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionOverrides {
    pub profile: Option<String>,
    pub site: Option<String>,
    pub auth_mode: Option<AuthMode>,
    pub script_name: Option<String>,
    pub script_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth_token: Option<String>,
    pub session_token: Option<String>,
    pub api_version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistedConnectionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<AuthMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub profiles: BTreeMap<String, PersistedConnectionProfile>,
}

/// Non-secret connection data for an agent-selectable local profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedConnectionProfile {
    pub site: String,
    pub auth_mode: AuthMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
}

/// Secrets are kept in the operating system credential store, never in config.json.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecureProfileSecrets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Credentials {
    Script {
        script_name: String,
        script_key: String,
    },
    UserPassword {
        username: String,
        password: String,
        auth_token: Option<String>,
    },
    SessionToken {
        session_token: String,
    },
}

impl Credentials {
    pub const fn auth_mode(&self) -> AuthMode {
        match self {
            Self::Script { .. } => AuthMode::Script,
            Self::UserPassword { .. } => AuthMode::UserPassword,
            Self::SessionToken { .. } => AuthMode::SessionToken,
        }
    }

    pub fn principal(&self) -> Option<String> {
        match self {
            Self::Script { script_name, .. } => Some(script_name.clone()),
            Self::UserPassword { username, .. } => Some(username.clone()),
            Self::SessionToken { .. } => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionSettings {
    pub site: String,
    pub credentials: Credentials,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionSummary {
    pub site: String,
    pub auth_mode: AuthMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    pub api_version: String,
}

impl ConnectionSettings {
    pub fn resolve(overrides: ConnectionOverrides) -> Result<Self> {
        let persisted = load_persisted_config()?;
        let profile_name = overrides
            .profile
            .clone()
            .or_else(|| env_var_compat("FPT_PROFILE", "SG_PROFILE"));
        if let Some(profile_name) = profile_name {
            return resolve_profile(&persisted, &profile_name);
        }
        let env_site = env_var_compat("FPT_SITE", "SG_SITE");
        let env_auth_mode = env_var_compat("FPT_AUTH_MODE", "SG_AUTH_MODE");
        let env_script_name = env_var_compat("FPT_SCRIPT_NAME", "SG_SCRIPT_NAME");
        let env_script_key = env_var_compat("FPT_SCRIPT_KEY", "SG_SCRIPT_KEY");
        let env_username = env_var_compat("FPT_USERNAME", "SG_USERNAME");
        let env_password = env_var_compat("FPT_PASSWORD", "SG_PASSWORD");
        let env_auth_token = env_var_compat("FPT_AUTH_TOKEN", "SG_AUTH_TOKEN");
        let env_session_token = env_var_compat("FPT_SESSION_TOKEN", "SG_SESSION_TOKEN");
        let env_api_version = env_var_compat("FPT_API_VERSION", "SG_API_VERSION");

        let site = overrides
            .site
            .or(env_site)
            .or(persisted.site)
            .unwrap_or_default();
        let script_name = overrides
            .script_name
            .or(env_script_name)
            .or(persisted.script_name)
            .unwrap_or_default();
        let script_key = overrides
            .script_key
            .or(env_script_key)
            .or(persisted.script_key)
            .unwrap_or_default();
        let username = overrides
            .username
            .or(env_username)
            .or(persisted.username)
            .unwrap_or_default();
        let password = overrides
            .password
            .or(env_password)
            .or(persisted.password)
            .unwrap_or_default();
        let auth_token = overrides
            .auth_token
            .or(env_auth_token)
            .or(persisted.auth_token)
            .unwrap_or_default();
        let session_token = overrides
            .session_token
            .or(env_session_token)
            .or(persisted.session_token)
            .unwrap_or_default();
        let api_version = api_version_or_default(
            overrides
                .api_version
                .as_deref()
                .or(env_api_version.as_deref())
                .or(persisted.api_version.as_deref()),
        );
        let auth_mode = overrides
            .auth_mode
            .or(parse_optional_auth_mode(env_auth_mode.as_deref())?)
            .or(persisted.auth_mode)
            .unwrap_or_else(|| infer_auth_mode(&username, &password, &auth_token, &session_token));

        let mut missing = Vec::new();
        if site.trim().is_empty() {
            missing.push("FPT_SITE / SG_SITE / config site / --site");
        }

        let credentials = match auth_mode {
            AuthMode::Script => {
                if script_name.trim().is_empty() {
                    missing.push(
                        "FPT_SCRIPT_NAME / SG_SCRIPT_NAME / config script_name / --script-name",
                    );
                }
                if script_key.trim().is_empty() {
                    missing
                        .push("FPT_SCRIPT_KEY / SG_SCRIPT_KEY / config script_key / --script-key");
                }

                Credentials::Script {
                    script_name,
                    script_key,
                }
            }
            AuthMode::UserPassword => {
                if username.trim().is_empty() {
                    missing.push("FPT_USERNAME / SG_USERNAME / config username / --username");
                }
                if password.trim().is_empty() {
                    missing.push("FPT_PASSWORD / SG_PASSWORD / config password / --password");
                }

                Credentials::UserPassword {
                    username,
                    password,
                    auth_token: (!auth_token.trim().is_empty()).then_some(auth_token),
                }
            }
            AuthMode::SessionToken => {
                if session_token.trim().is_empty() {
                    missing.push("FPT_SESSION_TOKEN / SG_SESSION_TOKEN / config session_token / --session-token");
                }

                Credentials::SessionToken { session_token }
            }
        };

        if !missing.is_empty() {
            return Err(AppError::invalid_input(
                "missing ShotGrid connection settings required to build an authenticated request"
            )
            .with_operation("resolve_connection_settings")
            .with_detail("auth_mode", auth_mode.as_str())
            .with_missing_fields(missing)
            .with_hint("Provide the missing values via CLI flags, environment variables, or `fpt config set`."));
        }

        Ok(Self {
            site: site.trim_end_matches('/').to_string(),
            credentials,
            api_version,
        })
    }

    pub fn auth_mode(&self) -> AuthMode {
        self.credentials.auth_mode()
    }

    pub fn summary(&self) -> ConnectionSummary {
        ConnectionSummary {
            site: self.site.clone(),
            auth_mode: self.auth_mode(),
            principal: self.credentials.principal(),
            profile: None,
            api_version: self.api_version.clone(),
        }
    }
}

fn resolve_profile(
    config: &PersistedConnectionConfig,
    profile_name: &str,
) -> Result<ConnectionSettings> {
    let profile = config.profiles.get(profile_name).ok_or_else(|| {
        AppError::invalid_input(format!("unknown FPT credential profile `{profile_name}`"))
            .with_operation("resolve_connection_profile")
            .with_invalid_field("profile")
            .with_hint("Create it interactively with `fpt auth login --profile <name> --site <url> --auth-mode user-password`.")
    })?;
    let secrets = load_profile_secrets(profile_name)?;
    let credentials = match profile.auth_mode {
        AuthMode::Script => Credentials::Script {
            script_name: required_profile_value(
                profile.script_name.as_deref(),
                profile_name,
                "script_name",
            )?,
            script_key: required_secret_value(secrets.script_key, profile_name, "script key")?,
        },
        AuthMode::UserPassword => Credentials::UserPassword {
            username: required_profile_value(
                profile.username.as_deref(),
                profile_name,
                "username",
            )?,
            password: required_secret_value(secrets.password, profile_name, "legacy passphrase")?,
            auth_token: secrets.auth_token.filter(|value| !value.trim().is_empty()),
        },
        AuthMode::SessionToken => Credentials::SessionToken {
            session_token: required_secret_value(
                secrets.session_token,
                profile_name,
                "session token",
            )?,
        },
    };

    Ok(ConnectionSettings {
        site: profile.site.trim_end_matches('/').to_string(),
        credentials,
        api_version: api_version_or_default(profile.api_version.as_deref()),
    })
}

pub fn save_profile(
    profile_name: &str,
    profile: PersistedConnectionProfile,
    secrets: SecureProfileSecrets,
) -> Result<PathBuf> {
    validate_profile_name(profile_name)?;
    let serialized = serde_json::to_string(&secrets).map_err(|error| {
        AppError::internal(format!(
            "could not serialize secure profile secrets: {error}"
        ))
        .with_operation("serialize_profile_secrets")
    })?;
    let entry = keyring::Entry::new("dcc-mcp/fpt-cli", profile_name).map_err(|error| {
        AppError::internal(format!(
            "could not access the system credential store: {error}"
        ))
        .with_operation("open_profile_keyring")
    })?;
    entry.set_password(&serialized).map_err(|error| {
        AppError::internal(format!(
            "could not save secure profile credentials: {error}"
        ))
        .with_operation("save_profile_secrets")
        .with_hint("Unlock the operating system credential store and retry.")
    })?;

    let mut config = load_persisted_config()?;
    config.profiles.insert(profile_name.to_string(), profile);
    save_persisted_config(&config)
}

pub fn remove_profile(profile_name: &str) -> Result<PathBuf> {
    validate_profile_name(profile_name)?;
    let mut config = load_persisted_config()?;
    if config.profiles.remove(profile_name).is_none() {
        return Err(AppError::invalid_input(format!(
            "unknown FPT credential profile `{profile_name}`"
        ))
        .with_operation("remove_connection_profile")
        .with_invalid_field("profile"));
    }
    let entry = keyring::Entry::new("dcc-mcp/fpt-cli", profile_name).map_err(|error| {
        AppError::internal(format!(
            "could not access the system credential store: {error}"
        ))
        .with_operation("open_profile_keyring")
    })?;
    let _ = entry.delete_credential();
    save_persisted_config(&config)
}

fn load_profile_secrets(profile_name: &str) -> Result<SecureProfileSecrets> {
    let entry = keyring::Entry::new("dcc-mcp/fpt-cli", profile_name).map_err(|error| {
        AppError::internal(format!(
            "could not access the system credential store: {error}"
        ))
        .with_operation("open_profile_keyring")
    })?;
    let serialized = entry.get_password().map_err(|error| {
        AppError::invalid_input(format!(
            "secure credentials for FPT profile `{profile_name}` are unavailable: {error}"
        ))
        .with_operation("load_profile_secrets")
        .with_hint("Run `fpt auth login --profile <name> ...` again on this machine.")
    })?;
    serde_json::from_str(&serialized).map_err(|error| {
        AppError::invalid_input(format!(
            "secure credentials for FPT profile `{profile_name}` are invalid: {error}"
        ))
        .with_operation("parse_profile_secrets")
    })
}

fn required_profile_value(value: Option<&str>, profile_name: &str, field: &str) -> Result<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            AppError::invalid_input(format!("FPT profile `{profile_name}` is missing {field}"))
                .with_operation("resolve_connection_profile")
        })
}

fn required_secret_value(value: Option<String>, profile_name: &str, field: &str) -> Result<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::invalid_input(format!(
                "FPT profile `{profile_name}` is missing secure {field}"
            ))
            .with_operation("resolve_connection_profile")
            .with_hint("Run `fpt auth login --profile <name> ...` again on this machine.")
        })
}

fn validate_profile_name(profile_name: &str) -> Result<()> {
    if profile_name.trim().is_empty() || profile_name.len() > 128 {
        return Err(AppError::invalid_input(
            "profile must contain 1-128 non-whitespace characters",
        )
        .with_operation("validate_connection_profile")
        .with_invalid_field("profile"));
    }
    Ok(())
}

pub fn api_version_or_default(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_API_VERSION)
        .to_string()
}

pub fn config_file_path() -> Result<PathBuf> {
    if let Some(path) = env::var_os("FPT_CONFIG_PATH") {
        return Ok(PathBuf::from(path));
    }

    let base_dir = if cfg!(windows) {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|home| home.join("AppData").join("Roaming")))
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|home| home.join(".config")))
    };

    base_dir
        .map(|dir| dir.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME))
        .ok_or_else(|| {
            AppError::internal(
                "could not resolve the configuration directory automatically; set `FPT_CONFIG_PATH` explicitly",
            )
            .with_operation("resolve_config_file_path")
            .with_hint("Set `FPT_CONFIG_PATH` to an explicit config file path if the platform config directory cannot be resolved.")
        })
}

pub fn load_persisted_config() -> Result<PersistedConnectionConfig> {
    let path = config_file_path()?;
    if !path.is_file() {
        return Ok(PersistedConnectionConfig::default());
    }

    let content = fs::read_to_string(&path).map_err(|error| {
        AppError::internal(format!(
            "could not read persisted config file `{}`: {error}",
            path.display()
        ))
        .with_operation("load_persisted_config")
        .with_resource(path.display().to_string())
        .with_hint("Check that the config file exists and is readable by the current user.")
    })?;

    serde_json::from_str(&content).map_err(|error| {
        AppError::invalid_input(format!(
            "persisted config file `{}` is not valid JSON: {error}",
            path.display()
        ))
        .with_operation("parse_persisted_config")
        .with_resource(path.display().to_string())
        .with_expected_shape("a JSON object matching the persisted connection config schema")
        .with_hint("Fix the JSON syntax in the config file or remove the invalid file and recreate it with `fpt config set`.")
    })
}

pub fn save_persisted_config(config: &PersistedConnectionConfig) -> Result<PathBuf> {
    let path = config_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::internal(format!(
                "could not create config directory `{}`: {error}",
                parent.display()
            ))
            .with_operation("create_config_directory")
            .with_resource(parent.display().to_string())
            .with_hint(
                "Check directory permissions or set `FPT_CONFIG_PATH` to a writable location.",
            )
        })?;
    }

    let mut contents = serde_json::to_string_pretty(config).map_err(|error| {
        AppError::internal(format!(
            "could not serialize persisted config as JSON: {error}"
        ))
        .with_operation("serialize_persisted_config")
    })?;
    contents.push('\n');
    fs::write(&path, &contents).map_err(|error| {
        AppError::internal(format!(
            "could not write persisted config file `{}`: {error}",
            path.display()
        ))
        .with_operation("save_persisted_config")
        .with_resource(path.display().to_string())
        .with_hint("Check file permissions or set `FPT_CONFIG_PATH` to a writable location.")
    })?;

    Ok(path)
}

pub(crate) fn resolve_site(overrides: ConnectionOverrides) -> Result<String> {
    let persisted = load_persisted_config()?;
    let site = overrides
        .site
        .or(env_var_compat("FPT_SITE", "SG_SITE"))
        .or(persisted.site)
        .unwrap_or_default();

    if site.trim().is_empty() {
        return Err(AppError::invalid_input(
            "missing ShotGrid site setting required for this command",
        )
        .with_operation("resolve_site")
        .with_missing_fields(["FPT_SITE / SG_SITE / config site / --site"])
        .with_invalid_field("site")
        .with_hint(
            "Provide the site via CLI flag, environment variable, or `fpt config set --site ...`.",
        ));
    }

    Ok(site.trim_end_matches('/').to_string())
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
}

fn env_var_compat(primary: &str, fallback: &str) -> Option<String> {
    env::var(primary)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var(fallback)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

fn parse_optional_auth_mode(value: Option<&str>) -> Result<Option<AuthMode>> {
    value.map(AuthMode::from_str).transpose()
}

fn infer_auth_mode(
    username: &str,
    password: &str,
    auth_token: &str,
    session_token: &str,
) -> AuthMode {
    if !username.trim().is_empty() || !password.trim().is_empty() || !auth_token.trim().is_empty() {
        return AuthMode::UserPassword;
    }

    if !session_token.trim().is_empty() {
        return AuthMode::SessionToken;
    }

    AuthMode::Script
}
