use crate::cli::{ConfigClearArgs, ConfigCommands, ConfigSetArgs};
use fpt_core::{AppError, Result};
use fpt_domain::{
    AuthMode, ConnectionOverrides, PersistedConnectionConfig, PersistedConnectionProfile,
    SecureProfileSecrets, config_file_path, load_persisted_config, remove_profile,
    save_persisted_config, save_profile,
};
use serde_json::{Value, json};
use std::{
    env,
    io::{self, Write},
    process::Command,
};

pub fn run(command: ConfigCommands) -> Result<Value> {
    match command {
        ConfigCommands::Get => get_config(),
        ConfigCommands::Path => {
            let path = config_file_path()?;
            Ok(json!({
                "command": "config.path",
                "path": path.display().to_string(),
            }))
        }
        ConfigCommands::Set(args) => set_config(args),
        ConfigCommands::Clear(args) => clear_config(args),
    }
}

pub fn login(overrides: ConnectionOverrides, open_browser: bool) -> Result<Value> {
    let profile = required_login_value(overrides.profile.as_deref(), "--profile")?;
    let env_site = env_value("FPT_SITE", "SG_SITE");
    let site = required_login_value(overrides.site.as_deref().or(env_site.as_deref()), "--site")?;
    let env_auth_mode = env_value("FPT_AUTH_MODE", "SG_AUTH_MODE");
    let auth_mode = overrides
        .auth_mode
        .or(env_auth_mode.as_deref().map(str::parse).transpose()?)
        .unwrap_or(AuthMode::UserPassword);

    if open_browser {
        open_site(&site)?;
    }

    let (script_name, username, secrets) = match auth_mode {
        AuthMode::Script => {
            let env_script_name = env_value("FPT_SCRIPT_NAME", "SG_SCRIPT_NAME");
            let script_name = required_login_value(
                overrides
                    .script_name
                    .as_deref()
                    .or(env_script_name.as_deref()),
                "--script-name",
            )?;
            let script_key = prompt_secret("FPT script key: ")?;
            (
                Some(script_name),
                None,
                SecureProfileSecrets {
                    script_key: Some(script_key),
                    ..Default::default()
                },
            )
        }
        AuthMode::UserPassword => {
            let env_username = env_value("FPT_USERNAME", "SG_USERNAME");
            let username = required_login_value(
                overrides.username.as_deref().or(env_username.as_deref()),
                "--username",
            )?;
            let password = prompt_secret("FPT legacy passphrase: ")?;
            let auth_token = prompt_optional_secret(
                "FPT personal access token or MFA code (leave empty when unused): ",
            )?;
            (
                None,
                Some(username),
                SecureProfileSecrets {
                    password: Some(password),
                    auth_token,
                    ..Default::default()
                },
            )
        }
        AuthMode::SessionToken => {
            let session_token = prompt_secret("FPT session token: ")?;
            (
                None,
                None,
                SecureProfileSecrets {
                    session_token: Some(session_token),
                    ..Default::default()
                },
            )
        }
    };
    let path = save_profile(
        &profile,
        PersistedConnectionProfile {
            site: site.clone(),
            auth_mode,
            script_name,
            username,
            api_version: overrides.api_version,
        },
        secrets,
    )?;
    Ok(json!({
        "command": "auth.login",
        "profile": profile,
        "site": site,
        "auth_mode": auth_mode,
        "credential_store": "system_keyring",
        "browser_opened": open_browser,
        "next_step": "Run `fpt user current --profile <profile>` to verify the authenticated user.",
        "config_path": path.display().to_string(),
    }))
}

pub fn logout(profile: Option<&str>) -> Result<Value> {
    let profile = required_login_value(profile, "--profile")?;
    let path = remove_profile(&profile)?;
    Ok(json!({
        "command": "auth.logout",
        "profile": profile,
        "config_path": path.display().to_string(),
    }))
}

fn required_login_value(value: Option<&str>, flag: &str) -> Result<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| AppError::invalid_input(format!("`auth login` requires {flag}")))
}

fn prompt_secret(prompt: &str) -> Result<String> {
    let value = rpassword::prompt_password(prompt).map_err(|error| {
        AppError::internal(format!("could not read secure terminal input: {error}"))
    })?;
    required_login_value(Some(&value), "a non-empty secret")
}

fn prompt_optional_secret(prompt: &str) -> Result<Option<String>> {
    let value = rpassword::prompt_password(prompt).map_err(|error| {
        AppError::internal(format!("could not read secure terminal input: {error}"))
    })?;
    Ok((!value.trim().is_empty()).then_some(value))
}

fn env_value(primary: &str, fallback: &str) -> Option<String> {
    env::var(primary)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var(fallback)
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}

