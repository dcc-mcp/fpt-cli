mod activity;
mod auth;
mod core;
mod entity;
mod follow;
mod hierarchy;
mod license;
mod note;
mod preferences;
mod schedule;
mod schema;
mod self_update;
mod server;
mod upload;
mod user;
mod work_schedule;

use fpt_core::CommandSpec;

static COMMANDS: &[CommandSpec] = &[
    // Core
    core::CAPABILITIES_SPEC,
    core::INSPECT_COMMAND_SPEC,
    core::INSPECT_LIST_SPEC,
    // Auth
    auth::AUTH_TEST_SPEC,
    auth::AUTH_LOGIN_SPEC,
    auth::AUTH_LOGOUT_SPEC,
    // Server
    server::SERVER_INFO_SPEC,
    server::SERVER_REST_VERSION_SPEC,
    server::SERVER_OPENAPI_SPEC_SPEC,
    // Schema
    schema::SCHEMA_ENTITIES_SPEC,
    schema::SCHEMA_ENTITY_READ_SPEC,
    schema::SCHEMA_ENTITY_UPDATE_SPEC,
    schema::SCHEMA_ENTITY_DELETE_SPEC,
    schema::SCHEMA_ENTITY_CREATE_SPEC,
    schema::SCHEMA_ENTITY_REVIVE_SPEC,
    schema::SCHEMA_FIELDS_SPEC,
    schema::SCHEMA_FIELD_CREATE_SPEC,
    schema::SCHEMA_FIELD_READ_SPEC,
    schema::SCHEMA_FIELD_UPDATE_SPEC,
    schema::SCHEMA_FIELD_DELETE_SPEC,
    schema::SCHEMA_FIELD_REVIVE_SPEC,
    // Entity — single operations
    entity::ENTITY_GET_SPEC,
    entity::ENTITY_FIND_SPEC,
    entity::ENTITY_FIND_ONE_SPEC,
    entity::ENTITY_SUMMARIZE_SPEC,
    entity::ENTITY_COUNT_SPEC,
    entity::ENTITY_CREATE_SPEC,
    entity::ENTITY_UPDATE_SPEC,
    entity::ENTITY_DELETE_SPEC,
    entity::ENTITY_REVIVE_SPEC,
    entity::ENTITY_TEXT_SEARCH_SPEC,
    entity::ENTITY_RELATIONSHIP_SPEC,
    entity::ENTITY_RELATIONSHIP_CREATE_SPEC,
    entity::ENTITY_RELATIONSHIP_UPDATE_SPEC,
    entity::ENTITY_RELATIONSHIP_DELETE_SPEC,
    entity::ENTITY_SHARE_SPEC,
    entity::ENTITY_UPDATE_LAST_ACCESSED_SPEC,
    // Entity — batch operations
    entity::ENTITY_BATCH_GET_SPEC,
    entity::ENTITY_BATCH_FIND_SPEC,
    entity::ENTITY_BATCH_FIND_ONE_SPEC,
    entity::ENTITY_BATCH_CREATE_SPEC,
    entity::ENTITY_BATCH_UPDATE_SPEC,
    entity::ENTITY_BATCH_DELETE_SPEC,
    entity::ENTITY_BATCH_REVIVE_SPEC,
    entity::ENTITY_BATCH_UPSERT_SPEC,
    entity::ENTITY_BATCH_SUMMARIZE_SPEC,
    entity::ENTITY_BATCH_COUNT_SPEC,
    // Follow
    follow::ENTITY_FOLLOWERS_SPEC,
    follow::ENTITY_FOLLOW_SPEC,
    follow::ENTITY_UNFOLLOW_SPEC,
    follow::USER_FOLLOWING_SPEC,
    // Note
    note::NOTE_THREADS_SPEC,
    note::NOTE_REPLY_CREATE_SPEC,
    note::NOTE_REPLY_READ_SPEC,
    note::NOTE_REPLY_UPDATE_SPEC,
    note::NOTE_REPLY_DELETE_SPEC,
    // User
    user::CURRENT_USER_SPEC,
    // Hierarchy
    hierarchy::HIERARCHY_SEARCH_SPEC,
    hierarchy::HIERARCHY_EXPAND_SPEC,
    // Schedule
    schedule::SCHEDULE_WORK_DAY_RULES_SPEC,
    schedule::SCHEDULE_WORK_DAY_RULES_UPDATE_SPEC,
    schedule::SCHEDULE_WORK_DAY_RULES_CREATE_SPEC,
    schedule::SCHEDULE_WORK_DAY_RULES_DELETE_SPEC,
    // License
    license::LICENSE_GET_SPEC,
    // Work schedule
    work_schedule::WORK_SCHEDULE_READ_SPEC,
    work_schedule::WORK_SCHEDULE_UPDATE_SPEC,
    // Upload / download
    upload::UPLOAD_URL_SPEC,
    upload::DOWNLOAD_URL_SPEC,
    upload::THUMBNAIL_URL_SPEC,
    upload::THUMBNAIL_UPLOAD_SPEC,
    upload::FILMSTRIP_URL_SPEC,
    // Activity
    activity::ACTIVITY_STREAM_SPEC,
    activity::EVENT_LOG_ENTRIES_SPEC,
    // Preferences
    preferences::PREFERENCES_GET_SPEC,
    preferences::PREFERENCES_UPDATE_SPEC,
    preferences::PREFERENCES_CUSTOM_ENTITY_SPEC,
    // Self-update & config
    self_update::SELF_UPDATE_SPEC,
    self_update::CONFIG_GET_SPEC,
    self_update::CONFIG_PATH_SPEC,
    self_update::CONFIG_SET_SPEC,
    self_update::CONFIG_CLEAR_SPEC,
];

pub fn command_specs() -> &'static [CommandSpec] {
    COMMANDS
}

pub fn find_command_spec(name: &str) -> Option<&'static CommandSpec> {
    let normalized = normalize_command_name(name);
    COMMANDS.iter().find(|spec| spec.name == normalized)
}

fn normalize_command_name(name: &str) -> String {
    name.trim().replace(' ', ".").to_ascii_lowercase()
}
