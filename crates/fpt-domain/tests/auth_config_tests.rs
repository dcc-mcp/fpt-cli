use fpt_domain::{
    AuthMode, ConnectionOverrides, ConnectionSettings, Credentials, PersistedConnectionConfig,
    save_persisted_config,
};
use std::env;
use std::fs;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvSnapshot {
    key: String,
    previous: Option<String>,
}

struct EnvGuard {
    snapshots: Vec<EnvSnapshot>,
}

impl EnvGuard {
    fn set(vars: &[(&str, Option<&str>)]) -> Self {
        let mut snapshots = Vec::with_capacity(vars.len());
        for (key, value) in vars {
            snapshots.push(EnvSnapshot {
                key: (*key).to_string(),
                previous: env::var(key).ok(),
            });
            unsafe {
                match value {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }
        }
        Self { snapshots }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for snapshot in &self.snapshots {
            unsafe {
                match &snapshot.previous {
                    Some(value) => env::set_var(&snapshot.key, value),
                    None => env::remove_var(&snapshot.key),
                }
            }
        }
    }
}

struct TempConfigPath {
    path: std::path::PathBuf,
}

impl TempConfigPath {
    fn new(name: &str) -> Self {
        let path = env::temp_dir().join(format!(
            "fpt-cli-config-test-{name}-{}.json",
            std::process::id()
        ));
        Self { path }
    }

    fn path_str(&self) -> &str {
        self.path.to_str().expect("temp config path is utf-8")
    }
}

impl Drop for TempConfigPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn resolves_script_auth_with_explicit_mode() {
    let _lock = env_lock().lock().expect("env lock");
    let config_path = TempConfigPath::new("script-explicit");
    let _guard = EnvGuard::set(&[
        ("FPT_CONFIG_PATH", Some(config_path.path_str())),
        ("FPT_PROFILE", None),
        ("SG_PROFILE", None),
        ("FPT_AUTH_MODE", None),
        ("SG_AUTH_MODE", None),
    ]);

