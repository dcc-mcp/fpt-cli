use fpt_core::{CommandSpec, RiskLevel};

const ENTITY_GET_EXAMPLES: &[&str] = &[
    "fpt entity get Shot 123 --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt entity get Shot 123 --site ... --auth-mode session-token --session-token ...",
];
const ENTITY_FIND_EXAMPLES: &[&str] = &[
    "fpt entity find Asset --input @query.json --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt entity find Asset --filter-dsl 'sg_status_list == \"ip\" and (code ~ \"bunny\" or id > 100)' --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt entity find Version --input @search.json --site ... --auth-mode script --script-name ... --script-key ...",
];
const ENTITY_FIND_ONE_EXAMPLES: &[&str] = &[
    "fpt entity find-one Shot --input @query.json --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt entity find-one Version --filter-dsl 'code ~ \"hero\"' --site ... --auth-mode script --script-name ... --script-key ...",
];
const ENTITY_SUMMARIZE_EXAMPLES: &[&str] = &[
    "fpt entity summarize Version --input @summaries.json --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt entity summarize Task --input '{\"filters\":[[\"sg_status_list\",\"is\",\"ip\"]],\"summary_fields\":[{\"field\":\"id\",\"type\":\"record_count\"}]}' --output json",
];
const ENTITY_CREATE_EXAMPLES: &[&str] = &[
    "fpt entity create Version --input @payload.json --dry-run",
    "fpt entity create Version --input @payload.json --site ... --auth-mode user-password --username ... --password ...",
];
const ENTITY_UPDATE_EXAMPLES: &[&str] = &[
    "fpt entity update Task 42 --input @patch.json --dry-run",
    "fpt entity update Task 42 --input @patch.json --site ... --auth-mode user-password --username ... --password ...",
];
const ENTITY_DELETE_EXAMPLES: &[&str] = &[
    "fpt entity delete Playlist 99 --dry-run",
    "fpt entity delete Playlist 99 --yes --site ... --auth-mode script --script-name ... --script-key ...",
];
const ENTITY_REVIVE_EXAMPLES: &[&str] = &[
    "fpt entity revive Shot 860 --dry-run",
    "fpt entity revive Shot 860 --site ... --auth-mode script --script-name ... --script-key ...",
];

const ENTITY_BATCH_GET_EXAMPLES: &[&str] = &[
    "fpt entity batch get Shot --input '{\"ids\":[101,102],\"fields\":[\"code\",\"sg_status_list\"]}' --output json",
];
const ENTITY_BATCH_FIND_EXAMPLES: &[&str] =
    &["fpt entity batch find Asset --input @batch_queries.json --output json"];
const ENTITY_BATCH_CREATE_EXAMPLES: &[&str] =
    &["fpt entity batch create Version --input @batch_payloads.json --dry-run --output json"];
const ENTITY_BATCH_UPDATE_EXAMPLES: &[&str] =
    &["fpt entity batch update Task --input @batch_updates.json --dry-run --output json"];
const ENTITY_BATCH_DELETE_EXAMPLES: &[&str] = &[
    "fpt entity batch delete Playlist --input '{\"ids\":[99,100]}' --dry-run --output json",
    "fpt entity batch delete Playlist --input '{\"ids\":[99,100]}' --yes --output json",
];

const ENTITY_NOTES: &[&str] = &[
    "create/update accepts raw REST JSON request bodies",
    "entity find supports `--filter-dsl` and automatically switches to `_search` for complex filters",
    "entity find also accepts a structured `search` object and `additional_filter_presets` for native `_search` payloads",
    "entity summarize uses the ShotGrid RPC `summarize` method and expects explicit `filters` + `summary_fields` input",
    "delete is blocked by default and requires explicit `--yes`",
    "revive uses the ShotGrid RPC `revive` method and supports `--dry-run`",
    "All entity commands reuse the same auth configuration",
];

const ENTITY_BATCH_NOTES: &[&str] = &[
    "Batch commands are client-side orchestration over existing REST CRUD endpoints",
    "Each item returns its own ok/error envelope in `results`",
    "Batch create/update/delete supports `--dry-run`; batch delete still requires explicit `--yes`",
    "Batch execution reuses the same auth configuration and benefits from in-process token reuse",
    "Batch requests run with controlled concurrency; use `FPT_BATCH_CONCURRENCY` to tune throughput when needed",
];

pub const ENTITY_GET_SPEC: CommandSpec = CommandSpec {
    name: "entity.get",
    summary: "Read a single entity record",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "entity + id",
    output: "json",
    examples: ENTITY_GET_EXAMPLES,
    notes: ENTITY_NOTES,
};

