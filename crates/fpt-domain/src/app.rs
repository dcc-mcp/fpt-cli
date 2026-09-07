mod activity;
mod auth;
pub mod batch;
mod entity;
mod find;
mod follow;
mod hierarchy;
mod license;
mod note;
mod query_helpers;
mod schedule;
mod schema;
mod server;
mod summarize;
mod upload;
mod user;
mod work_schedule;

use std::env;
use std::sync::OnceLock;

use fpt_core::{AppError, Result};
use serde_json::{Value, json};

use crate::capability::{command_specs, find_command_spec};
use crate::transport::{RestTransport, ShotgridTransport};

/// Top-level application facade that orchestrates `ShotGrid`/FPT operations.
///
/// `App` is generic over its transport layer (`T: ShotgridTransport`) so that
/// production code can use [`RestTransport`] while tests supply a recording or
/// stub transport.  The default type parameter is [`RestTransport`], making
/// `App::default()` ready for production use.
pub struct App<T = RestTransport> {
    transport: T,
}

impl Default for App<RestTransport> {
    fn default() -> Self {
        Self {
            transport: RestTransport::default(),
        }
    }
}

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub const fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn capabilities(&self, cli_version: &str) -> Value {
        json!({
            "name": "fpt",
            "version": cli_version,
            "transports": [
                { "name": "rest", "status": "available" },
                { "name": "rpc", "status": "planned" }
            ],
            "required_environment": [
                "FPT_SITE"
            ],
            "optional_environment": [
                "FPT_AUTH_MODE",
                "FPT_SCRIPT_NAME",
                "FPT_SCRIPT_KEY",
                "FPT_USERNAME",
                "FPT_PASSWORD",
                "FPT_AUTH_TOKEN",
                "FPT_SESSION_TOKEN",
                "FPT_API_VERSION"
            ],
            "auth_modes": [
                {
                    "name": "script",
                    "grant_type": "client_credentials",
                    "required": ["FPT_SCRIPT_NAME", "FPT_SCRIPT_KEY"]
                },
                {
                    "name": "user_password",
                    "grant_type": "password",
                    "required": ["FPT_USERNAME", "FPT_PASSWORD"],
                    "optional": ["FPT_AUTH_TOKEN"]
                },
                {
                    "name": "session_token",
                    "grant_type": "session_token",
                    "required": ["FPT_SESSION_TOKEN"]
                }
            ],
            "commands": command_specs(),
        })
    }

    pub fn inspect_command(&self, name: &str) -> Result<Value> {
        let spec = find_command_spec(name)
            .ok_or_else(|| AppError::unsupported(format!("unknown command `{name}`")))?;
        Ok(json!(spec))
    }

    /// Return a flat JSON array of all available command names.
    ///
    /// This enables agent-side schema introspection without loading the full
    /// capabilities payload: the agent can call `inspect list` to get all
    /// command names, then use `inspect command <name>` to fetch the contract
    /// for only the command(s) it actually needs.  This pattern keeps context
    /// window usage low and avoids over-fetching capabilities.
    pub fn inspect_list(&self) -> Value {
        let names: Vec<&str> = command_specs().iter().map(|s| s.name).collect();
        json!({
            "commands": names,
            "count": names.len(),
        })
    }
}

const DEFAULT_BATCH_CONCURRENCY: usize = 8;
const MAX_BATCH_CONCURRENCY: usize = 32;

/// Environment variable for overriding the default batch concurrency limit.
const ENV_FPT_BATCH_CONCURRENCY: &str = "FPT_BATCH_CONCURRENCY";

/// Returns the batch concurrency limit, configurable via `FPT_BATCH_CONCURRENCY`.
///
/// The result is cached on first access via [`OnceLock`] so that repeated
/// calls within batch operations avoid redundant env-var lookups.
fn batch_concurrency_limit() -> usize {
    static CACHED: OnceLock<usize> = OnceLock::new();
    *CACHED.get_or_init(|| {
        env::var(ENV_FPT_BATCH_CONCURRENCY)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .map_or(DEFAULT_BATCH_CONCURRENCY, |value| {
                value.min(MAX_BATCH_CONCURRENCY)
            })
    })
}

fn sort_batch_results(results: &mut [Value]) {
    results.sort_by_key(batch_result_index);
}

fn batch_result_index(value: &Value) -> usize {
    value
        .get("index")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX)
}

#[derive(Debug, Clone)]
struct BatchUpdateItem {
    id: u64,
    body: Value,
}
