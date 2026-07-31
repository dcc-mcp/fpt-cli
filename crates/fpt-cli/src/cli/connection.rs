use clap::Args;
use fpt_domain::ConnectionOverrides;

use super::common::AuthModeArg;

#[derive(Debug, Args, Clone, Default)]
pub struct ConnectionArgs {
    #[arg(
        long,
        global = true,
        help = "Use a secure local FPT credential profile"
    )]
    pub profile: Option<String>,

    #[arg(long, global = true)]
    pub site: Option<String>,

    #[arg(long = "auth-mode", value_enum, global = true)]
    pub auth_mode: Option<AuthModeArg>,

    #[arg(long = "script-name", global = true)]
    pub script_name: Option<String>,

    #[arg(long = "script-key", global = true)]
    pub script_key: Option<String>,

    #[arg(long, global = true)]
    pub username: Option<String>,

    #[arg(long, global = true)]
    pub password: Option<String>,

    #[arg(long = "auth-token", global = true)]
    pub auth_token: Option<String>,

    #[arg(long = "session-token", global = true)]
    pub session_token: Option<String>,

    #[arg(long = "api-version", global = true)]
    pub api_version: Option<String>,
}

impl From<ConnectionArgs> for ConnectionOverrides {
    fn from(value: ConnectionArgs) -> Self {
        Self {
            profile: value.profile,
            site: value.site,
            auth_mode: value.auth_mode.map(Into::into),
            script_name: value.script_name,
            script_key: value.script_key,
            username: value.username,
            password: value.password,
            auth_token: value.auth_token,
            session_token: value.session_token,
            api_version: value.api_version,
        }
    }
}
