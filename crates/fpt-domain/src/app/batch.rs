use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Instant;

use fpt_core::{AppError, Result};
use futures::stream::{self, StreamExt};
use serde_json::{Value, json};

use crate::config::{ConnectionOverrides, ConnectionSettings, api_version_or_default};
use crate::transport::{
    RequestPlan, ShotgridTransport, plan_entity_create, plan_entity_delete, plan_entity_revive,
    plan_entity_update,
};

use super::find::{build_find_params, upsert_query_param};
use super::query_helpers::string_list;
use super::{App, BatchUpdateItem, batch_concurrency_limit, sort_batch_results};

/// Convert an [`Instant`] elapsed duration to whole milliseconds.
///
/// This is a lossless helper: `Duration::as_millis()` returns `u128` but all
/// realistic batch durations fit comfortably in `u64`.  Using a dedicated
/// function avoids scattering bare `as u64` casts throughout the batch methods.
#[inline]
fn elapsed_ms(started_at: Instant) -> u64 {
    started_at
        .elapsed()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

/// How to handle a conflict when an entity with the key field value already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnConflict {
    /// Skip the item — do not create or update.
    Skip,
    /// Update the existing entity with the new body.
    Update,
    /// Return an error for the item.
    Error,
}

impl std::fmt::Display for OnConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Skip => f.write_str("skip"),
            Self::Update => f.write_str("update"),
            Self::Error => f.write_str("error"),
        }
    }
}

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn entity_batch_get(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
    ) -> Result<Value> {
        let (ids, fields) = parse_batch_get_input(input)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let started_at = Instant::now();
        let mut results = stream::iter(ids.into_iter().enumerate())
            .map(|(index, id)| {
                let fields = fields.clone();
                async move {
                    let id_val = json!(id);
                    match transport.entity_get(config, entity, id, fields).await {
                        Ok(response) => {
                            batch_result_ok(index, &[("id", id_val), ("response", response)])
                        }
                        Err(error) => batch_result_err(index, &error, &[("id", id_val)]),
                    }
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response_with_stats(
            "entity.batch.get",
            entity,
            results,
            elapsed_ms(started_at),
        ))
    }

    pub async fn entity_batch_find(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
    ) -> Result<Value> {
        let requests = parse_batch_find_input(input)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let started_at = Instant::now();
        let mut results = stream::iter(requests.into_iter().enumerate())
            .map(|(index, request)| async move {
                match build_find_params(Some(request.clone()), None) {
                    Ok(params) => match transport.entity_find(config, entity, params).await {
                        Ok(response) => {
                            batch_result_ok(index, &[("request", request), ("response", response)])
                        }
                        Err(error) => batch_result_err(index, &error, &[("request", request)]),
                    },
                    Err(error) => batch_result_err(index, &error, &[("request", request)]),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response_with_stats(
            "entity.batch.find",
            entity,
            results,
            elapsed_ms(started_at),
        ))
    }

    pub async fn entity_batch_create(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
        dry_run: bool,
    ) -> Result<Value> {
        let items = parse_batch_create_input(input)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            let plans = items
                .into_iter()
                .map(|body| plan_entity_create(&api_version, entity, body))
                .collect::<Vec<_>>();
            return Ok(batch_dry_run_response("entity.batch.create", entity, plans));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let started_at = Instant::now();
        let mut results = stream::iter(items.into_iter().enumerate())
            .map(|(index, body)| async move {
                match transport.entity_create(config, entity, &body).await {
                    Ok(response) => {
                        batch_result_ok(index, &[("request", body), ("response", response)])
                    }
                    Err(error) => batch_result_err(index, &error, &[("request", body)]),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response_with_stats(
            "entity.batch.create",
            entity,
            results,
            elapsed_ms(started_at),
        ))
    }

    pub async fn entity_batch_update(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
        dry_run: bool,
    ) -> Result<Value> {
        let items = parse_batch_update_input(input)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            let plans = items
                .iter()
                .map(|item| plan_entity_update(&api_version, entity, item.id, item.body.clone()))
                .collect::<Vec<_>>();
            return Ok(batch_dry_run_response("entity.batch.update", entity, plans));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let started_at = Instant::now();
        let mut results = stream::iter(items.into_iter().enumerate())
            .map(|(index, item)| async move {
                match transport
                    .entity_update(config, entity, item.id, &item.body)
                    .await
                {
                    Ok(response) => batch_result_ok(
                        index,
                        &[
                            ("id", json!(item.id)),
                            ("request", item.body),
                            ("response", response),
                        ],
                    ),
                    Err(error) => batch_result_err(
                        index,
                        &error,
                        &[("id", json!(item.id)), ("request", item.body)],
                    ),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response_with_stats(
            "entity.batch.update",
            entity,
            results,
            elapsed_ms(started_at),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn entity_batch_upsert(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
        key: &str,
        on_conflict: OnConflict,
        dry_run: bool,
        checkpoint_path: Option<String>,
        resume: bool,
    ) -> Result<Value> {
        let items = parse_batch_create_input(input)?;

        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            let plans = items
                .into_iter()
                .map(|body| {
                    json!({
                        "action": "create_or_update",
                        "key": key,
                        "on_conflict": on_conflict.to_string(),
                        "plan": plan_entity_create(&api_version, entity, body),
                    })
                })
                .collect::<Vec<_>>();
            return Ok(json!({
                "dry_run": true,
                "operation": "entity.batch.upsert",
                "entity": entity,
                "key": key,
                "on_conflict": on_conflict.to_string(),
                "count": plans.len(),
                "plans": plans,
            }));
        }

        // Load checkpoint if resuming.
        let completed_indices = if resume {
            if let Some(ref path) = checkpoint_path {
                load_checkpoint_indices(path)?
            } else {
                return Err(AppError::invalid_input(
                    "`--resume` requires `--checkpoint` to specify the checkpoint file path",
                )
                .with_operation("entity_batch_upsert")
                .with_hint("Provide both `--checkpoint <file>` and `--resume` to resume an interrupted upsert."));
            }
        } else {
            HashSet::new()
        };

        let checkpoint_writer = checkpoint_path
            .as_ref()
            .map(|path| open_checkpoint_writer(path, resume))
            .transpose()?;
        // Wrap in a mutex for concurrent access.
        let checkpoint_writer = checkpoint_writer.map(std::sync::Mutex::new);

        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let key = key.to_string();
        let started_at = Instant::now();
        let total_items = items.len();
        let mut results = stream::iter(items.into_iter().enumerate())
            .map(|(index, body)| {
                let key = key.clone();
                let checkpoint_writer = &checkpoint_writer;
                let completed_indices = &completed_indices;
                async move {
                    // Skip items already completed in a previous run.
                    if completed_indices.contains(&index) {
                        let result = json!({
                            "index": index,
                            "ok": true,
                            "action": "resumed_skip",
                            "request": body,
                        });
                        return result;
                    }

                    // Look up whether an entity with this key value already exists.
                    let key_value = match body.get(&key) {
                        Some(v) => v.clone(),
                        None => {
                            let result = json!({
                                "index": index,
                                "ok": false,
                                "action": "skipped",
                                "request": body,
                                "error": {
                                    "code": "INVALID_INPUT",
                                    "message": format!("upsert key field `{key}` is missing from item {}", index + 1),
                                },
                            });
                            write_checkpoint(checkpoint_writer, &result);
                            return result;
                        }
                    };

                    // Build a find-one query for the key field.
                    let filter_input = json!({
                        "search": {
                            "filters": {
                                "filter_operator": "all",
                                "filters": [[key, "is", key_value]],
                            }
                        }
                    });
                    let mut params = match build_find_params(Some(filter_input), None) {
                        Ok(p) => p,
                        Err(error) => {
                            let result = batch_result_err(
                                index,
                                &error,
                                &[("action", json!("error")), ("request", body)],
                            );
                            write_checkpoint(checkpoint_writer, &result);
                            return result;
                        }
                    };
                    upsert_query_param(&mut params.query, "page[size]", "1");

                    let existing = match transport.entity_find(config, entity, params).await {
                        Ok(response) => {
                            // Extract first item from data array
                            response
                                .get("data")
                                .and_then(|d| d.as_array())
                                .and_then(|arr| arr.first())
                                .cloned()
                        }
                        Err(error) => {
                            let result = batch_result_err(
                                index,
                                &error,
                                &[("action", json!("error")), ("request", body)],
                            );
                            write_checkpoint(checkpoint_writer, &result);
                            return result;
                        }
                    };

                    let result = match existing {
                        None => {
                            // No existing entity — create it.
                            match transport.entity_create(config, entity, &body).await {
                                Ok(response) => batch_result_ok(
                                    index,
                                    &[
                                        ("action", json!("created")),
                                        ("request", body),
                                        ("response", response),
                                    ],
                                ),
                                Err(error) => batch_result_err(
                                    index,
                                    &error,
                                    &[("action", json!("error")), ("request", body)],
                                ),
                            }
                        }
                        Some(existing_entity) => {
                            match on_conflict {
                                OnConflict::Skip => batch_result_ok(
                                    index,
                                    &[
                                        ("action", json!("skipped")),
                                        ("request", body),
                                        ("existing", existing_entity),
                                    ],
                                ),
                                OnConflict::Error => {
                                    let existing_id = existing_entity
                                        .get("id")
                                        .and_then(Value::as_u64);
                                    let id_display = existing_id
                                        .map_or_else(|| "unknown".to_string(), |id| id.to_string());
                                    let error = AppError::policy_blocked(format!(
                                        "entity with {key}={} already exists (id={id_display}); use --on-conflict skip or update to handle conflicts",
                                        body.get(&key).unwrap_or(&Value::Null)
                                    ))
                                    .with_operation("entity_batch_upsert");
                                    batch_result_err(
                                        index,
                                        &error,
                                        &[
                                            ("action", json!("conflict")),
                                            ("request", body),
                                            ("existing", existing_entity),
                                        ],
                                    )
                                }
                                OnConflict::Update => {
                                    let existing_id = match existing_entity
                                        .get("id")
                                        .and_then(Value::as_u64)
                                    {
                                        Some(id) => id,
                                        None => {
                                            let error = AppError::api(
                                                "existing entity is missing `id` field",
                                            )
                                            .with_operation("entity_batch_upsert");
                                            return batch_result_err(
                                                index,
                                                &error,
                                                &[
                                                    ("action", json!("error")),
                                                    ("request", body),
                                                ],
                                            );
                                        }
                                    };
                                    match transport
                                        .entity_update(config, entity, existing_id, &body)
                                        .await
                                    {
                                        Ok(response) => batch_result_ok(
                                            index,
                                            &[
                                                ("action", json!("updated")),
                                                ("id", json!(existing_id)),
                                                ("request", body),
                                                ("response", response),
                                            ],
                                        ),
                                        Err(error) => batch_result_err(
                                            index,
                                            &error,
                                            &[
                                                ("action", json!("error")),
                                                ("id", json!(existing_id)),
                                                ("request", body),
                                            ],
                                        ),
                                    }
                                }
                            }
                        }
                    };

                    write_checkpoint(checkpoint_writer, &result);
                    result
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);
        let elapsed = elapsed_ms(started_at);

        let failure_count = results
            .iter()
            .filter(|item| item.get("ok").and_then(Value::as_bool) != Some(true))
            .count();
        let success_count = results.len().saturating_sub(failure_count);
        let created_count = count_by_action(&results, "created");
        let updated_count = count_by_action(&results, "updated");
        let skipped_count = count_by_action(&results, "skipped");
        let resumed_skip_count = count_by_action(&results, "resumed_skip");

        Ok(json!({
            "ok": failure_count == 0,
            "operation": "entity.batch.upsert",
            "entity": entity,
            "key": key,
            "on_conflict": on_conflict.to_string(),
            "total": total_items,
            "success_count": success_count,
            "failure_count": failure_count,
            "created_count": created_count,
            "updated_count": updated_count,
            "skipped_count": skipped_count,
            "resumed_skip_count": resumed_skip_count,
            "checkpoint": checkpoint_path,
            "stats": {
                "elapsed_ms": elapsed,
            },
            "results": results,
        }))
    }

    pub async fn entity_batch_delete(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
        dry_run: bool,
        yes: bool,
    ) -> Result<Value> {
        let ids = parse_batch_id_list_input(input)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            let plans = ids
                .into_iter()
                .map(|id| plan_entity_delete(&api_version, entity, id))
                .collect::<Vec<_>>();
            return Ok(batch_dry_run_response("entity.batch.delete", entity, plans));
        }

        if !yes {
            return Err(AppError::policy_blocked(
                "entity batch delete is a destructive operation; pass `--yes` to execute it, or use `--dry-run` to inspect the request plan first",
            )
            .with_operation("entity_batch_delete")
            .with_hint("Add `--yes` to confirm the batch deletion, or `--dry-run` to preview the request plans without executing them."));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let started_at = Instant::now();
        let mut results = stream::iter(ids.into_iter().enumerate())
            .map(|(index, id)| async move {
                let id_val = json!(id);
                match transport.entity_delete(config, entity, id).await {
                    Ok(response) => {
                        batch_result_ok(index, &[("id", id_val), ("response", response)])
                    }
                    Err(error) => batch_result_err(index, &error, &[("id", id_val)]),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response_with_stats(
            "entity.batch.delete",
            entity,
            results,
            elapsed_ms(started_at),
        ))
    }

    pub async fn entity_batch_revive(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
        dry_run: bool,
    ) -> Result<Value> {
        let ids = parse_batch_id_list_input(input)?;
        if dry_run {
            let plans = ids
                .iter()
                .map(|&id| plan_entity_revive(entity, id))
                .collect::<Vec<_>>();
            return Ok(batch_dry_run_response("entity.batch.revive", entity, plans));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let started_at = Instant::now();
        let mut results = stream::iter(ids.into_iter().enumerate())
            .map(|(index, id)| async move {
                let id_val = json!(id);
                match transport.entity_revive(config, entity, id).await {
                    Ok(response) => {
                        batch_result_ok(index, &[("id", id_val), ("response", response)])
                    }
                    Err(error) => batch_result_err(index, &error, &[("id", id_val)]),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response_with_stats(
            "entity.batch.revive",
            entity,
            results,
            elapsed_ms(started_at),
        ))
    }

    pub async fn entity_batch_find_one(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
    ) -> Result<Value> {
        let requests = parse_batch_find_input(input)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let started_at = Instant::now();
        let mut results = stream::iter(requests.into_iter().enumerate())
            .map(|(index, request)| async move {
                match build_find_params(Some(request.clone()), None) {
                    Ok(mut params) => {
                        upsert_query_param(&mut params.query, "page[size]", "1");
                        match transport.entity_find(config, entity, params).await {
                            Ok(response) => {
                                let record = super::find::extract_find_one_response(response);
                                match record {
                                    Ok(value) => batch_result_ok(
                                        index,
                                        &[("request", request), ("response", value)],
                                    ),
                                    Err(error) => {
                                        batch_result_err(index, &error, &[("request", request)])
                                    }
                                }
                            }
                            Err(error) => batch_result_err(index, &error, &[("request", request)]),
                        }
                    }
                    Err(error) => batch_result_err(index, &error, &[("request", request)]),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response_with_stats(
            "entity.batch.find-one",
            entity,
            results,
            elapsed_ms(started_at),
        ))
    }
    pub async fn entity_batch_summarize(
        &self,
        overrides: ConnectionOverrides,
        input: Value,
    ) -> Result<Value> {
        let requests = parse_batch_summarize_input(input)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let started_at = Instant::now();
        let mut results = stream::iter(requests.into_iter().enumerate())
            .map(|(index, request)| async move {
                let entity = request
                    .get("entity")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string();
                let payload = request.get("payload").cloned().unwrap_or(json!({}));
                match transport.entity_summarize(config, &entity, &payload).await {
                    Ok(response) => batch_result_ok(
                        index,
                        &[
                            ("entity", json!(entity)),
                            ("request", request),
                            ("response", response),
                        ],
                    ),
                    Err(error) => batch_result_err(
                        index,
                        &error,
                        &[("entity", json!(entity)), ("request", request)],
                    ),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response_with_stats(
            "entity.batch.summarize",
            "_multi",
            results,
            elapsed_ms(started_at),
        ))
    }

    pub async fn entity_batch_count(
        &self,
        overrides: ConnectionOverrides,
        input: Value,
    ) -> Result<Value> {
        let requests = parse_batch_count_input(input)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let started_at = Instant::now();
        let mut results = stream::iter(requests.into_iter().enumerate())
            .map(|(index, request)| async move {
                let entity = request
                    .get("entity")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string();
                let payload = request.get("payload").cloned().unwrap_or(json!({}));
                match transport.entity_summarize(config, &entity, &payload).await {
                    Ok(response) => batch_result_ok(
                        index,
                        &[
                            ("entity", json!(entity)),
                            ("request", request),
                            ("response", response),
                        ],
                    ),
                    Err(error) => batch_result_err(
                        index,
                        &error,
                        &[("entity", json!(entity)), ("request", request)],
                    ),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response_with_stats(
            "entity.batch.count",
            "_multi",
            results,
            elapsed_ms(started_at),
        ))
    }

    /// Execute a server-side batch request via the REST `_batch` endpoint.
    ///
    /// Unlike the client-side batch commands that orchestrate multiple
    /// individual REST calls, this sends a single transactional request to
    /// `POST /api/{ver}/entity/_batch`.  The server executes all operations
    /// atomically: either all succeed or all roll back.
    pub async fn entity_batch_server(
        &self,
        overrides: ConnectionOverrides,
        input: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let body = validate_batch_server_input(input)?;
        self.transport.entity_batch_server(&config, &body).await
    }
}

/// Build a single successful batch result entry.
///
/// Accepts any number of extra key-value pairs that are merged into the
/// resulting JSON object alongside the standard `"index"` and `"ok": true`
/// fields.  This avoids scattering identical `json!({...})` templates
/// across every batch method.
fn batch_result_ok(index: usize, extras: &[(&str, Value)]) -> Value {
    let mut map = serde_json::Map::with_capacity(3 + extras.len());
    map.insert("index".to_string(), json!(index));
    map.insert("ok".to_string(), json!(true));
    for &(key, ref value) in extras {
        map.insert(key.to_string(), value.clone());
    }
    Value::Object(map)
}

/// Build a single failed batch result entry.
///
/// Works like [`batch_result_ok`] but sets `"ok": false` and includes the
/// serialised error envelope under the `"error"` key.
fn batch_result_err(index: usize, error: &AppError, extras: &[(&str, Value)]) -> Value {
    let mut map = serde_json::Map::with_capacity(4 + extras.len());
    map.insert("index".to_string(), json!(index));
    map.insert("ok".to_string(), json!(false));
    for &(key, ref value) in extras {
        map.insert(key.to_string(), value.clone());
    }
    map.insert(
        "error".to_string(),
        serde_json::to_value(error.envelope()).unwrap_or(json!({"code": "INTERNAL_ERROR"})),
    );
    Value::Object(map)
}

/// Build a batch response that includes execution statistics.
///
/// `elapsed_ms` is the total wall-clock time for the batch operation.
fn batch_response_with_stats(
    operation: &str,
    entity: &str,
    results: Vec<Value>,
    elapsed_ms: u64,
) -> Value {
    let failure_count = results
        .iter()
        .filter(|item| item.get("ok").and_then(Value::as_bool) != Some(true))
        .count();
    let success_count = results.len().saturating_sub(failure_count);
    let total = results.len();
    let throughput_eps = if elapsed_ms > 0 {
        (total as f64) / (elapsed_ms as f64 / 1000.0)
    } else {
        0.0
    };

    json!({
        "ok": failure_count == 0,
        "operation": operation,
        "entity": entity,
        "total": total,
        "success_count": success_count,
        "failure_count": failure_count,
        "stats": {
            "elapsed_ms": elapsed_ms,
            "throughput_eps": (throughput_eps * 100.0).round() / 100.0,
        },
        "results": results,
    })
}

fn batch_dry_run_response(operation: &str, entity: &str, plans: Vec<RequestPlan>) -> Value {
    json!({
        "dry_run": true,
        "operation": operation,
        "entity": entity,
        "count": plans.len(),
        "plans": plans,
    })
}

fn parse_batch_get_input(input: Value) -> Result<(Vec<u64>, Option<Vec<String>>)> {
    match input {
        Value::Array(values) => Ok((u64_list(&values, "ids")?, None)),
        Value::Object(object) => {
            let ids = object
                .get("ids")
                .ok_or_else(|| {
                    AppError::invalid_input(
                        "entity batch get input is missing required field `ids`",
                    )
                    .with_operation("parse_batch_get_input")
                    .with_missing_fields(["ids"])
                    .with_expected_shape("a JSON object containing `ids` (array of positive integers) and optional `fields`")
                })?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`ids` must be a JSON array of positive integers")
                    .with_operation("parse_batch_get_input")
                    .with_invalid_field("ids")
                    .with_expected_shape("a JSON array of positive integers"))?;
            let ids = u64_list(ids, "ids")?;
            let fields = object
                .get("fields")
                .map(|value| string_list(value, "fields"))
                .transpose()?;
            Ok((ids, fields.filter(|items| !items.is_empty())))
        }
        _ => Err(AppError::invalid_input(
            "entity batch get input must be either a JSON array of ids or an object containing `ids` and optional `fields`",
        )
        .with_operation("parse_batch_get_input")
        .with_expected_shape("a JSON array of positive integers, or a JSON object containing `ids` and optional `fields`")),
    }
}

fn parse_batch_find_input(input: Value) -> Result<Vec<Value>> {
    match input {
        Value::Array(values) => object_items(values, "entity batch find requests"),
        Value::Object(object) => {
            let requests = object
                .get("requests")
                .ok_or_else(|| {
                    AppError::invalid_input(
                        "entity batch find input is missing required field `requests`",
                    )
                    .with_operation("parse_batch_find_input")
                    .with_missing_fields(["requests"])
                    .with_expected_shape("a JSON object containing `requests` (array of find request objects)")
                })?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`requests` must be a JSON array of objects")
                    .with_operation("parse_batch_find_input")
                    .with_invalid_field("requests")
                    .with_expected_shape("a JSON array of find request objects"))?
                .clone();
            object_items(requests, "requests")
        }
        _ => Err(AppError::invalid_input(
            "entity batch find input must be either a JSON array of request objects or an object containing `requests`",
        )
        .with_operation("parse_batch_find_input")
        .with_expected_shape("a JSON array of request objects, or a JSON object containing `requests`")),
    }
}

fn parse_batch_create_input(input: Value) -> Result<Vec<Value>> {
    match input {
        Value::Array(values) => non_empty_items(values, "entity batch create items"),
        Value::Object(object) => {
            let items = object
                .get("items")
                .ok_or_else(|| {
                    AppError::invalid_input(
                        "entity batch create input is missing required field `items`",
                    )
                    .with_operation("parse_batch_create_input")
                    .with_missing_fields(["items"])
                    .with_expected_shape("a JSON object containing `items` (array of entity body objects)")
                })?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`items` must be a JSON array of objects")
                    .with_operation("parse_batch_create_input")
                    .with_invalid_field("items")
                    .with_expected_shape("a JSON array of entity body objects"))?
                .clone();
            non_empty_items(items, "items")
        }
        _ => Err(AppError::invalid_input(
            "entity batch create input must be either a JSON array of item bodies or an object containing `items`",
        )
        .with_operation("parse_batch_create_input")
        .with_expected_shape("a JSON array of entity body objects, or a JSON object containing `items`")),
    }
}

fn parse_batch_update_input(input: Value) -> Result<Vec<BatchUpdateItem>> {
    let items = match input {
        Value::Array(values) => non_empty_items(values, "entity batch update items")?,
        Value::Object(object) => {
            let items = object
                .get("items")
                .ok_or_else(|| {
                    AppError::invalid_input(
                        "entity batch update input is missing required field `items`",
                    )
                    .with_operation("parse_batch_update_input")
                    .with_missing_fields(["items"])
                    .with_expected_shape("a JSON object containing `items` (array of update objects with `id` and `body`)")
                })?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`items` must be a JSON array of objects")
                    .with_operation("parse_batch_update_input")
                    .with_invalid_field("items")
                    .with_expected_shape("a JSON array of update objects with `id` and `body`"))?
                .clone();
            non_empty_items(items, "items")?
        }
        _ => {
            return Err(AppError::invalid_input(
                "entity batch update input must be either a JSON array of update objects or an object containing `items`",
            )
            .with_operation("parse_batch_update_input")
            .with_expected_shape("a JSON array of update objects, or a JSON object containing `items`"));
        }
    };

    items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let object = item.as_object().ok_or_else(|| {
                AppError::invalid_input(format!(
                    "item {} in entity batch update must be a JSON object with `id` and `body` fields",
                    index + 1
                ))
                .with_operation("parse_batch_update_input")
                .with_invalid_field("items")
                .with_expected_shape("each item must be a JSON object containing `id` (positive integer) and `body` (object)")
            })?;
            let id = object
                .get("id")
                .ok_or_else(|| {
                    AppError::invalid_input(format!(
                        "item {} in entity batch update is missing required field `id`",
                        index + 1
                    ))
                    .with_operation("parse_batch_update_input")
                    .with_missing_fields(["id"])
                })
                .and_then(|value| {
                    value.as_u64().ok_or_else(|| {
                        AppError::invalid_input(format!(
                            "`id` in item {} of entity batch update must be a positive integer",
                            index + 1
                        ))
                        .with_operation("parse_batch_update_input")
                        .with_invalid_field("id")
                        .with_expected_shape("a positive integer")
                    })
                })?;
            let body = object.get("body").cloned().ok_or_else(|| {
                AppError::invalid_input(format!(
                    "item {} in entity batch update is missing required field `body`",
                    index + 1
                ))
                .with_operation("parse_batch_update_input")
                .with_missing_fields(["body"])
            })?;
            Ok(BatchUpdateItem { id, body })
        })
        .collect()
}

/// Parse a batch input that contains a flat list of entity ids.
///
/// Used by both batch-delete and batch-revive commands.
fn parse_batch_id_list_input(input: Value) -> Result<Vec<u64>> {
    match input {
        Value::Array(values) => u64_list(&values, "ids"),
        Value::Object(object) => {
            let ids = object
                .get("ids")
                .ok_or_else(|| {
                    AppError::invalid_input("batch id list input is missing required field `ids`")
                        .with_operation("parse_batch_id_list_input")
                        .with_missing_fields(["ids"])
                        .with_expected_shape(
                            "a JSON object containing `ids` (array of positive integers)",
                        )
                })?
                .as_array()
                .ok_or_else(|| {
                    AppError::invalid_input("`ids` must be a JSON array of positive integers")
                        .with_operation("parse_batch_id_list_input")
                        .with_invalid_field("ids")
                        .with_expected_shape("a JSON array of positive integers")
                })?;
            u64_list(ids, "ids")
        }
        _ => Err(AppError::invalid_input(
            "batch id list input must be either a JSON array of ids or an object containing `ids`",
        )
        .with_operation("parse_batch_id_list_input")
        .with_expected_shape(
            "a JSON array of positive integers, or a JSON object containing `ids`",
        )),
    }
}

fn non_empty_items(values: Vec<Value>, field_name: &str) -> Result<Vec<Value>> {
    if values.is_empty() {
        return Err(
            AppError::invalid_input(format!("`{field_name}` must not be an empty array"))
                .with_operation("validate_batch_input")
                .with_invalid_field(field_name)
                .with_hint("Provide at least one item in the array."),
        );
    }
    Ok(values)
}

fn object_items(values: Vec<Value>, field_name: &str) -> Result<Vec<Value>> {
    let values = non_empty_items(values, field_name)?;
    for (index, value) in values.iter().enumerate() {
        if !value.is_object() {
            return Err(AppError::invalid_input(format!(
                "item {} in `{field_name}` must be a JSON object",
                index + 1
            ))
            .with_operation("validate_batch_input")
            .with_invalid_field(field_name)
            .with_expected_shape("each item must be a JSON object"));
        }
    }
    Ok(values)
}

fn u64_list(values: &[Value], field_name: &str) -> Result<Vec<u64>> {
    if values.is_empty() {
        return Err(
            AppError::invalid_input(format!("`{field_name}` must not be an empty array"))
                .with_operation("validate_batch_input")
                .with_invalid_field(field_name)
                .with_hint("Provide at least one id in the array."),
        );
    }

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value.as_u64().ok_or_else(|| {
                AppError::invalid_input(format!(
                    "item {} in `{field_name}` must be a positive integer",
                    index + 1
                ))
                .with_operation("validate_batch_input")
                .with_invalid_field(field_name)
                .with_expected_shape("a positive integer")
            })
        })
        .collect()
}

/// Count batch result items that match a specific `"action"` string value.
fn count_by_action(results: &[Value], action: &str) -> usize {
    results
        .iter()
        .filter(|item| item.get("action").and_then(Value::as_str) == Some(action))
        .count()
}

// ---------------------------------------------------------------------------
// Checkpoint helpers for resumable bulk upsert
// ---------------------------------------------------------------------------

/// Read a JSONL checkpoint file and return the set of indices that were
/// already completed in a previous run.
///
/// Each line in the checkpoint file is a JSON object that MUST contain an
/// `"index"` field (non-negative integer). Lines that cannot be parsed or
/// lack the field are silently skipped so that a partially written last line
/// does not prevent resumption.
fn load_checkpoint_indices(path: &str) -> Result<HashSet<usize>> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(AppError::invalid_input(format!(
            "checkpoint file `{}` does not exist; cannot resume without it",
            path.display()
        ))
        .with_operation("load_checkpoint_indices")
        .with_hint(
            "Run the upsert without `--resume` first, or provide an existing checkpoint file path.",
        ));
    }

    let file = File::open(path).map_err(|e| {
        AppError::internal(format!(
            "failed to open checkpoint file `{}`: {e}",
            path.display()
        ))
        .with_operation("load_checkpoint_indices")
    })?;

    let reader = BufReader::new(file);
    let mut indices = HashSet::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue, // skip unreadable lines
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            if let Some(idx) = value.get("index").and_then(Value::as_u64) {
                if let Ok(i) = usize::try_from(idx) {
                    indices.insert(i);
                }
            }
        }
        // Silently skip malformed lines so a partial last write doesn't block resume.
    }

    Ok(indices)
}

/// Open (or create) the checkpoint file for writing.
///
/// When `resume` is true the file is opened in **append** mode so that new
/// entries are added after the existing ones.  When `resume` is false the file
/// is created (or truncated) so that a fresh checkpoint is started.
fn open_checkpoint_writer(path: &str, resume: bool) -> Result<File> {
    let result = if resume {
        OpenOptions::new().create(true).append(true).open(path)
    } else {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
    };

    result.map_err(|e| {
        AppError::internal(format!(
            "failed to open checkpoint file `{path}` for writing: {e}"
        ))
        .with_operation("open_checkpoint_writer")
    })
}

/// Append a single result entry to the checkpoint file as a JSONL line.
///
/// If the checkpoint writer is `None` (i.e. no `--checkpoint` was specified)
/// this is a no-op.  Write errors are silently ignored because losing a
/// checkpoint line is acceptable — the item will simply be re-processed on
/// the next resume.
fn write_checkpoint(writer: &Option<std::sync::Mutex<File>>, result: &Value) {
    if let Some(mutex) = writer {
        if let Ok(mut file) = mutex.lock() {
            // serde_json::to_string won't fail for Value
            if let Ok(line) = serde_json::to_string(result) {
                let _ = writeln!(file, "{line}");
            }
        }
    }
}

/// Parse a batch summarize input.
///
/// Each request must be an object with `entity` (string) and `payload` (object).
/// Accepts either a JSON array of request objects or an object containing `requests`.
fn parse_batch_summarize_input(input: Value) -> Result<Vec<Value>> {
    let items = match input {
        Value::Array(values) => non_empty_items(values, "entity batch summarize requests")?,
        Value::Object(object) => {
            let requests = object
                .get("requests")
                .ok_or_else(|| {
                    AppError::invalid_input(
                        "entity batch summarize input is missing required field `requests`",
                    )
                    .with_operation("parse_batch_summarize_input")
                    .with_missing_fields(["requests"])
                    .with_expected_shape(
                        "a JSON object containing `requests` (array of summarize request objects)",
                    )
                })?
                .as_array()
                .ok_or_else(|| {
                    AppError::invalid_input("`requests` must be a JSON array of objects")
                        .with_operation("parse_batch_summarize_input")
                        .with_invalid_field("requests")
                        .with_expected_shape("a JSON array of summarize request objects")
                })?
                .clone();
            non_empty_items(requests, "requests")?
        }
        _ => {
            return Err(AppError::invalid_input(
                "entity batch summarize input must be either a JSON array of request objects or an object containing `requests`",
            )
            .with_operation("parse_batch_summarize_input")
            .with_expected_shape("a JSON array of request objects, or a JSON object containing `requests`"));
        }
    };

    // Validate each item has the required fields.
    for (index, item) in items.iter().enumerate() {
        let object = item.as_object().ok_or_else(|| {
            AppError::invalid_input(format!(
                "item {} in entity batch summarize must be a JSON object with `entity` and `payload`",
                index + 1
            ))
            .with_operation("parse_batch_summarize_input")
            .with_expected_shape("each request must be a JSON object containing `entity` (string) and `payload` (object)")
        })?;
        if !object.contains_key("entity") {
            return Err(AppError::invalid_input(format!(
                "item {} is missing required field `entity`",
                index + 1
            ))
            .with_operation("parse_batch_summarize_input")
            .with_missing_fields(["entity"]));
        }
        if !object.contains_key("payload") {
            return Err(AppError::invalid_input(format!(
                "item {} is missing required field `payload`",
                index + 1
            ))
            .with_operation("parse_batch_summarize_input")
            .with_missing_fields(["payload"]));
        }
    }

    Ok(items)
}

/// Parse a batch count input.
///
/// Each request can be a string (entity name) or an object with `entity` and
/// optional `filters` / `filter_operator`.  The function wraps each request
/// into a `record_count` summarize payload.
/// Accepts a JSON array of entity name strings or request objects,
/// or a JSON object containing `entities` or `requests`.
fn parse_batch_count_input(input: Value) -> Result<Vec<Value>> {
    let items = match input {
        Value::Array(values) => {
            let items = non_empty_items(values, "entity batch count requests")?;
            items
                .into_iter()
                .map(|item| match item {
                    Value::String(entity) => Ok(json!({
                        "entity": entity,
                        "payload": {
                            "summaries": [{"field": "id", "type": "record_count"}],
                            "filters": {"filter_operator": "all", "filters": []},
                        }
                    })),
                    Value::Object(object) => {
                        let entity = object
                            .get("entity")
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                AppError::invalid_input(
                                    "each batch count request must contain an `entity` string field",
                                )
                                .with_operation("parse_batch_count_input")
                                .with_missing_fields(["entity"])
                            })?;
                        let filters = object.get("filters").cloned().unwrap_or(json!([]));
                        let filter_operator = object
                            .get("filter_operator")
                            .and_then(Value::as_str)
                            .unwrap_or("all");
                        Ok(json!({
                            "entity": entity,
                            "payload": {
                                "summaries": [{"field": "id", "type": "record_count"}],
                                "filters": {
                                    "filter_operator": filter_operator,
                                    "filters": filters,
                                },
                            }
                        }))
                    }
                    _ => Err(AppError::invalid_input(
                        "each batch count item must be either a string (entity name) or an object with `entity`",
                    )
                    .with_operation("parse_batch_count_input")
                    .with_expected_shape("a string entity name or a JSON object with `entity` field")),
                })
                .collect::<Result<Vec<_>>>()?
        }
        Value::Object(object) => {
            let array_field = object
                .get("entities")
                .or_else(|| object.get("requests"))
                .ok_or_else(|| {
                    AppError::invalid_input(
                        "entity batch count input is missing required field `entities` or `requests`",
                    )
                    .with_operation("parse_batch_count_input")
                    .with_missing_fields(["entities"])
                    .with_expected_shape(
                        "a JSON object containing `entities` (array of entity names or request objects)",
                    )
                })?
                .as_array()
                .ok_or_else(|| {
                    AppError::invalid_input("`entities` must be a JSON array")
                        .with_operation("parse_batch_count_input")
                        .with_invalid_field("entities")
                        .with_expected_shape("a JSON array of entity names or request objects")
                })?
                .clone();
            parse_batch_count_input(Value::Array(array_field))?
        }
        _ => {
            return Err(AppError::invalid_input(
                "entity batch count input must be either a JSON array or an object containing `entities`",
            )
            .with_operation("parse_batch_count_input")
            .with_expected_shape(
                "a JSON array of entity names/objects, or a JSON object containing `entities`",
            ));
        }
    };

    Ok(items)
}

/// Validate the input for the server-side batch endpoint.
///
/// The body must be a JSON object; the server validates the detailed
/// request structure (`requests` array, per-request entity/method, etc.).
/// This function only rejects obviously malformed input so the CLI fails
/// fast with a clear message.
fn validate_batch_server_input(input: Value) -> Result<Value> {
    if !input.is_object() {
        return Err(
            AppError::invalid_input("entity batch-server input must be a JSON object")
                .with_operation("validate_batch_server_input")
                .with_expected_shape(
                    "a JSON object containing `requests` (array of server-side batch operation descriptors)",
                ),
        );
    }
    Ok(input)
}