pub const ENTITY_FIND_SPEC: CommandSpec = CommandSpec {
    name: "entity.find",
    summary: "Query a collection of entity records",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "query JSON / search JSON / --filter-dsl",
    output: "json",
    examples: ENTITY_FIND_EXAMPLES,
    notes: ENTITY_NOTES,
};

pub const ENTITY_FIND_ONE_SPEC: CommandSpec = CommandSpec {
    name: "entity.find-one",
    summary: "Query a collection and return the first matching record or null",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "same as entity.find",
    output: "json entity / null",
    examples: ENTITY_FIND_ONE_EXAMPLES,
    notes: ENTITY_NOTES,
};

pub const ENTITY_SUMMARIZE_SPEC: CommandSpec = CommandSpec {
    name: "entity.summarize",
    summary: "Summarize matching entity records with ShotGrid summary operators",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rpc",
    fallback_transport: None,
    input: "filters + summary_fields JSON",
    output: "json summary payload",
    examples: ENTITY_SUMMARIZE_EXAMPLES,
    notes: ENTITY_NOTES,
};

pub const ENTITY_CREATE_SPEC: CommandSpec = CommandSpec {
    name: "entity.create",
    summary: "Create an entity record",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: true,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "raw REST request body",
    output: "json",
    examples: ENTITY_CREATE_EXAMPLES,
    notes: ENTITY_NOTES,
};

pub const ENTITY_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "entity.update",
    summary: "Update an entity record",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: true,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "raw REST request body",
    output: "json",
    examples: ENTITY_UPDATE_EXAMPLES,
    notes: ENTITY_NOTES,
};

pub const ENTITY_DELETE_SPEC: CommandSpec = CommandSpec {
    name: "entity.delete",
    summary: "Delete an entity record",
    risk: RiskLevel::Destructive,
    implemented: true,
    supports_dry_run: true,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "entity + id",
    output: "json",
    examples: ENTITY_DELETE_EXAMPLES,
    notes: ENTITY_NOTES,
};

pub const ENTITY_REVIVE_SPEC: CommandSpec = CommandSpec {
    name: "entity.revive",
    summary: "Revive a previously retired entity record",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: true,
    preferred_transport: "rpc",
    fallback_transport: None,
    input: "entity + id",
    output: "json bool",
    examples: ENTITY_REVIVE_EXAMPLES,
    notes: ENTITY_NOTES,
};

pub const ENTITY_BATCH_GET_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.get",
    summary: "Read multiple entity records by id",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "ids array / object with ids + optional fields",
    output: "json",
    examples: ENTITY_BATCH_GET_EXAMPLES,
    notes: ENTITY_BATCH_NOTES,
};

pub const ENTITY_BATCH_FIND_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.find",
    summary: "Run multiple entity queries in one CLI invocation",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "request object array / object with requests",
    output: "json",
    examples: ENTITY_BATCH_FIND_EXAMPLES,
    notes: ENTITY_BATCH_NOTES,
};

pub const ENTITY_BATCH_CREATE_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.create",
    summary: "Create multiple entity records",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: true,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "request body array / object with items",
    output: "json",
    examples: ENTITY_BATCH_CREATE_EXAMPLES,
    notes: ENTITY_BATCH_NOTES,
};

pub const ENTITY_BATCH_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.update",
    summary: "Update multiple entity records",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: true,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "[{id, body}] / object with items",
    output: "json",
    examples: ENTITY_BATCH_UPDATE_EXAMPLES,
    notes: ENTITY_BATCH_NOTES,
};

pub const ENTITY_BATCH_DELETE_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.delete",
    summary: "Delete multiple entity records",
    risk: RiskLevel::Destructive,
    implemented: true,
    supports_dry_run: true,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "ids array / object with ids",
    output: "json",
    examples: ENTITY_BATCH_DELETE_EXAMPLES,
    notes: ENTITY_BATCH_NOTES,
};

const ENTITY_TEXT_SEARCH_EXAMPLES: &[&str] = &[
    "fpt entity text-search --input '{\"text\":\"hero shot\"}' --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt entity text-search --input '{\"text\":\"explosion\",\"entity_types\":{\"Shot\":{},\"Asset\":{}}}' --site ...",
];

const ENTITY_TEXT_SEARCH_NOTES: &[&str] = &[
    "Performs a cross-entity full-text search via POST /api/{ver}/entity/_text_search",
    "Input JSON must include a `text` string; optional `entity_types` restricts the search scope",
    "Returns matching records across multiple entity types",
];

