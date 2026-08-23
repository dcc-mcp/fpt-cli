use clap::{Args, Parser, Subcommand, ValueEnum};
use fpt_domain::app::batch::OnConflict;

use super::common::{AuthModeArg, OutputFormatArg};
use super::connection::ConnectionArgs;

#[derive(Debug, Parser)]
#[command(
    name = "fpt",
    version,
    about = "Flow Production Tracking CLI for OpenClaw"
)]
pub struct Cli {
    #[command(flatten)]
    pub connection: ConnectionArgs,

    #[arg(long, value_enum, global = true, default_value = "json")]
    pub output: OutputFormatArg,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Capabilities,
    #[command(subcommand)]
    Inspect(InspectCommands),
    #[command(subcommand)]
    Auth(AuthCommands),
    #[command(subcommand)]
    Server(ServerCommands),
    #[command(subcommand)]
    Schema(SchemaCommands),
    #[command(subcommand)]
    Entity(EntityCommands),
    #[command(subcommand, name = "work-schedule")]
    WorkSchedule(WorkScheduleCommands),
    #[command(subcommand)]
    Upload(UploadCommands),
    #[command(subcommand)]
    Download(DownloadCommands),
    #[command(subcommand)]
    Thumbnail(ThumbnailCommands),
    #[command(subcommand)]
    Activity(ActivityCommands),
    #[command(subcommand, name = "event-log")]
    EventLog(EventLogCommands),
    #[command(subcommand)]
    Preferences(PreferencesCommands),
    #[command(subcommand)]
    Followers(FollowersCommands),
    #[command(subcommand)]
    Note(NoteCommands),
    #[command(subcommand)]
    User(UserCommands),
    #[command(subcommand)]
    Filmstrip(FilmstripCommands),
    #[command(subcommand)]
    Hierarchy(HierarchyCommands),
    #[command(subcommand)]
    Schedule(ScheduleCommands),
    #[command(subcommand)]
    License(LicenseCommands),
    #[command(subcommand, name = "self")]
    SelfCommand(SelfCommands),
    #[command(subcommand)]
    Config(ConfigCommands),
    #[command(
        name = "self-update",
        about = "Compatibility alias for `fpt self update`",
        hide = true
    )]
    SelfUpdate(SelfUpdateArgs),
}

#[derive(Debug, Subcommand)]
pub enum SelfCommands {
    #[command(
        name = "update",
        about = "Check or install the released fpt binary for the current platform"
    )]
    Update(SelfUpdateArgs),
}

#[derive(Debug, Args, Clone)]
pub struct SelfUpdateArgs {
    #[arg(long, help = "Only check whether a newer release is available")]
    pub check: bool,

    #[arg(
        long,
        help = "Install a specific release version instead of the latest release"
    )]
    pub version: Option<String>,

    #[arg(long, help = "Override the GitHub repository in owner/repo format")]
    pub repository: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    #[command(name = "get", about = "Show the effective persisted CLI configuration")]
    Get,
    #[command(name = "path", about = "Show the config file path")]
    Path,
    #[command(
        name = "set",
        about = "Persist CLI configuration values for later reuse"
    )]
    Set(ConfigSetArgs),
    #[command(name = "clear", about = "Remove persisted CLI configuration values")]
    Clear(ConfigClearArgs),
}

#[derive(Debug, Args, Clone, Default)]
pub struct ConfigSetArgs {
    #[arg(long)]
    pub site: Option<String>,

    #[arg(long = "auth-mode", value_enum)]
    pub auth_mode: Option<AuthModeArg>,

    #[arg(long = "script-name")]
    pub script_name: Option<String>,

    #[arg(long = "script-key")]
    pub script_key: Option<String>,

    #[arg(long)]
    pub username: Option<String>,

    #[arg(long)]
    pub password: Option<String>,

    #[arg(long = "auth-token")]
    pub auth_token: Option<String>,

    #[arg(long = "session-token")]
    pub session_token: Option<String>,

    #[arg(long = "api-version")]
    pub api_version: Option<String>,
}

#[derive(Debug, Args, Clone, Default)]
pub struct ConfigClearArgs {
    #[arg(long, help = "Clear all persisted configuration fields")]
    pub all: bool,

    /// Comma-separated list of field names to clear.
    ///
    /// Valid names: site, auth-mode, script-name, script-key,
    /// username, password, auth-token, session-token, api-version
    #[arg(
        long,
        value_delimiter = ',',
        help = "Comma-separated list of fields to clear (e.g. --fields site,auth-mode)"
    )]
    pub fields: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum InspectCommands {
    Command {
        name: String,
    },
    #[command(name = "list", about = "List all available command names")]
    List,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    Test,
    #[command(
        name = "login",
        about = "Create or replace a local credential profile in the system credential store"
    )]
    Login(AuthLoginArgs),
    #[command(
        name = "logout",
        about = "Remove a local credential profile and its secure credentials"
    )]
    Logout,
}

