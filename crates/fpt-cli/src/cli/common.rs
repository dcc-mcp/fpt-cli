use clap::ValueEnum;
use fpt_core::OutputFormat;
use fpt_domain::AuthMode;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormatArg {
    Toon,
    Json,
    PrettyJson,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(value: OutputFormatArg) -> Self {
        match value {
            OutputFormatArg::Toon => Self::Toon,
            OutputFormatArg::Json => Self::Json,
            OutputFormatArg::PrettyJson => Self::PrettyJson,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AuthModeArg {
    Script,
    UserPassword,
    SessionToken,
}

impl From<AuthModeArg> for AuthMode {
    fn from(value: AuthModeArg) -> Self {
        match value {
            AuthModeArg::Script => Self::Script,
            AuthModeArg::UserPassword => Self::UserPassword,
            AuthModeArg::SessionToken => Self::SessionToken,
        }
    }
}
