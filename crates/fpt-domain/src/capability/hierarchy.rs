use fpt_core::{CommandSpec, RiskLevel};

const HIERARCHY_SEARCH_EXAMPLES: &[&str] = &[
    "fpt hierarchy search --input @hierarchy_query.json --site ... --auth-mode script --script-name ... --script-key ...",
];

const HIERARCHY_SEARCH_NOTES: &[&str] = &[
    "Search the entity hierarchy tree using the ShotGrid hierarchy/_search endpoint",
    "Input must be a JSON body describing the hierarchy search criteria (root_entity, seed_entity_field, entity_fields, etc.)",
];

const HIERARCHY_EXPAND_EXAMPLES: &[&str] = &[
    "fpt hierarchy expand --input @expand_query.json --site ... --auth-mode script --script-name ... --script-key ...",
];

const HIERARCHY_EXPAND_NOTES: &[&str] = &[
    "Expand a specific node in the hierarchy tree using the ShotGrid hierarchy/_expand endpoint",
    "Input must be a JSON body containing the `path` and optionally `entity_fields` for the expand request",
];

pub const HIERARCHY_SEARCH_SPEC: CommandSpec = CommandSpec {
    name: "hierarchy.search",
    summary: "Search the ShotGrid entity hierarchy navigation tree",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "hierarchy search body JSON",
    output: "json",
    examples: HIERARCHY_SEARCH_EXAMPLES,
    notes: HIERARCHY_SEARCH_NOTES,
};

pub const HIERARCHY_EXPAND_SPEC: CommandSpec = CommandSpec {
    name: "hierarchy.expand",
    summary: "Expand a node in the ShotGrid entity hierarchy tree",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "hierarchy expand body JSON with path",
    output: "json",
    examples: HIERARCHY_EXPAND_EXAMPLES,
    notes: HIERARCHY_EXPAND_NOTES,
};