#[derive(Debug, Args, Clone, Default)]
pub struct AuthLoginArgs {
    #[arg(
        long,
        help = "Open the Flow Production Tracking site before prompting for local credentials"
    )]
    pub open_browser: bool,
}

#[derive(Debug, Subcommand)]
pub enum ServerCommands {
    Info,
    #[command(
        name = "version",
        about = "Fetch REST API version and server metadata (no authentication required)"
    )]
    Version,
    #[command(
        name = "openapi-spec",
        about = "Download the OpenAPI v3 specification for the ShotGrid REST API (no authentication required)"
    )]
    OpenapiSpec {
        #[arg(
            long,
            default_value = "json",
            help = "Specification format: json or yaml"
        )]
        format: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SchemaCommands {
    Entities,
    Fields {
        entity: String,
    },
    #[command(
        name = "field-create",
        about = "Create a new custom field on an entity type"
    )]
    FieldCreate {
        entity: String,
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "field-update",
        about = "Update properties of an existing custom field"
    )]
    FieldUpdate {
        entity: String,
        field_name: String,
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "field-delete",
        about = "Delete a custom field from an entity type"
    )]
    FieldDelete {
        entity: String,
        field_name: String,
    },
    #[command(
        name = "field-read",
        about = "Read the schema definition of a single field"
    )]
    FieldRead {
        entity: String,
        field_name: String,
    },
    #[command(
        name = "entity-read",
        about = "Read the full schema definition of an entity type"
    )]
    EntityRead {
        entity: String,
    },
    #[command(
        name = "field-revive",
        about = "Revive a previously deleted custom field"
    )]
    FieldRevive {
        entity: String,
        field_name: String,
    },
    #[command(name = "entity-update", about = "Update properties of an entity type")]
    EntityUpdate {
        entity: String,
        #[arg(long)]
        input: String,
    },
    #[command(name = "entity-delete", about = "Delete (retire) an entity type")]
    EntityDelete {
        entity: String,
    },
    #[command(name = "entity-create", about = "Create a new custom entity type")]
    EntityCreate {
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "entity-revive",
        about = "Revive a previously retired entity type"
    )]
    EntityRevive {
        entity: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum EntityCommands {
    Get {
        entity: String,
        id: u64,
        #[arg(long, value_delimiter = ',')]
        fields: Vec<String>,
    },
    Find {
        entity: String,
        #[arg(long)]
        input: Option<String>,
        #[arg(long = "filter-dsl")]
        filter_dsl: Option<String>,
    },
    #[command(name = "find-one")]
    FindOne {
        entity: String,
        #[arg(long)]
        input: Option<String>,
        #[arg(long = "filter-dsl")]
        filter_dsl: Option<String>,
    },
    Summarize {
        entity: String,
        #[arg(long)]
        input: String,
    },
    Create {
        entity: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    Update {
        entity: String,
        id: u64,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    Delete {
        entity: String,
        id: u64,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    Revive {
        entity: String,
        id: u64,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(
        name = "text-search",
        about = "Search across entity types using full-text search"
    )]
    TextSearch {
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "relationship",
        about = "Read relationships for a specific entity field"
    )]
    Relationship {
        entity: String,
        id: u64,
        #[arg(long, help = "Related field name (e.g. shots, assets)")]
        field: String,
        #[arg(
            long,
            help = "Optional query parameters as JSON (fields, page, sort, etc.)"
        )]
        input: Option<String>,
    },
    #[command(
        name = "relationship-create",
        about = "Add links to a multi-entity relationship field"
    )]
    RelationshipCreate {
        entity: String,
        id: u64,
        #[arg(long, help = "Related field name (e.g. shots, assets)")]
        field: String,
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "relationship-update",
        about = "Replace links in a multi-entity relationship field"
    )]
    RelationshipUpdate {
        entity: String,
        id: u64,
        #[arg(long, help = "Related field name (e.g. shots, assets)")]
        field: String,
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "relationship-delete",
        about = "Remove links from a multi-entity relationship field"
    )]
    RelationshipDelete {
        entity: String,
        id: u64,
        #[arg(long, help = "Related field name (e.g. shots, assets)")]
        field: String,
        #[arg(long)]
        input: String,
    },
    #[command(name = "share", about = "Share an entity record to other projects")]
    Share {
        entity: String,
        id: u64,
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "update-last-accessed",
        about = "Update the last-accessed timestamp for a project"
    )]
    UpdateLastAccessed {
        #[arg(help = "Project record id")]
        project_id: u64,
    },
    #[command(
        name = "count",
        about = "Count entity records matching optional filters"
    )]
    Count {
        entity: String,
        #[arg(long)]
        input: Option<String>,
        #[arg(long = "filter-dsl")]
        filter_dsl: Option<String>,
    },
    #[command(subcommand)]
    Batch(BatchEntityCommands),
}

