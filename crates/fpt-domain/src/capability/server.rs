use fpt_core::{CommandSpec, RiskLevel};

const SERVER_INFO_EXAMPLES: &[&str] =
    &["fpt server info --site https://example.shotgrid.autodesk.com --output json"];

const SERVER_INFO_NOTES: &[&str] = &[
    "Uses the ShotGrid RPC `info` method over `/api3/json`",
    "Requires only `--site`; auth flags are ignored for this command",
    "Returns raw server metadata such as version and authentication mode",
];

pub const SERVER_INFO_SPEC: CommandSpec = CommandSpec {
    name: "server.info",
    summary: "Fetch ShotGrid server metadata",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rpc",
    fallback_transport: None,
    input: "site",
    output: "json",
    examples: SERVER_INFO_EXAMPLES,
    notes: SERVER_INFO_NOTES,
};

const REST_VERSION_EXAMPLES: &[&str] = &[
    "fpt server version --site https://example.shotgrid.autodesk.com --output json",
];

const REST_VERSION_NOTES: &[&str] = &[
    "Calls `GET /api/{version}/` on the ShotGrid REST API",
    "No authentication required — returns server and API version metadata",
    "Useful for verifying REST API connectivity without credentials",
];

pub const SERVER_REST_VERSION_SPEC: CommandSpec = CommandSpec {
    name: "server.version",
    summary: "Fetch REST API version and server metadata (unauthenticated)",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "site",
    output: "json",
    examples: REST_VERSION_EXAMPLES,
    notes: REST_VERSION_NOTES,
};

const OPENAPI_SPEC_EXAMPLES: &[&str] = &[
    "fpt server openapi-spec --site https://example.shotgrid.autodesk.com --output json",
    "fpt server openapi-spec --site https://example.shotgrid.autodesk.com --format yaml --output json",
];

const OPENAPI_SPEC_NOTES: &[&str] = &[
    "Calls `GET /api/{version}/spec.{format}` on the ShotGrid REST API",
    "No authentication required — downloads the OpenAPI v3 specification",
    "Supports JSON (default) and YAML formats via the `--format` flag",
    "YAML responses are wrapped in a JSON envelope with `format` and `content` fields",
];

pub const SERVER_OPENAPI_SPEC_SPEC: CommandSpec = CommandSpec {
    name: "server.openapi-spec",
    summary: "Download the OpenAPI v3 specification for the ShotGrid REST API (unauthenticated)",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "site",
    output: "json",
    examples: OPENAPI_SPEC_EXAMPLES,
    notes: OPENAPI_SPEC_NOTES,
};
