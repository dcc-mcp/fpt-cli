use fpt_core::{CommandSpec, RiskLevel};

const AUTH_TEST_EXAMPLES: &[&str] = &[
    "fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode script --script-name bot --script-key xxx",
    "fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode user-password --username user@example.com --password secret",
    "fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode session-token --session-token xxx",
];

const AUTH_NOTES: &[&str] = &[
    "Uses REST OAuth to obtain an access token",
    "Supports three auth modes: script, user-password, and session-token",
    "If 2FA is enabled, pass `--auth-token` as an additional parameter",
];

pub const AUTH_TEST_SPEC: CommandSpec = CommandSpec {
    name: "auth.test",
    summary: "Validate ShotGrid authentication via REST OAuth",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "site + one auth profile (script / user-password / session-token)",
    output: "json",
    examples: AUTH_TEST_EXAMPLES,
    notes: AUTH_NOTES,
};

const AUTH_LOGIN_EXAMPLES: &[&str] = &[
    "fpt auth login --profile gamedemo --site https://gamedemo.shotgrid.autodesk.com --auth-mode user-password --username artist@example.com --open-browser",
    "fpt user current --profile gamedemo",
];
const AUTH_LOGIN_NOTES: &[&str] = &[
    "Prompts for secrets locally and stores them in the operating system credential store",
    "The browser option guides Autodesk Identity users through site login and PAT setup; it does not expose browser cookies or tokens to the agent",
    "Use `fpt user current --profile <name>` to prove the effective ShotGrid identity",
];

pub const AUTH_LOGIN_SPEC: CommandSpec = CommandSpec {
    name: "auth.login",
    summary: "Create a secure local user or service credential profile",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "system_keyring",
    fallback_transport: None,
    input: "--profile, --site, auth mode, and non-secret identity fields; secrets are prompted locally",
    output: "json",
    examples: AUTH_LOGIN_EXAMPLES,
    notes: AUTH_LOGIN_NOTES,
};

const AUTH_LOGOUT_EXAMPLES: &[&str] = &["fpt auth logout --profile gamedemo"];

pub const AUTH_LOGOUT_SPEC: CommandSpec = CommandSpec {
    name: "auth.logout",
    summary: "Remove a secure local credential profile",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "system_keyring",
    fallback_transport: None,
    input: "--profile <name>",
    output: "json",
    examples: AUTH_LOGOUT_EXAMPLES,
    notes: &["Deletes both local profile metadata and the matching system credential entry"],
};