pub const ENTITY_TEXT_SEARCH_SPEC: CommandSpec = CommandSpec {
    name: "entity.text-search",
    summary: "Search across entity types using full-text search",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "input JSON with text (search query) and optional entity_types",
    output: "json",
    examples: ENTITY_TEXT_SEARCH_EXAMPLES,
    notes: ENTITY_TEXT_SEARCH_NOTES,
};

const ENTITY_BATCH_REVIVE_EXAMPLES: &[&str] = &[
    "fpt entity batch revive Shot --input '{\"ids\":[860,861]}' --dry-run --output json",
    "fpt entity batch revive Shot --input '{\"ids\":[860,861]}' --site ... --auth-mode script --script-name ... --script-key ...",
];

pub const ENTITY_BATCH_REVIVE_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.revive",
    summary: "Revive multiple previously retired entity records",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: true,
    preferred_transport: "rpc",
    fallback_transport: None,
    input: "ids array / object with ids",
    output: "json",
    examples: ENTITY_BATCH_REVIVE_EXAMPLES,
    notes: ENTITY_BATCH_NOTES,
};

const ENTITY_RELATIONSHIP_EXAMPLES: &[&str] = &[
    "fpt entity relationship Shot 123 --field assets --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt entity relationship Task 42 --field entity --input '{\"fields\":\"code,id\"}' --site ...",
];

const ENTITY_RELATIONSHIP_NOTES: &[&str] = &[
    "Reads related entities for a specific field via GET /api/{ver}/entity/{type}/{id}/relationships/{field}",
    "Optional --input accepts query parameters like fields, page, and sort",
];

pub const ENTITY_RELATIONSHIP_SPEC: CommandSpec = CommandSpec {
    name: "entity.relationship",
    summary: "Read relationships for a specific entity field",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + field name + optional query params",
    output: "json",
    examples: ENTITY_RELATIONSHIP_EXAMPLES,
    notes: ENTITY_RELATIONSHIP_NOTES,
};

const ENTITY_RELATIONSHIP_CREATE_EXAMPLES: &[&str] = &[
    "fpt entity relationship-create Shot 123 --field assets --input '{\"data\":[{\"type\":\"Asset\",\"id\":7}]}' --site ...",
];

const ENTITY_RELATIONSHIP_CREATE_NOTES: &[&str] = &[
    "Adds links to a multi-entity relationship field via POST /api/{ver}/entity/{type}/{id}/relationships/{field}",
    "Input must be a JSON object containing `data` (array of entity link objects with `type` and `id`)",
];

pub const ENTITY_RELATIONSHIP_CREATE_SPEC: CommandSpec = CommandSpec {
    name: "entity.relationship-create",
    summary: "Add links to a multi-entity relationship field",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + field name + JSON body with data array",
    output: "json",
    examples: ENTITY_RELATIONSHIP_CREATE_EXAMPLES,
    notes: ENTITY_RELATIONSHIP_CREATE_NOTES,
};

const ENTITY_RELATIONSHIP_UPDATE_EXAMPLES: &[&str] = &[
    "fpt entity relationship-update Shot 123 --field assets --input '{\"data\":[{\"type\":\"Asset\",\"id\":7}]}' --site ...",
];

const ENTITY_RELATIONSHIP_UPDATE_NOTES: &[&str] = &[
    "Replaces all links in a multi-entity relationship field via PUT /api/{ver}/entity/{type}/{id}/relationships/{field}",
    "Input must be a JSON object containing `data` (array of entity link objects with `type` and `id`)",
];

pub const ENTITY_RELATIONSHIP_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "entity.relationship-update",
    summary: "Replace links in a multi-entity relationship field",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + field name + JSON body with data array",
    output: "json",
    examples: ENTITY_RELATIONSHIP_UPDATE_EXAMPLES,
    notes: ENTITY_RELATIONSHIP_UPDATE_NOTES,
};

const ENTITY_RELATIONSHIP_DELETE_EXAMPLES: &[&str] = &[
    "fpt entity relationship-delete Shot 123 --field assets --input '{\"data\":[{\"type\":\"Asset\",\"id\":7}]}' --site ...",
];

const ENTITY_RELATIONSHIP_DELETE_NOTES: &[&str] = &[
    "Removes links from a multi-entity relationship field via DELETE /api/{ver}/entity/{type}/{id}/relationships/{field}",
    "Input must be a JSON object containing `data` (array of entity link objects with `type` and `id` to remove)",
];