fn open_site(site: &str) -> Result<()> {
    let mut command = if cfg!(target_os = "windows") {
        Command::new("explorer.exe")
    } else if cfg!(target_os = "macos") {
        Command::new("open")
    } else {
        Command::new("xdg-open")
    };
    command.arg(site);
    command.status().map_err(|error| {
        AppError::internal(format!("could not open the default browser: {error}"))
            .with_operation("open_auth_browser")
            .with_hint("Open the Flow Production Tracking site manually, then rerun without --open-browser.")
    })?;
    io::stderr().flush().ok();
    Ok(())
}

fn get_config() -> Result<Value> {
    let path = config_file_path()?;
    let config = load_persisted_config()?;
    Ok(json!({
        "command": "config.get",
        "path": path.display().to_string(),
        "config": config_diagnostics(&config),
    }))
}

fn set_config(args: ConfigSetArgs) -> Result<Value> {
    if !has_any_set_arg(&args) {
        return Err(AppError::invalid_input(
            "`config set` requires at least one field to persist; pass options such as `--site`, `--auth-mode`, or credentials flags",
        ));
    }

    let mut config = load_persisted_config()?;
    if let Some(site) = args.site {
        config.site = Some(site.trim().trim_end_matches('/').to_string());
    }
    if let Some(auth_mode) = args.auth_mode {
        config.auth_mode = Some(auth_mode.into());
    }
    if let Some(script_name) = args.script_name {
        config.script_name = Some(script_name.trim().to_string());
    }
    if let Some(script_key) = args.script_key {
        config.script_key = Some(script_key.trim().to_string());
    }
    if let Some(username) = args.username {
        config.username = Some(username.trim().to_string());
    }
    if let Some(password) = args.password {
        config.password = Some(password);
    }
    if let Some(auth_token) = args.auth_token {
        config.auth_token = Some(auth_token.trim().to_string());
    }
    if let Some(session_token) = args.session_token {
        config.session_token = Some(session_token.trim().to_string());
    }
    if let Some(api_version) = args.api_version {
        config.api_version = Some(api_version.trim().to_string());
    }

    let path = save_persisted_config(&config)?;
    Ok(json!({
        "command": "config.set",
        "path": path.display().to_string(),
        "config": config_diagnostics(&config),
    }))
}

/// All field names accepted by `config clear --fields`.
const VALID_CLEAR_FIELDS: &[&str] = &[
    "site",
    "auth-mode",
    "script-name",
    "script-key",
    "username",
    "password",
    "auth-token",
    "session-token",
    "api-version",
];

fn clear_config(args: ConfigClearArgs) -> Result<Value> {
    if !args.all && args.fields.is_empty() {
        return Err(AppError::invalid_input(
            "`config clear` requires `--all` or `--fields <name,...>`; \
             valid field names: site, auth-mode, script-name, script-key, \
             username, password, auth-token, session-token, api-version",
        ));
    }

    // Validate field names before doing any work.
    for name in &args.fields {
        if !VALID_CLEAR_FIELDS.contains(&name.as_str()) {
            return Err(AppError::invalid_input(format!(
                "unknown field name `{name}`; valid names: {}",
                VALID_CLEAR_FIELDS.join(", ")
            )));
        }
    }

    let mut config = load_persisted_config()?;
    if args.all {
        config = PersistedConnectionConfig::default();
    } else {
        let f = &args.fields;
        let has = |name: &str| f.iter().any(|n| n == name);
        if has("site") {
            config.site = None;
        }
        if has("auth-mode") {
            config.auth_mode = None;
        }
        if has("script-name") {
            config.script_name = None;
        }
        if has("script-key") {
            config.script_key = None;
        }
        if has("username") {
            config.username = None;
        }
        if has("password") {
            config.password = None;
        }
        if has("auth-token") {
            config.auth_token = None;
        }
        if has("session-token") {
            config.session_token = None;
        }
        if has("api-version") {
            config.api_version = None;
        }
    }

    let path = save_persisted_config(&config)?;
    Ok(json!({
        "command": "config.clear",
        "path": path.display().to_string(),
        "config": config_diagnostics(&config),
    }))
}

fn config_diagnostics(config: &PersistedConnectionConfig) -> Value {
    json!({
        "site": config.site,
        "auth_mode": config.auth_mode,
        "script_name": config.script_name,
        "script_key": config.script_key.as_ref().map(|_| "<redacted>"),
        "username": config.username,
        "password": config.password.as_ref().map(|_| "<redacted>"),
        "auth_token": config.auth_token.as_ref().map(|_| "<redacted>"),
        "session_token": config.session_token.as_ref().map(|_| "<redacted>"),
        "api_version": config.api_version,
        "profiles": config.profiles,
    })
}

const fn has_any_set_arg(args: &ConfigSetArgs) -> bool {
    args.site.is_some()
        || args.auth_mode.is_some()
        || args.script_name.is_some()
        || args.script_key.is_some()
        || args.username.is_some()
        || args.password.is_some()
        || args.auth_token.is_some()
        || args.session_token.is_some()
        || args.api_version.is_some()
}
