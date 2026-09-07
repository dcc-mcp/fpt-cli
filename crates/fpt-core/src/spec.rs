use crate::model::RiskLevel;
use serde::Serialize;

/// Machine-readable specification for a single CLI command.
///
/// These specs are surfaced through `fpt capabilities` and `fpt inspect command <name>`
/// so that agents can introspect the command contract without parsing help text.
/// Each spec is defined as a `const` in the corresponding capability module.
#[derive(Debug, Clone, Serialize)]
pub struct CommandSpec {
    /// Dot-separated command name (e.g. `"entity.find"`, `"auth.test"`).
    pub name: &'static str,
    /// One-line human-readable summary of what the command does.
    pub summary: &'static str,
    /// Operational risk classification.
    pub risk: RiskLevel,
    /// Whether the command is currently implemented and callable.
    pub implemented: bool,
    /// Whether the command supports `--dry-run` to preview without executing.
    pub supports_dry_run: bool,
    /// Primary transport layer used by this command (`"rest"` or `"rpc"`).
    pub preferred_transport: &'static str,
    /// Optional secondary transport if the primary is unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_transport: Option<&'static str>,
    /// Description of the expected input shape (e.g. `"JSON object"`, `"none"`).
    pub input: &'static str,
    /// Description of the expected output shape.
    pub output: &'static str,
    /// Example CLI invocations.
    pub examples: &'static [&'static str],
    /// Additional notes for agents or human operators.
    pub notes: &'static [&'static str],
}