pub const ENTITY_RELATIONSHIP_DELETE_SPEC: CommandSpec = CommandSpec {
    name: "entity.relationship-delete",
    summary: "Remove links from a multi-entity relationship field",
    risk: RiskLevel::Destructive,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + field name + JSON body with data array",
    output: "json",
    examples: ENTITY_RELATIONSHIP_DELETE_EXAMPLES,
    notes: ENTITY_RELATIONSHIP_DELETE_NOTES,
};

const ENTITY_SHARE_EXAMPLES: &[&str] = &[
    "fpt entity share Shot 123 --input '{\"entities\":[{\"type\":\"Project\",\"id\":85}]}' --site ...",
];

const ENTITY_SHARE_NOTES: &[&str] = &[
    "Shares an entity record to other projects via POST /api/{ver}/entity/{type}/{id}/_share",
    "Input must be a JSON object describing the share target",
];

pub const ENTITY_SHARE_SPEC: CommandSpec = CommandSpec {
    name: "entity.share",
    summary: "Share an entity record to other projects",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + JSON body describing share target",
    output: "json",
    examples: ENTITY_SHARE_EXAMPLES,
    notes: ENTITY_SHARE_NOTES,
};

const ENTITY_UPDATE_LAST_ACCESSED_EXAMPLES: &[&str] = &[
    "fpt entity update-last-accessed 123 --site ... --auth-mode script --script-name ... --script-key ...",
];

const ENTITY_UPDATE_LAST_ACCESSED_NOTES: &[&str] = &[
    "Updates the last-accessed timestamp for a project via PUT /api/{ver}/entity/projects/{id}/_update_last_accessed",
    "Useful for marking a project as recently used in the ShotGrid UI",
];

pub const ENTITY_UPDATE_LAST_ACCESSED_SPEC: CommandSpec = CommandSpec {
    name: "entity.update-last-accessed",
    summary: "Update the last-accessed timestamp for a project",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "project id",
    output: "json",
    examples: ENTITY_UPDATE_LAST_ACCESSED_EXAMPLES,
    notes: ENTITY_UPDATE_LAST_ACCESSED_NOTES,
};

const ENTITY_BATCH_FIND_ONE_EXAMPLES: &[&str] = &[
    "fpt entity batch find-one Shot --input @batch_queries.json --output json",
    "fpt entity batch find-one Asset --input '[{\"fields\":[\"code\"],\"filters\":[[\"sg_status_list\",\"is\",\"ip\"]]}]' --output json",
];

pub const ENTITY_BATCH_FIND_ONE_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.find-one",
    summary: "Run multiple find-one queries in one CLI invocation",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "request object array / object with requests",
    output: "json",
    examples: ENTITY_BATCH_FIND_ONE_EXAMPLES,
    notes: ENTITY_BATCH_NOTES,
};

const ENTITY_BATCH_UPSERT_EXAMPLES: &[&str] = &[
    "fpt entity batch upsert Shot --input @items.json --key code --on-conflict skip --dry-run --output json",
    "fpt entity batch upsert Shot --input @items.json --key code --on-conflict update --checkpoint upsert.jsonl --output json",
];

pub const ENTITY_BATCH_UPSERT_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.upsert",
    summary: "Create or update entities based on a key field (idempotent bulk upsert)",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: true,
    preferred_transport: "rest",
    fallback_transport: Some("rpc"),
    input: "item body array + --key field + --on-conflict strategy",
    output: "json",
    examples: ENTITY_BATCH_UPSERT_EXAMPLES,
    notes: ENTITY_BATCH_NOTES,
};

const ENTITY_COUNT_EXAMPLES: &[&str] = &[
    "fpt entity count Shot --output json",
    "fpt entity count Task --filter-dsl 'sg_status_list == \"ip\"' --output json",
    "fpt entity count Version --input '{\"filters\":[[\"sg_status_list\",\"is\",\"ip\"]]}' --output json",
];

const ENTITY_COUNT_NOTES: &[&str] = &[
    "Convenience wrapper around entity.summarize with record_count on the id field",
    "Accepts optional --input (JSON filters) or --filter-dsl (DSL expression)",
    "Returns the RPC summarize response with record_count",
];

pub const ENTITY_COUNT_SPEC: CommandSpec = CommandSpec {
    name: "entity.count",
    summary: "Count entity records matching optional filters",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rpc",
    fallback_transport: None,
    input: "optional filters JSON or --filter-dsl",
    output: "json summary payload",
    examples: ENTITY_COUNT_EXAMPLES,
    notes: ENTITY_COUNT_NOTES,
};

const ENTITY_BATCH_SUMMARIZE_EXAMPLES: &[&str] =
    &["fpt entity batch summarize --input @batch_summaries.json --output json"];

