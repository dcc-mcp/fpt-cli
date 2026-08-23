use fpt_core::{CommandSpec, RiskLevel};

const SCHEDULE_WORK_DAY_RULES_EXAMPLES: &[&str] = &[
    "fpt schedule work-day-rules --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt schedule work-day-rules --input '{\"page\":{\"size\":50}}' --site ... --auth-mode script --script-name ... --script-key ...",
];

const SCHEDULE_WORK_DAY_RULES_NOTES: &[&str] = &[
    "Read the site-level or project-level work day rules from ShotGrid",
    "Uses the REST schedule/work_day_rules endpoint",
];

const SCHEDULE_WORK_DAY_RULES_READ_EXAMPLES: &[&str] = &[
    "fpt schedule work-day-rules-read 42 --site ... --auth-mode script --script-name ... --script-key ...",
];

const SCHEDULE_WORK_DAY_RULES_READ_NOTES: &[&str] = &[
    "Read a single work day rule by its record id",
    "Uses the REST GET schedule/work_day_rules/{id} endpoint",
];

const SCHEDULE_WORK_DAY_RULES_UPDATE_EXAMPLES: &[&str] = &[
    "fpt schedule work-day-rules-update 42 --input @rule.json --site ... --auth-mode script --script-name ... --script-key ...",
];

const SCHEDULE_WORK_DAY_RULES_UPDATE_NOTES: &[&str] = &[
    "Update a specific work day rule by its record id",
    "Uses the REST PUT schedule/work_day_rules/{id} endpoint",
];

const SCHEDULE_WORK_DAY_RULES_CREATE_EXAMPLES: &[&str] = &[
    "fpt schedule work-day-rules-create --input @rule.json --site ... --auth-mode script --script-name ... --script-key ...",
];

const SCHEDULE_WORK_DAY_RULES_CREATE_NOTES: &[&str] = &[
    "Create a new work day rule in the ShotGrid scheduling system",
    "Uses the REST POST schedule/work_day_rules endpoint",
    "Input must be a JSON object containing rule fields such as `date`, `description`, and `project`",
];

const SCHEDULE_WORK_DAY_RULES_DELETE_EXAMPLES: &[&str] = &[
    "fpt schedule work-day-rules-delete 42 --site ... --auth-mode script --script-name ... --script-key ...",
];

const SCHEDULE_WORK_DAY_RULES_DELETE_NOTES: &[&str] = &[
    "Delete a work day rule by its record id",
    "Uses the REST DELETE schedule/work_day_rules/{id} endpoint",
    "This operation is destructive and cannot be undone",
];

pub const SCHEDULE_WORK_DAY_RULES_SPEC: CommandSpec = CommandSpec {
    name: "schedule.work-day-rules",
    summary: "Read work day rules from the ShotGrid scheduling system",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "optional query params JSON (page, etc.)",
    output: "json",
    examples: SCHEDULE_WORK_DAY_RULES_EXAMPLES,
    notes: SCHEDULE_WORK_DAY_RULES_NOTES,
};

pub const SCHEDULE_WORK_DAY_RULES_READ_SPEC: CommandSpec = CommandSpec {
    name: "schedule.work-day-rules-read",
    summary: "Read a single work day rule by record id",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "work day rule record id",
    output: "json",
    examples: SCHEDULE_WORK_DAY_RULES_READ_EXAMPLES,
    notes: SCHEDULE_WORK_DAY_RULES_READ_NOTES,
};

pub const SCHEDULE_WORK_DAY_RULES_UPDATE_SPEC: CommandSpec = CommandSpec {
    name: "schedule.work-day-rules-update",
    summary: "Update a specific work day rule in the ShotGrid scheduling system",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "work day rule update body JSON",
    output: "json",
    examples: SCHEDULE_WORK_DAY_RULES_UPDATE_EXAMPLES,
    notes: SCHEDULE_WORK_DAY_RULES_UPDATE_NOTES,
};

pub const SCHEDULE_WORK_DAY_RULES_CREATE_SPEC: CommandSpec = CommandSpec {
    name: "schedule.work-day-rules-create",
    summary: "Create a new work day rule in the ShotGrid scheduling system",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "work day rule body JSON",
    output: "json",
    examples: SCHEDULE_WORK_DAY_RULES_CREATE_EXAMPLES,
    notes: SCHEDULE_WORK_DAY_RULES_CREATE_NOTES,
};

pub const SCHEDULE_WORK_DAY_RULES_DELETE_SPEC: CommandSpec = CommandSpec {
    name: "schedule.work-day-rules-delete",
    summary: "Delete a work day rule from the ShotGrid scheduling system",
    risk: RiskLevel::Destructive,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "work day rule record id",
    output: "json",
    examples: SCHEDULE_WORK_DAY_RULES_DELETE_EXAMPLES,
    notes: SCHEDULE_WORK_DAY_RULES_DELETE_NOTES,
};
