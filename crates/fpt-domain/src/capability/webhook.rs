use fpt_core::{CommandSpec, RiskLevel};

const WEBHOOK_EXAMPLES: &[&str] = &[
    "fpt webhook hooks-list --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt webhook hook-create --input '{\"event_type\":\"shotgrid.entity.Shot.change\",\"url\":\"https://example.com/hook\"}' --site ...",
];

const WEBHOOK_NOTES: &[&str] = &[
    "Webhook and delivery endpoints are served under /api/{ver}/webhook",
    "Hook and delivery ids are opaque record ids (usually UUIDs), not numeric ids",
    "Managing webhooks requires site administrator credentials",
];

const HOOK_LIST_NOTES: &[&str] = &[
    "Lists webhooks via GET /api/{ver}/webhook/hooks",
    "Optional --input JSON is converted into query parameters (status, url, page)",
];

const HOOK_CREATE_NOTES: &[&str] = &[
    "Creates a webhook via POST /api/{ver}/webhook/hooks",
    "The body must contain at least an event type and a target `url`",
];

const HOOK_UPDATE_NOTES: &[&str] = &[
    "Updates a webhook via PUT /api/{ver}/webhook/hooks/{hook_id}",
    "Only the supplied fields are modified",
];

const HOOK_DELETE_NOTES: &[&str] = &[
    "Deletes a webhook via DELETE /api/{ver}/webhook/hooks/{hook_id}",
    "This is irreversible — the webhook stops receiving deliveries immediately",
];

const HOOK_TEST_NOTES: &[&str] = &[
    "Triggers a test delivery via POST /api/{ver}/webhook/hooks/{hook_id}/test_connection",
    "Use this to validate the target endpoint before relying on the webhook",
];

const DELIVERY_LIST_NOTES: &[&str] = &[
    "Lists deliveries for a hook via GET /api/{ver}/webhook/hooks/{hook_id}/deliveries",
    "Optional --input JSON is converted into query parameters (status, from, to, entity_type, page)",
];

const DELIVERY_UPDATE_NOTES: &[&str] = &[
    "Updates a delivery via PUT /api/{ver}/webhook/deliveries/{delivery_id}",
    "Typically used to acknowledge a failed delivery",
];

const DELIVERY_REDELIVER_NOTES: &[&str] = &[
    "Re-sends a delivery via POST /api/{ver}/webhook/deliveries/{delivery_id}/redeliver",
    "Only failed deliveries can be re-delivered",
];

pub const WEBHOOK_HOOKS_LIST_SPEC: CommandSpec = CommandSpec {
    name: "webhook.hooks-list",
    summary: "List the webhooks registered on the site",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "optional query params JSON (status, url, page)",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: HOOK_LIST_NOTES,
};

pub const WEBHOOK_HOOK_CREATE_SPEC: CommandSpec = CommandSpec {
    name: "webhook.hook-create",
    summary: "Create a new webhook",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "webhook definition JSON",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: HOOK_CREATE_NOTES,
};

pub const WEBHOOK_HOOK_READ_SPEC: CommandSpec = CommandSpec {
    name: "webhook.hook-read",
    summary: "Read a single webhook by record id",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "hook id",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: WEBHOOK_NOTES,
};

pub const WEBHOOK_HOOK_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "webhook.hook-update",
    summary: "Update an existing webhook",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "hook id + webhook properties JSON",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: HOOK_UPDATE_NOTES,
};

pub const WEBHOOK_HOOK_DELETE_SPEC: CommandSpec = CommandSpec {
    name: "webhook.hook-delete",
    summary: "Delete a webhook",
    risk: RiskLevel::Destructive,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "hook id",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: HOOK_DELETE_NOTES,
};

pub const WEBHOOK_HOOK_TEST_SPEC: CommandSpec = CommandSpec {
    name: "webhook.hook-test",
    summary: "Send a test delivery for a webhook",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "hook id",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: HOOK_TEST_NOTES,
};

pub const WEBHOOK_DELIVERIES_LIST_SPEC: CommandSpec = CommandSpec {
    name: "webhook.deliveries-list",
    summary: "List the deliveries recorded for a webhook",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "hook id + optional query params JSON",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: DELIVERY_LIST_NOTES,
};

pub const WEBHOOK_DELIVERY_READ_SPEC: CommandSpec = CommandSpec {
    name: "webhook.delivery-read",
    summary: "Read a single webhook delivery by record id",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "delivery id",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: WEBHOOK_NOTES,
};

pub const WEBHOOK_DELIVERY_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "webhook.delivery-update",
    summary: "Update a webhook delivery",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "delivery id + delivery properties JSON",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: DELIVERY_UPDATE_NOTES,
};

pub const WEBHOOK_DELIVERY_REDELIVER_SPEC: CommandSpec = CommandSpec {
    name: "webhook.delivery-redeliver",
    summary: "Re-deliver a failed webhook delivery",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "delivery id",
    output: "json",
    examples: WEBHOOK_EXAMPLES,
    notes: DELIVERY_REDELIVER_NOTES,
};