const ENTITY_BATCH_SUMMARIZE_NOTES: &[&str] = &[
    "Runs multiple summarize queries concurrently in one CLI invocation",
    "Each request must contain `entity` (string) and `payload` (summarize body object)",
    "Results are aggregated with per-item ok/error envelopes",
];

pub const ENTITY_BATCH_SUMMARIZE_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.summarize",
    summary: "Run multiple summarize queries in one CLI invocation",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rpc",
    fallback_transport: None,
    input: "request object array / object with requests",
    output: "json",
    examples: ENTITY_BATCH_SUMMARIZE_EXAMPLES,
    notes: ENTITY_BATCH_SUMMARIZE_NOTES,
};

const ENTITY_BATCH_COUNT_EXAMPLES: &[&str] = &[
    "fpt entity batch count --input '[\"Shot\",\"Asset\",\"Task\"]' --output json",
    "fpt entity batch count --input '{\"entities\":[\"Shot\",\"Asset\"]}' --output json",
    "fpt entity batch count --input '[{\"entity\":\"Task\",\"filters\":[[\"sg_status_list\",\"is\",\"ip\"]]}]' --output json",
];

const ENTITY_BATCH_COUNT_NOTES: &[&str] = &[
    "Counts records for multiple entity types in one CLI invocation using the RPC summarize method",
    "Input can be a flat array of entity name strings, or objects with `entity` + optional `filters` + `filter_operator`",
    "Also accepts an object with `entities` or `requests` key wrapping the array",
    "Each result contains the entity name, summarize response, and per-item ok/error envelope",
    "Use this instead of calling entity.count once per entity type — reduces round-trips and token overhead",
];

pub const ENTITY_BATCH_COUNT_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch.count",
    summary: "Count records for multiple entity types in one CLI invocation",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rpc",
    fallback_transport: None,
    input: "entity name array / object with entities — each item optionally includes filters",
    output: "json",
    examples: ENTITY_BATCH_COUNT_EXAMPLES,
    notes: ENTITY_BATCH_COUNT_NOTES,
};

const ENTITY_SUMMARIZE_REST_EXAMPLES: &[&str] = &[
    "fpt entity summarize-rest Shot --input '{\"summary_fields\":[{\"field\":\"id\",\"type\":\"count\"}],\"filters\":[[\"sg_status_list\",\"is\",\"ip\"]]}' --output json",
    "fpt entity summarize-rest Task --input @summarize_rest.json --site ... --auth-mode script --script-name ... --script-key ...",
];

const ENTITY_SUMMARIZE_REST_NOTES: &[&str] = &[
    "Uses the REST _summarize endpoint: POST /api/{ver}/entity/{entity}/_summarize",
    "This is the REST-native alternative to the legacy RPC summarize method",
    "The request body is forwarded directly; the server validates the payload structure",
    "Input must be a JSON object with summary_fields, filters, and optional grouping",
];

pub const ENTITY_SUMMARIZE_REST_SPEC: CommandSpec = CommandSpec {
    name: "entity.summarize-rest",
    summary: "Summarize entity records using the REST _summarize endpoint",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "JSON object with summary_fields, filters, and optional grouping",
    output: "json summary payload",
    examples: ENTITY_SUMMARIZE_REST_EXAMPLES,
    notes: ENTITY_SUMMARIZE_REST_NOTES,
};

const ENTITY_BATCH_SERVER_EXAMPLES: &[&str] = &[
    "fpt entity batch-server --input '{\"requests\":[{\"request_type\":\"create\",\"entity\":\"Shot\",\"data\":{\"code\":\"shot_010\",\"project\":{\"type\":\"Project\",\"id\":1}}}]}' --output json",
    "fpt entity batch-server --input @batch_ops.json --site ... --auth-mode script --script-name ... --script-key ...",
];

const ENTITY_BATCH_SERVER_NOTES: &[&str] = &[
    "Executes a server-side transactional batch via POST /api/{ver}/entity/_batch",
    "All operations in the batch are executed atomically: all succeed or all roll back",
    "Unlike client-side batch commands, this uses the official ShotGrid server-side batch endpoint",
    "Input must be a JSON object containing `requests` (array of operation descriptors)",
    "Each request descriptor requires `request_type` (create/update/delete), `entity`, and request-specific fields",
];

pub const ENTITY_BATCH_SERVER_SPEC: CommandSpec = CommandSpec {
    name: "entity.batch-server",
    summary: "Execute a server-side transactional batch of entity operations",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "JSON object with requests array of operation descriptors",
    output: "json",
    examples: ENTITY_BATCH_SERVER_EXAMPLES,
    notes: ENTITY_BATCH_SERVER_NOTES,
};
