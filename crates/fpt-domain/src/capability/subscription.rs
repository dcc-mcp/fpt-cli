use fpt_core::{CommandSpec, RiskLevel};

const SUBSCRIPTION_EXAMPLES: &[&str] = &[
    "fpt subscription user-list --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt subscription user-assign --input '{\"1554\":\"standard\",\"1588\":\"trial\"}' --site ...",
];

const SUBSCRIPTION_NOTES: &[&str] = &[
    "Subscription seat endpoints are served under /api/{ver}/subscription_seat",
    "Reading or assigning subscription seats requires site administrator credentials",
];

const USER_ASSIGN_NOTES: &[&str] = &[
    "Assigns subscriptions via POST /api/{ver}/subscription_seat/user_subscriptions",
    "The body maps user ids to subscription names, e.g. {\"1554\": \"standard\"}",
];

pub const SUBSCRIPTION_USER_LIST_SPEC: CommandSpec = CommandSpec {
    name: "subscription.user-list",
    summary: "Read the subscription assigned to every user on the site",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "authenticated profile",
    output: "json",
    examples: SUBSCRIPTION_EXAMPLES,
    notes: SUBSCRIPTION_NOTES,
};

pub const SUBSCRIPTION_USER_ASSIGN_SPEC: CommandSpec = CommandSpec {
    name: "subscription.user-assign",
    summary: "Assign subscription seats to users",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "JSON object mapping user ids to subscription names",
    output: "json",
    examples: SUBSCRIPTION_EXAMPLES,
    notes: USER_ASSIGN_NOTES,
};
