use serde::{Deserialize, Serialize};

/// Supported output formats for CLI responses.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// TOON encoding — a compact, deterministic serialization format that is
    /// smaller and faster to parse than JSON.  Intended for agent-to-agent
    /// pipelines where token efficiency matters.
    Toon,
    /// Minified JSON (default).
    #[default]
    Json,
    /// Pretty-printed JSON for human readability.
    PrettyJson,
}

/// Risk level classification for CLI commands.
///
/// Used by the command spec registry and dry-run planner to communicate the
/// operational risk of each command to agents and human operators.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Read-only operations that do not modify server state.
    Read,
    /// Operations that create or update server state.
    Write,
    /// Operations that permanently delete or retire data.
    Destructive,
}
