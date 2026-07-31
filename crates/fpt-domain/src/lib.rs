#![allow(clippy::result_large_err)]

pub mod app;
pub mod capability;
pub mod config;
pub mod filter_dsl;
pub mod transport;

pub use app::App;
pub use config::{
    AuthMode, ConnectionOverrides, ConnectionSettings, Credentials, PersistedConnectionConfig,
    PersistedConnectionProfile, SecureProfileSecrets, api_version_or_default, config_file_path,
    load_persisted_config, remove_profile, save_persisted_config, save_profile,
};
pub use filter_dsl::parse_filter_dsl;
pub use transport::{
    FindParams, RequestPlan, RestTransport, ShotgridTransport, entity_collection_path,
};

/// Convenience alias for the default production `App` backed by `RestTransport`.
///
/// Other Rust crates (or FFI / PyO3 bindings) can depend on `fpt-domain` and use
/// `ShotgridApp` directly without constructing the transport layer manually:
///
/// ```ignore
/// use fpt_domain::{ShotgridApp, ConnectionOverrides};
///
/// let app = ShotgridApp::default();
/// let result = app.entity_get(overrides, "Shot", 42, None).await?;
/// ```
pub type ShotgridApp = App<RestTransport>;

/// Re-exports the most commonly used types for convenient glob imports.
///
/// ```ignore
/// use fpt_domain::prelude::*;
/// ```
pub mod prelude {
    pub use fpt_core::{AppError, Result};
    pub use serde_json::{Value, json};

    pub use crate::ShotgridApp;
    pub use crate::app::App;
    pub use crate::config::{AuthMode, ConnectionOverrides, ConnectionSettings};
    pub use crate::transport::{FindParams, RestTransport, ShotgridTransport};
}