#[derive(Debug, Subcommand)]
pub enum WorkScheduleCommands {
    Read {
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "update",
        about = "Update the ShotGrid work schedule for a specific date"
    )]
    Update {
        #[arg(long)]
        input: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BatchEntityCommands {
    Get {
        entity: String,
        #[arg(long)]
        input: String,
    },
    Find {
        entity: String,
        #[arg(long)]
        input: String,
    },
    Create {
        entity: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    Update {
        entity: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    Delete {
        entity: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    #[command(
        name = "upsert",
        about = "Create or update entities based on a key field (idempotent bulk upsert)"
    )]
    Upsert {
        entity: String,
        #[arg(long)]
        input: String,
        /// Field name used to look up existing entities (e.g. `code`)
        #[arg(long)]
        key: String,
        /// How to handle a conflict when an entity with the key value already exists
        #[arg(long = "on-conflict", default_value = "skip")]
        on_conflict: OnConflictArg,
        #[arg(long)]
        dry_run: bool,
        /// Path to a JSONL checkpoint file. Each completed item is appended so
        /// interrupted runs can be resumed safely.
        #[arg(long)]
        checkpoint: Option<String>,
        /// Resume a previously interrupted upsert by reading the checkpoint file
        /// and skipping items that were already processed.
        #[arg(long)]
        resume: bool,
    },
    #[command(
        name = "revive",
        about = "Revive multiple previously retired entity records"
    )]
    Revive {
        entity: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(
        name = "find-one",
        about = "Run multiple find-one queries in one CLI invocation"
    )]
    FindOne {
        entity: String,
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "summarize",
        about = "Run multiple summarize queries in one CLI invocation"
    )]
    Summarize {
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "count",
        about = "Count records for multiple entity types in one CLI invocation"
    )]
    Count {
        #[arg(long)]
        input: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum UploadCommands {
    #[command(
        name = "url",
        about = "Get a pre-signed upload URL for an entity field attachment"
    )]
    Url {
        entity: String,
        id: u64,
        field_name: String,
        file_name: String,
        #[arg(long, help = "MIME content type of the file being uploaded")]
        content_type: Option<String>,
        #[arg(long, help = "Request a multipart upload URL for large files")]
        multipart: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum DownloadCommands {
    #[command(
        name = "url",
        about = "Get a pre-signed download URL for an entity field attachment"
    )]
    Url {
        entity: String,
        id: u64,
        field_name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ThumbnailCommands {
    #[command(
        name = "url",
        about = "Get the thumbnail image URL for an entity record"
    )]
    Url { entity: String, id: u64 },
    #[command(
        name = "upload",
        about = "Upload a thumbnail image for an entity record"
    )]
    Upload {
        entity: String,
        id: u64,
        #[arg(long)]
        input: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ActivityCommands {
    #[command(
        name = "stream",
        about = "Fetch the activity stream for a specific entity record"
    )]
    Stream {
        entity: String,
        id: u64,
        #[arg(
            long,
            help = "Optional query parameters as JSON (page, fields, entity_fields, etc.)"
        )]
        input: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum EventLogCommands {
    #[command(name = "entries", about = "Query ShotGrid event log entries")]
    Entries {
        #[arg(
            long,
            help = "Optional query parameters as JSON (fields, sort, page, etc.)"
        )]
        input: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum PreferencesCommands {
    #[command(name = "get", about = "Read site-level ShotGrid preferences")]
    Get,
    #[command(name = "update", about = "Update site-level ShotGrid preferences")]
    Update {
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "custom-entity",
        about = "Enable a custom entity type on the ShotGrid site"
    )]
    CustomEntity {
        #[arg(long)]
        input: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum FollowersCommands {
    #[command(name = "list", about = "List all followers of an entity record")]
    List { entity: String, id: u64 },
    #[command(
        name = "follow",
        about = "Add a user as a follower of an entity record"
    )]
    Follow {
        entity: String,
        id: u64,
        #[arg(
            long,
            help = "User JSON object with type and id, e.g. '{\"type\":\"HumanUser\",\"id\":456}'"
        )]
        input: String,
    },
    #[command(
        name = "unfollow",
        about = "Remove a user from an entity record's followers"
    )]
    Unfollow {
        entity: String,
        id: u64,
        #[arg(
            long,
            help = "User JSON object with type and id, e.g. '{\"type\":\"HumanUser\",\"id\":456}'"
        )]
        input: String,
    },
    #[command(name = "following", about = "List all entities a user is following")]
    Following {
        #[arg(help = "HumanUser record id")]
        user_id: u64,
        #[arg(
            long,
            help = "Optional query parameters as JSON (fields, page, sort, etc.)"
        )]
        input: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum NoteCommands {
    #[command(
        name = "threads",
        about = "List all replies in a top-level Note thread"
    )]
    Threads {
        #[arg(help = "Top-level Note record id")]
        note_id: u64,
        #[arg(
            long,
            help = "Optional query parameters as JSON (fields, page, sort, etc.)"
        )]
        input: Option<String>,
    },
    #[command(
        name = "reply-create",
        about = "Create a reply in a top-level Note thread"
    )]
    ReplyCreate {
        #[arg(help = "Top-level Note record id")]
        note_id: u64,
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "reply-read",
        about = "Read a single reply in a top-level Note thread"
    )]
    ReplyRead {
        #[arg(help = "Top-level Note record id")]
        note_id: u64,
        #[arg(help = "Reply record id")]
        reply_id: u64,
        #[arg(long, help = "Optional query parameters as JSON (fields, etc.)")]
        input: Option<String>,
    },
    #[command(
        name = "reply-update",
        about = "Update an existing reply in a top-level Note thread"
    )]
    ReplyUpdate {
        #[arg(help = "Top-level Note record id")]
        note_id: u64,
        #[arg(help = "Reply record id")]
        reply_id: u64,
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "reply-delete",
        about = "Delete a reply from a top-level Note thread"
    )]
    ReplyDelete {
        #[arg(help = "Top-level Note record id")]
        note_id: u64,
        #[arg(help = "Reply record id")]
        reply_id: u64,
    },
}