    let settings = ConnectionSettings::resolve(ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::Script),
        script_name: Some("bot".to_string()),
        script_key: Some("secret".to_string()),
        ..Default::default()
    })
    .expect("script auth resolves");

    assert_eq!(settings.auth_mode(), AuthMode::Script);
    match settings.credentials {
        Credentials::Script {
            script_name,
            script_key,
        } => {
            assert_eq!(script_name, "bot");
            assert_eq!(script_key, "secret");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn infers_user_password_auth_from_username_and_password() {
    let _lock = env_lock().lock().expect("env lock");
    let config_path = TempConfigPath::new("infer-user-password");
    let _guard = EnvGuard::set(&[
        ("FPT_CONFIG_PATH", Some(config_path.path_str())),
        ("FPT_AUTH_MODE", None),
        ("SG_AUTH_MODE", None),
        ("FPT_SCRIPT_NAME", None),
        ("FPT_SCRIPT_KEY", None),
        ("SG_SCRIPT_NAME", None),
        ("SG_SCRIPT_KEY", None),
        ("FPT_SESSION_TOKEN", None),
        ("SG_SESSION_TOKEN", None),
        ("FPT_AUTH_TOKEN", None),
        ("SG_AUTH_TOKEN", None),
    ]);

    let settings = ConnectionSettings::resolve(ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com".to_string()),
        username: Some("artist@example.com".to_string()),
        password: Some("secret".to_string()),
        ..Default::default()
    })
    .expect("user password auth resolves");

    assert_eq!(settings.auth_mode(), AuthMode::UserPassword);
    match settings.credentials {
        Credentials::UserPassword {
            username,
            password,
            auth_token,
        } => {
            assert_eq!(username, "artist@example.com");
            assert_eq!(password, "secret");
            assert!(auth_token.is_none());
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn resolves_session_token_auth_with_explicit_mode() {
    let _lock = env_lock().lock().expect("env lock");
    let config_path = TempConfigPath::new("session-explicit");
    let _guard = EnvGuard::set(&[
        ("FPT_CONFIG_PATH", Some(config_path.path_str())),
        ("FPT_SITE", None),
        ("FPT_AUTH_MODE", None),
        ("FPT_SCRIPT_NAME", None),
        ("FPT_SCRIPT_KEY", None),
        ("FPT_USERNAME", None),
        ("FPT_PASSWORD", None),
        ("FPT_AUTH_TOKEN", None),
        ("FPT_SESSION_TOKEN", None),
        ("SG_SITE", None),
        ("SG_AUTH_MODE", None),
        ("SG_SCRIPT_NAME", None),
        ("SG_SCRIPT_KEY", None),
        ("SG_USERNAME", None),
        ("SG_PASSWORD", None),
        ("SG_AUTH_TOKEN", None),
        ("SG_SESSION_TOKEN", None),
    ]);

    let settings = ConnectionSettings::resolve(ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::SessionToken),
        session_token: Some("session-123".to_string()),
        ..Default::default()
    })
    .expect("session token auth resolves");

    assert_eq!(settings.auth_mode(), AuthMode::SessionToken);
    match settings.credentials {
        Credentials::SessionToken { session_token } => {
            assert_eq!(session_token, "session-123");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn resolves_script_auth_from_sg_env_fallback() {
    let _lock = env_lock().lock().expect("env lock");
    let _guard = EnvGuard::set(&[
        ("SG_SITE", Some("https://sg-env.shotgrid.autodesk.com")),
        ("SG_SCRIPT_NAME", Some("sg-bot")),
        ("SG_SCRIPT_KEY", Some("sg-secret")),
        ("FPT_SITE", None),
        ("FPT_SCRIPT_NAME", None),
        ("FPT_SCRIPT_KEY", None),
        ("FPT_AUTH_MODE", None),
        ("SG_AUTH_MODE", None),
        ("FPT_USERNAME", None),
        ("FPT_PASSWORD", None),
        ("FPT_AUTH_TOKEN", None),
        ("FPT_SESSION_TOKEN", None),
        ("SG_USERNAME", None),
        ("SG_PASSWORD", None),
        ("SG_AUTH_TOKEN", None),
        ("SG_SESSION_TOKEN", None),
    ]);

    let settings = ConnectionSettings::resolve(ConnectionOverrides::default())
        .expect("script auth resolves from SG env");

    assert_eq!(settings.site, "https://sg-env.shotgrid.autodesk.com");
    assert_eq!(settings.auth_mode(), AuthMode::Script);
    match settings.credentials {
        Credentials::Script {
            script_name,
            script_key,
        } => {
            assert_eq!(script_name, "sg-bot");
            assert_eq!(script_key, "sg-secret");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn prefers_fpt_env_over_sg_env_when_both_present() {
    let _lock = env_lock().lock().expect("env lock");
    let _guard = EnvGuard::set(&[
        ("FPT_SITE", Some("https://fpt-env.shotgrid.autodesk.com")),
        ("FPT_SCRIPT_NAME", Some("fpt-bot")),
        ("FPT_SCRIPT_KEY", Some("fpt-secret")),
        ("SG_SITE", Some("https://sg-env.shotgrid.autodesk.com")),
        ("SG_SCRIPT_NAME", Some("sg-bot")),
        ("SG_SCRIPT_KEY", Some("sg-secret")),
        ("FPT_AUTH_MODE", None),
        ("SG_AUTH_MODE", None),
        ("FPT_USERNAME", None),
        ("FPT_PASSWORD", None),
        ("FPT_AUTH_TOKEN", None),
        ("FPT_SESSION_TOKEN", None),
        ("SG_USERNAME", None),
        ("SG_PASSWORD", None),
        ("SG_AUTH_TOKEN", None),
        ("SG_SESSION_TOKEN", None),
    ]);

    let settings = ConnectionSettings::resolve(ConnectionOverrides::default())
        .expect("script auth resolves from FPT env");

    assert_eq!(settings.site, "https://fpt-env.shotgrid.autodesk.com");
    assert_eq!(settings.auth_mode(), AuthMode::Script);
    match settings.credentials {
        Credentials::Script {
            script_name,
            script_key,
        } => {
            assert_eq!(script_name, "fpt-bot");
            assert_eq!(script_key, "fpt-secret");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn resolves_connection_from_persisted_config_when_env_and_flags_are_missing() {
    let _lock = env_lock().lock().expect("env lock");
    let config_path = TempConfigPath::new("persisted-fallback");
    let _guard = EnvGuard::set(&[
        ("FPT_CONFIG_PATH", Some(config_path.path_str())),
        ("FPT_SITE", None),
        ("FPT_AUTH_MODE", None),
        ("FPT_SCRIPT_NAME", None),
        ("FPT_SCRIPT_KEY", None),
        ("FPT_USERNAME", None),
        ("FPT_PASSWORD", None),
        ("FPT_AUTH_TOKEN", None),
        ("FPT_SESSION_TOKEN", None),
        ("FPT_API_VERSION", None),
        ("SG_SITE", None),
        ("SG_AUTH_MODE", None),
        ("SG_SCRIPT_NAME", None),
        ("SG_SCRIPT_KEY", None),
        ("SG_USERNAME", None),
        ("SG_PASSWORD", None),
        ("SG_AUTH_TOKEN", None),
        ("SG_SESSION_TOKEN", None),
        ("SG_API_VERSION", None),
    ]);

    save_persisted_config(&PersistedConnectionConfig {
        site: Some("https://persisted.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::SessionToken),
        session_token: Some("persisted-session".to_string()),
        api_version: Some("v1.2".to_string()),
        ..Default::default()
    })
    .expect("persisted config saved");

    let settings = ConnectionSettings::resolve(ConnectionOverrides::default())
        .expect("settings resolve from persisted config");

    assert_eq!(settings.site, "https://persisted.shotgrid.autodesk.com");
    assert_eq!(settings.auth_mode(), AuthMode::SessionToken);
    assert_eq!(settings.api_version, "v1.2");
    match settings.credentials {
        Credentials::SessionToken { session_token } => {
            assert_eq!(session_token, "persisted-session");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn prefers_environment_over_persisted_config() {
    let _lock = env_lock().lock().expect("env lock");
    let config_path = TempConfigPath::new("persisted-precedence");
    let _guard = EnvGuard::set(&[
        ("FPT_CONFIG_PATH", Some(config_path.path_str())),
        ("FPT_SITE", Some("https://env.shotgrid.autodesk.com")),
        ("FPT_AUTH_MODE", Some("script")),
        ("FPT_SCRIPT_NAME", Some("env-bot")),
        ("FPT_SCRIPT_KEY", Some("env-secret")),
        ("FPT_SESSION_TOKEN", None),
        ("FPT_USERNAME", None),
        ("FPT_PASSWORD", None),
        ("FPT_AUTH_TOKEN", None),
        ("FPT_API_VERSION", None),
        ("SG_SITE", None),
        ("SG_AUTH_MODE", None),
        ("SG_SCRIPT_NAME", None),
        ("SG_SCRIPT_KEY", None),
        ("SG_USERNAME", None),
        ("SG_PASSWORD", None),
        ("SG_AUTH_TOKEN", None),
        ("SG_SESSION_TOKEN", None),
        ("SG_API_VERSION", None),
    ]);

    save_persisted_config(&PersistedConnectionConfig {
        site: Some("https://persisted.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::SessionToken),
        session_token: Some("persisted-session".to_string()),
        ..Default::default()
    })
    .expect("persisted config saved");

    let settings = ConnectionSettings::resolve(ConnectionOverrides::default())
        .expect("settings resolve from env before persisted config");

    assert_eq!(settings.site, "https://env.shotgrid.autodesk.com");
    assert_eq!(settings.auth_mode(), AuthMode::Script);
    match settings.credentials {
        Credentials::Script {
            script_name,
            script_key,
        } => {
            assert_eq!(script_name, "env-bot");
            assert_eq!(script_key, "env-secret");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}
