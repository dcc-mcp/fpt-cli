use fpt_core::{CommandSpec, RiskLevel};

const SELF_UPDATE_EXAMPLES: &[&str] = &[
    "fpt self update --check --output pretty-json",
    "fpt self update",
    "fpt self update --version 0.1.0",
];

const SELF_UPDATE_NOTES: &[&str] = &[
    "Downloads release assets over HTTPS from GitHub Releases",
    "Selects the release package that matches the current operating system and CPU architecture",
    "Verifies the downloaded archive against `fpt-checksums.txt` when the checksum asset is present",
    "`--check` is read-only; without it the command replaces the current executable in place",
    "The legacy `fpt self-update` form remains available as a compatibility alias",
];

const CONFIG_GET_EXAMPLES: &[&str] = &["fpt config get --output pretty-json"];
const CONFIG_PATH_EXAMPLES: &[&str] = &["fpt config path"];
const CONFIG_SET_EXAMPLES: &[&str] = &[
    "fpt config set --site https://example.shotgrid.autodesk.com --auth-mode session-token --session-token token-123",
    "fpt config set --site https://example.shotgrid.autodesk.com --auth-mode user-password --username user@example.com --password secret",
];
const CONFIG_CLEAR_EXAMPLES: &[&str] = &[
    "fpt config clear --fields session-token",
    "fpt config clear --fields site,auth-mode",
    "fpt config clear --all",
];

const CONFIG_NOTES: &[&str] = &[
    "Persisted config is used when command-line flags and environment variables are not provided",
    "Legacy configuration is stored in a local JSON file; use `auth login --profile` for new secrets",
    "Configuration output redacts any legacy secrets",
    "Use `config clear` to remove legacy values or `auth logout --profile` to remove secure profiles",
];

pub const SELF_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "self.update",
    summary: "Check or install the released CLI binary for the current platform",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "https",
    fallback_transport: None,
    input: "optional version / repository override",
    output: "json",
    examples: SELF_UPDATE_EXAMPLES,
    notes: SELF_UPDATE_NOTES,
};

pub const CONFIG_GET_SPEC: CommandSpec = CommandSpec {
    name: "config.get",
    summary: "Show the persisted local CLI configuration",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "core",
    fallback_transport: None,
    input: "none",
    output: "json",
    examples: CONFIG_GET_EXAMPLES,
    notes: CONFIG_NOTES,
};

pub const CONFIG_PATH_SPEC: CommandSpec = CommandSpec {
    name: "config.path",
    summary: "Show the file path of the persisted local CLI configuration",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "core",
    fallback_transport: None,
    input: "none",
    output: "json",
    examples: CONFIG_PATH_EXAMPLES,
    notes: CONFIG_NOTES,
};

pub const CONFIG_SET_SPEC: CommandSpec = CommandSpec {
    name: "config.set",
    summary: "Persist local CLI configuration values for later reuse",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "core",
    fallback_transport: None,
    input: "one or more config fields",
    output: "json",
    examples: CONFIG_SET_EXAMPLES,
    notes: CONFIG_NOTES,
};

pub const CONFIG_CLEAR_SPEC: CommandSpec = CommandSpec {
    name: "config.clear",
    summary: "Remove persisted local CLI configuration values",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "core",
    fallback_transport: None,
    input: "--all or --fields <name,...>",
    output: "json",
    examples: CONFIG_CLEAR_EXAMPLES,
    notes: CONFIG_NOTES,
};