#[derive(Debug, Subcommand)]
pub enum HierarchyCommands {
    #[command(
        name = "search",
        about = "Search project hierarchy with a structured JSON query"
    )]
    Search {
        #[arg(long)]
        input: String,
    },
    #[command(name = "expand", about = "Expand a node in the project hierarchy tree")]
    Expand {
        #[arg(long)]
        input: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum UserCommands {
    #[command(name = "current", about = "Get the currently authenticated user")]
    Current {
        #[arg(
            long = "user-type",
            default_value = "human",
            help = "User type: human (HumanUser) or api (ApiUser)"
        )]
        user_type: String,
        #[arg(long, help = "Optional query parameters as JSON (fields, etc.)")]
        input: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum FilmstripCommands {
    #[command(
        name = "url",
        about = "Get the filmstrip thumbnail image URL for an entity record"
    )]
    Url { entity: String, id: u64 },
}

#[derive(Debug, Subcommand)]
pub enum ScheduleCommands {
    #[command(
        name = "work-day-rules",
        about = "Read work day rules from the ShotGrid scheduling system"
    )]
    Read {
        #[arg(long, help = "Optional query parameters as JSON (page, etc.)")]
        input: Option<String>,
    },
    #[command(
        name = "work-day-rules-read",
        about = "Read a single work day rule by record id"
    )]
    ReadOne {
        #[arg(help = "Work day rule record id")]
        rule_id: u64,
    },
    #[command(
        name = "work-day-rules-update",
        about = "Update a specific work day rule by record id"
    )]
    Update {
        #[arg(help = "Work day rule record id")]
        rule_id: u64,
        #[arg(long)]
        input: String,
    },
    #[command(name = "work-day-rules-create", about = "Create a new work day rule")]
    Create {
        #[arg(long)]
        input: String,
    },
    #[command(
        name = "work-day-rules-delete",
        about = "Delete a work day rule by record id"
    )]
    Delete {
        #[arg(help = "Work day rule record id")]
        rule_id: u64,
    },
}

#[derive(Debug, Subcommand)]
pub enum LicenseCommands {
    #[command(name = "get", about = "Read the ShotGrid site license information")]
    Get,
}

/// How to handle a conflict when an entity with the key field value already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OnConflictArg {
    /// Skip the item — do not create or update (default)
    Skip,
    /// Update the existing entity with the new body
    Update,
    /// Return an error for the conflicting item
    Error,
}

impl From<OnConflictArg> for OnConflict {
    fn from(value: OnConflictArg) -> Self {
        match value {
            OnConflictArg::Skip => OnConflict::Skip,
            OnConflictArg::Update => OnConflict::Update,
            OnConflictArg::Error => OnConflict::Error,
        }
    }
}
