use crate::cli::{
    ActivityCommands, AuthCommands, BatchEntityCommands, Cli, Commands, DownloadCommands,
    EntityCommands, EventLogCommands, FilmstripCommands, FollowersCommands, HierarchyCommands,
    InspectCommands, LicenseCommands, NoteCommands, PreferencesCommands, ScheduleCommands,
    SchemaCommands, SelfCommands, ServerCommands, ThumbnailCommands, UploadCommands, UserCommands,
    WorkScheduleCommands,
};
use crate::config;
use crate::self_update;
use fpt_core::{AppError, Result, read_json_input};
use fpt_domain::App;
use fpt_domain::transport::UploadUrlRequest;
use serde_json::Value;

pub async fn run(cli: Cli) -> Result<Value> {
    let app = App::default();
    let connection = cli.connection.into();

    match cli.command {
        Commands::Capabilities => Ok(app.capabilities(env!("CARGO_PKG_VERSION"))),
        Commands::Inspect(command) => match command {
            InspectCommands::Command { name } => app.inspect_command(&name),
            InspectCommands::List => Ok(app.inspect_list()),
        },
        Commands::Auth(command) => match command {
            AuthCommands::Test => app.auth_test(connection).await,
            AuthCommands::Login(args) => config::login(connection, args.open_browser),
            AuthCommands::Logout => config::logout(connection.profile.as_deref()),
        },
        Commands::Server(command) => match command {
            ServerCommands::Info => app.server_info(connection).await,
            ServerCommands::Version => app.rest_api_version(connection).await,
            ServerCommands::OpenapiSpec { format } => app.openapi_spec(connection, &format).await,
        },
        Commands::Schema(command) => match command {
            SchemaCommands::Entities => app.schema_entities(connection).await,
            SchemaCommands::Fields { entity } => app.schema_fields(connection, &entity).await,
            SchemaCommands::FieldCreate { entity, input } => {
                let body = required_json_input(input)?;
                app.schema_field_create(connection, &entity, body).await
            }
            SchemaCommands::FieldUpdate {
                entity,
                field_name,
                input,
            } => {
                let body = required_json_input(input)?;
                app.schema_field_update(connection, &entity, &field_name, body)
                    .await
            }
            SchemaCommands::FieldDelete { entity, field_name } => {
                app.schema_field_delete(connection, &entity, &field_name)
                    .await
            }
            SchemaCommands::FieldRead { entity, field_name } => {
                app.schema_field_read(connection, &entity, &field_name)
                    .await
            }
            SchemaCommands::EntityRead { entity } => {
                app.schema_entity_read(connection, &entity).await
            }
            SchemaCommands::FieldRevive { entity, field_name } => {
                app.schema_field_revive(connection, &entity, &field_name)
                    .await
            }
            SchemaCommands::EntityUpdate { entity, input } => {
                let body = required_json_input(input)?;
                app.schema_entity_update(connection, &entity, body).await
            }
            SchemaCommands::EntityDelete { entity } => {
                app.schema_entity_delete(connection, &entity).await
            }
            SchemaCommands::EntityCreate { input } => {
                let body = required_json_input(input)?;
                app.schema_entity_create(connection, body).await
            }
            SchemaCommands::EntityRevive { entity } => {
                app.schema_entity_revive(connection, &entity).await
            }
        },
        Commands::Entity(command) => match command {
            EntityCommands::Get { entity, id, fields } => {
                let fields = (!fields.is_empty()).then_some(fields);
                app.entity_get(connection, &entity, id, fields).await
            }
            EntityCommands::Find {
                entity,
                input,
                filter_dsl,
            } => {
                let input = read_json_input(input.as_deref())?;
                app.entity_find(connection, &entity, input, filter_dsl)
                    .await
            }
            EntityCommands::FindOne {
                entity,
                input,
                filter_dsl,
            } => {
                let input = read_json_input(input.as_deref())?;
                app.entity_find_one(connection, &entity, input, filter_dsl)
                    .await
            }
            EntityCommands::Summarize { entity, input } => {
                let body = required_json_input(input)?;
                app.entity_summarize(connection, &entity, body).await
            }
            EntityCommands::Create {
                entity,
                input,
                dry_run,
            } => {
                let body = required_json_input(input)?;
                app.entity_create(connection, &entity, body, dry_run).await
            }
            EntityCommands::Update {
                entity,
                id,
                input,
                dry_run,
            } => {
                let body = required_json_input(input)?;
                app.entity_update(connection, &entity, id, body, dry_run)
                    .await
            }
            EntityCommands::Delete {
                entity,
                id,
                dry_run,
                yes,
            } => {
                app.entity_delete(connection, &entity, id, dry_run, yes)
                    .await
            }
            EntityCommands::Revive {
                entity,
                id,
                dry_run,
            } => app.entity_revive(connection, &entity, id, dry_run).await,
            EntityCommands::TextSearch { input } => {
                let body = required_json_input(input)?;
                app.text_search(connection, body).await
            }
            EntityCommands::Relationship {
                entity,
                id,
                field,
                input,
            } => {
                let input = read_json_input(input.as_deref())?;
                app.entity_relationships(connection, &entity, id, &field, input)
                    .await
            }
            EntityCommands::RelationshipCreate {
                entity,
                id,
                field,
                input,
            } => {
                let body = required_json_input(input)?;
                app.entity_relationship_create(connection, &entity, id, &field, body)
                    .await
            }
            EntityCommands::RelationshipUpdate {
                entity,
                id,
                field,
                input,
            } => {
                let body = required_json_input(input)?;
                app.entity_relationship_update(connection, &entity, id, &field, body)
                    .await
            }
            EntityCommands::RelationshipDelete {
                entity,
                id,
                field,
                input,
            } => {
                let body = required_json_input(input)?;
                app.entity_relationship_delete(connection, &entity, id, &field, body)
                    .await
            }
            EntityCommands::Share { entity, id, input } => {
                let body = required_json_input(input)?;
                app.entity_share(connection, &entity, id, body).await
            }
            EntityCommands::UpdateLastAccessed { project_id } => {
                app.project_update_last_accessed(connection, project_id)
                    .await
            }
            EntityCommands::Count {
                entity,
                input,
                filter_dsl,
            } => {
                let input = read_json_input(input.as_deref())?;
                app.entity_count(connection, &entity, input, filter_dsl)
                    .await
            }
            EntityCommands::Batch(command) => match command {
                BatchEntityCommands::Get { entity, input } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_get(connection, &entity, body).await
                }
                BatchEntityCommands::Find { entity, input } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_find(connection, &entity, body).await
                }
                BatchEntityCommands::Create {
                    entity,
                    input,
                    dry_run,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_create(connection, &entity, body, dry_run)
                        .await
                }
                BatchEntityCommands::Update {
                    entity,
                    input,
                    dry_run,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_update(connection, &entity, body, dry_run)
                        .await
                }
                BatchEntityCommands::Delete {
                    entity,
                    input,
                    dry_run,
                    yes,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_delete(connection, &entity, body, dry_run, yes)
                        .await
                }
                BatchEntityCommands::Upsert {
                    entity,
                    input,
                    key,
                    on_conflict,
                    dry_run,
                    checkpoint,
                    resume,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_upsert(
                        connection,
                        &entity,
                        body,
                        &key,
                        on_conflict.into(),
                        dry_run,
                        checkpoint,
                        resume,
                    )
                    .await
                }
                BatchEntityCommands::Revive {
                    entity,
                    input,
                    dry_run,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_revive(connection, &entity, body, dry_run)
                        .await
                }
                BatchEntityCommands::FindOne { entity, input } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_find_one(connection, &entity, body).await
                }
                BatchEntityCommands::Summarize { input } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_summarize(connection, body).await
                }
                BatchEntityCommands::Count { input } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_count(connection, body).await
                }
            },
        },
        Commands::WorkSchedule(command) => match command {
            WorkScheduleCommands::Read { input } => {
                let body = required_json_input(input)?;
                app.work_schedule_read(connection, body).await
            }
            WorkScheduleCommands::Update { input } => {
                let body = required_json_input(input)?;
                app.work_schedule_update(connection, body).await
            }
        },
        Commands::Upload(command) => match command {
            UploadCommands::Url {
                entity,
                id,
                field_name,
                file_name,
                content_type,
                multipart,
            } => {
                app.upload_url(
                    connection,
                    UploadUrlRequest {
                        entity: &entity,
                        id,
                        field_name: &field_name,
                        file_name: &file_name,
                        content_type: content_type.as_deref(),
                        multipart_upload: multipart,
                    },
                )
                .await
            }
        },
        Commands::Download(command) => match command {
            DownloadCommands::Url {
                entity,
                id,
                field_name,
            } => app.download_url(connection, &entity, id, &field_name).await,
        },
        Commands::Thumbnail(command) => match command {
            ThumbnailCommands::Url { entity, id } => {
                app.thumbnail_url(connection, &entity, id).await
            }
            ThumbnailCommands::Upload { entity, id, input } => {
                let body = required_json_input(input)?;
                app.thumbnail_upload(connection, &entity, id, body).await
            }
        },
        Commands::Activity(command) => match command {
            ActivityCommands::Stream { entity, id, input } => {
                let input = read_json_input(input.as_deref())?;
                app.activity_stream(connection, &entity, id, input).await
            }
        },
        Commands::EventLog(command) => match command {
            EventLogCommands::Entries { input } => {
                let input = read_json_input(input.as_deref())?;
                app.event_log_entries(connection, input).await
            }
        },
        Commands::Preferences(command) => match command {
            PreferencesCommands::Get => app.preferences_get(connection).await,
            PreferencesCommands::Update { input } => {
                let body = required_json_input(input)?;
                app.preferences_update(connection, body).await
            }
            PreferencesCommands::CustomEntity { input } => {
                let body = required_json_input(input)?;
                app.preferences_custom_entity(connection, body).await
            }
        },
        Commands::Followers(command) => match command {
            FollowersCommands::List { entity, id } => {
                app.entity_followers(connection, &entity, id).await
            }
            FollowersCommands::Follow { entity, id, input } => {
                let body = required_json_input(input)?;
                app.entity_follow(connection, &entity, id, body).await
            }
            FollowersCommands::Unfollow { entity, id, input } => {
                let body = required_json_input(input)?;
                app.entity_unfollow(connection, &entity, id, body).await
            }
            FollowersCommands::Following { user_id, input } => {
                let input = read_json_input(input.as_deref())?;
                app.user_following(connection, user_id, input).await
            }
        },
        Commands::Note(command) => match command {
            NoteCommands::Threads { note_id, input } => {
                let input = read_json_input(input.as_deref())?;
                app.note_threads(connection, note_id, input).await
            }
            NoteCommands::ReplyCreate { note_id, input } => {
                let body = required_json_input(input)?;
                app.note_reply_create(connection, note_id, body).await
            }
            NoteCommands::ReplyRead {
                note_id,
                reply_id,
                input,
            } => {
                let input = read_json_input(input.as_deref())?;
                app.note_reply_read(connection, note_id, reply_id, input)
                    .await
            }
            NoteCommands::ReplyUpdate {
                note_id,
                reply_id,
                input,
            } => {
                let body = required_json_input(input)?;
                app.note_reply_update(connection, note_id, reply_id, body)
                    .await
            }
            NoteCommands::ReplyDelete { note_id, reply_id } => {
                app.note_reply_delete(connection, note_id, reply_id).await
            }
        },
        Commands::User(command) => match command {
            UserCommands::Current { user_type, input } => {
                let input = read_json_input(input.as_deref())?;
                app.current_user(connection, &user_type, input).await
            }
        },
        Commands::Filmstrip(command) => match command {
            FilmstripCommands::Url { entity, id } => {
                app.filmstrip_thumbnail(connection, &entity, id).await
            }
        },
        Commands::Hierarchy(command) => match command {
            HierarchyCommands::Search { input } => {
                let body = required_json_input(input)?;
                app.hierarchy_search(connection, body).await
            }
            HierarchyCommands::Expand { input } => {
                let body = required_json_input(input)?;
                app.hierarchy_expand(connection, body).await
            }
        },
        Commands::Schedule(command) => match command {
            ScheduleCommands::Read { input } => {
                let input = read_json_input(input.as_deref())?;
                app.schedule_work_day_rules(connection, input).await
            }
            ScheduleCommands::ReadOne { rule_id } => {
                app.schedule_work_day_rules_read(connection, rule_id)
                    .await
            }
            ScheduleCommands::Update { rule_id, input } => {
                let body = required_json_input(input)?;
                app.schedule_work_day_rules_update(connection, rule_id, body)
                    .await
            }
            ScheduleCommands::Create { input } => {
                let body = required_json_input(input)?;
                app.schedule_work_day_rules_create(connection, body).await
            }
            ScheduleCommands::Delete { rule_id } => {
                app.schedule_work_day_rules_delete(connection, rule_id)
                    .await
            }
        },
        Commands::License(command) => match command {
            LicenseCommands::Get => app.license(connection).await,
        },
        Commands::SelfCommand(command) => match command {
            SelfCommands::Update(args) => self_update::run(args).await,
        },
        Commands::Config(command) => config::run(command),
        Commands::SelfUpdate(args) => self_update::run(args).await,
    }
}

fn required_json_input(input: String) -> Result<Value> {
    read_json_input(Some(&input))?.ok_or_else(|| {
        AppError::invalid_input(
            "this command requires a JSON input payload; provide inline JSON, `@file.json`, or `@-` for stdin",
        )
    })
}
