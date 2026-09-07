use fpt_core::{AppError, Result};
use serde_json::{Map, Value, json};

use crate::config::{ConnectionOverrides, ConnectionSettings, api_version_or_default};
use crate::filter_dsl::parse_filter_dsl;
use crate::transport::{
    RequestPlan, ShotgridTransport, plan_entity_create, plan_entity_delete, plan_entity_revive,
    plan_entity_update,
};

use super::App;
use super::find::{build_find_params, extract_find_one_response, upsert_query_param};
use super::query_helpers::{build_query_params, normalize_filters};

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn entity_get(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        fields: Option<Vec<String>>,
    ) -> Result<Value> {
        validate_entity_type(entity)?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_get(&config, entity, id, fields).await
    }

    pub async fn entity_find(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Option<Value>,
        filter_dsl: Option<String>,
    ) -> Result<Value> {
        validate_entity_type(entity)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_find_params(input, filter_dsl)?;
        self.transport.entity_find(&config, entity, params).await
    }

    pub async fn entity_find_one(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Option<Value>,
        filter_dsl: Option<String>,
    ) -> Result<Value> {
        validate_entity_type(entity)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let mut params = build_find_params(input, filter_dsl)?;
        upsert_query_param(&mut params.query, "page[size]", "1");
        let response = self.transport.entity_find(&config, entity, params).await?;
        extract_find_one_response(response)
    }

    pub async fn entity_create(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        body: Value,
        dry_run: bool,
    ) -> Result<Value> {
        validate_entity_type(entity)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            return Ok(dry_run_response(plan_entity_create(
                &api_version,
                entity,
                body,
            )));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_create(&config, entity, &body).await
    }

    pub async fn entity_update(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        body: Value,
        dry_run: bool,
    ) -> Result<Value> {
        validate_entity_type(entity)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            return Ok(dry_run_response(plan_entity_update(
                &api_version,
                entity,
                id,
                body,
            )));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .entity_update(&config, entity, id, &body)
            .await
    }

    pub async fn entity_delete(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        dry_run: bool,
        yes: bool,
    ) -> Result<Value> {
        validate_entity_type(entity)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            return Ok(dry_run_response(plan_entity_delete(
                &api_version,
                entity,
                id,
            )));
        }

        if !yes {
            return Err(AppError::policy_blocked(
                "entity delete is a destructive operation; pass `--yes` to execute it, or use `--dry-run` to inspect the request plan first",
            )
            .with_operation("entity_delete")
            .with_hint("Add `--yes` to confirm the deletion, or `--dry-run` to preview the request without executing it."));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_delete(&config, entity, id).await
    }

    pub async fn entity_revive(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        dry_run: bool,
    ) -> Result<Value> {
        validate_entity_type(entity)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            return Ok(dry_run_response(plan_entity_revive(
                &api_version,
                entity,
                id,
            )));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_revive(&config, entity, id).await
    }

    pub async fn entity_relationships(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        related_field: &str,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport
            .entity_relationships(&config, entity, id, related_field, &params)
            .await
    }

    pub async fn entity_relationship_create(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        related_field: &str,
        body: Value,
    ) -> Result<Value> {
        validate_relationship_body(&body)?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .entity_relationship_create(&config, entity, id, related_field, &body)
            .await
    }

    pub async fn entity_relationship_update(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        related_field: &str,
        body: Value,
    ) -> Result<Value> {
        validate_relationship_body(&body)?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .entity_relationship_update(&config, entity, id, related_field, &body)
            .await
    }

    pub async fn entity_relationship_delete(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        related_field: &str,
        body: Value,
    ) -> Result<Value> {
        validate_relationship_body(&body)?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .entity_relationship_delete(&config, entity, id, related_field, &body)
            .await
    }

    pub async fn entity_share(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        body: Value,
    ) -> Result<Value> {
        validate_share_body(&body)?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .entity_share(&config, entity, id, &body)
            .await
    }

    pub async fn project_update_last_accessed(
        &self,
        overrides: ConnectionOverrides,
        project_id: u64,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .project_update_last_accessed(&config, project_id)
            .await
    }

    pub async fn text_search(&self, overrides: ConnectionOverrides, input: Value) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let payload = normalize_text_search_input(input)?;
        self.transport.text_search(&config, &payload).await
    }

    pub async fn entity_count(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Option<Value>,
        filter_dsl: Option<String>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let payload = build_count_payload(input, filter_dsl)?;
        self.transport
            .entity_summarize(&config, entity, &payload)
            .await
    }
}

fn normalize_text_search_input(input: Value) -> Result<Value> {
    let object = input.as_object().ok_or_else(|| {
        AppError::invalid_input("entity text-search input must be a JSON object")
            .with_operation("normalize_text_search_input")
            .with_expected_shape(
                "a JSON object containing `text` (search query string) and optional `entity_types`",
            )
    })?;

    if !object.contains_key("text") {
        return Err(AppError::invalid_input(
            "text-search requires a `text` field with the search query string",
        )
        .with_operation("normalize_text_search_input")
        .with_missing_fields(["text"])
        .with_expected_shape("a JSON object containing at least a `text` string field"));
    }

    let text = object.get("text").and_then(Value::as_str).ok_or_else(|| {
        AppError::invalid_input("`text` must be a non-empty string")
            .with_operation("normalize_text_search_input")
            .with_invalid_field("text")
            .with_expected_shape("a non-empty string")
    })?;

    if text.trim().is_empty() {
        return Err(AppError::invalid_input("`text` cannot be empty")
            .with_operation("normalize_text_search_input")
            .with_invalid_field("text")
            .with_hint("Provide a non-empty search query string."));
    }

    Ok(input)
}

/// Build an RPC `summarize` payload that counts records matching an
/// optional set of filters.
///
/// The returned JSON is suitable for passing directly to `entity_summarize`
/// and always requests `record_count` on the `id` field.  Filters can be
/// supplied via:
///
/// * `input` — a JSON object with an optional `filters` field (array or
///   object form) and an optional `filter_operator` field.
/// * `filter_dsl` — a DSL expression that is parsed into RPC-compatible
///   conditions.
///
/// When neither is provided the resulting payload uses an empty filter set
/// (count all records of the entity type).
fn build_count_payload(input: Option<Value>, filter_dsl: Option<String>) -> Result<Value> {
    let mut payload = Map::new();

    // Always request record_count on "id".
    payload.insert(
        "summaries".to_string(),
        json!([{"field": "id", "type": "record_count"}]),
    );

    // Determine filter source.
    let filters = match (input, filter_dsl) {
        (Some(_input), Some(_)) => {
            return Err(AppError::invalid_input(
                "`--input` and `--filter-dsl` are mutually exclusive for entity count",
            )
            .with_operation("build_count_payload")
            .with_conflicting_fields(["--input", "--filter-dsl"])
            .with_hint("Provide either `--input` with a JSON filter payload, or `--filter-dsl` with a DSL expression, but not both."));
        }
        (Some(input), None) => {
            let object = input.as_object().ok_or_else(|| {
                AppError::invalid_input("entity count input must be a JSON object")
                    .with_operation("build_count_payload")
                    .with_expected_shape(
                        "a JSON object containing optional `filters` and `filter_operator`",
                    )
            })?;
            let filters = object.get("filters").cloned().unwrap_or(json!([]));
            let filter_operator = object.get("filter_operator").cloned();
            normalize_filters(filters, filter_operator)?
        }
        (None, Some(dsl)) => {
            let parsed = parse_filter_dsl(&dsl)?;
            // The DSL parser returns a logical-group object with
            // `logical_operator` + `conditions` — translate to the RPC
            // `filter_operator` + `filters` shape.
            let operator = parsed
                .get("logical_operator")
                .and_then(Value::as_str)
                .map_or("all", |op| match op {
                    "and" => "all",
                    "or" => "any",
                    other => other,
                });
            let conditions = parsed.get("conditions").cloned().unwrap_or(json!([]));
            json!({
                "filter_operator": operator,
                "filters": conditions,
            })
        }
        (None, None) => {
            json!({
                "filter_operator": "all",
                "filters": [],
            })
        }
    };

    payload.insert("filters".to_string(), filters);
    Ok(Value::Object(payload))
}

/// Validate that the relationship write body is a JSON object containing a `data` field.
fn validate_relationship_body(body: &Value) -> Result<()> {
    let object = body.as_object().ok_or_else(|| {
        AppError::invalid_input("relationship write body must be a JSON object")
            .with_operation("validate_relationship_body")
            .with_expected_shape(
                "a JSON object containing `data` (array of entity link objects with `type` and `id`)",
            )
    })?;

    if !object.contains_key("data") {
        return Err(AppError::invalid_input(
            "relationship write body must contain a `data` field with an array of entity links",
        )
        .with_operation("validate_relationship_body")
        .with_missing_fields(["data"])
        .with_expected_shape(
            "a JSON object containing `data` (array of entity link objects with `type` and `id`)",
        ));
    }

    Ok(())
}

/// Validate that the entity share body is a JSON object containing `entities` or `projects`.
fn validate_share_body(body: &Value) -> Result<()> {
    body.as_object().ok_or_else(|| {
        AppError::invalid_input("entity share body must be a JSON object")
            .with_operation("validate_share_body")
            .with_expected_shape(
                "a JSON object describing the share target (e.g. containing `entities` or project links)",
            )
    })?;

    Ok(())
}

/// Validate that an entity type name is safe to use in a REST URL path.
///
/// Agents can hallucinate malformed entity names that would embed query
/// parameters (e.g. `"Shot?fields=code"`) or fragment identifiers
/// (e.g. `"Asset#section"`) into the path.  These are rejected here so that
/// the URL builder in `RestTransport` never silently builds a malformed request.
///
/// Control characters (anything below ASCII 0x20) are also rejected because
/// they cannot appear in a valid `ShotGrid` entity type name and may indicate
/// a prompt-injection attempt.
pub(crate) fn validate_entity_type(entity: &str) -> Result<()> {
    if entity.is_empty() {
        return Err(AppError::invalid_input("entity type name must not be empty")
            .with_operation("validate_entity_type")
            .with_invalid_field("entity")
            .with_hint("Provide a valid ShotGrid entity type name, such as `Shot`, `Asset`, or `Task`."));
    }

    if entity.contains('?') || entity.contains('#') {
        return Err(AppError::invalid_input(format!(
            "entity type name `{entity}` must not contain `?` or `#`; do not embed query parameters or fragment identifiers in entity names"
        ))
        .with_operation("validate_entity_type")
        .with_invalid_field("entity")
        .with_hint("Provide the plain entity type name without query parameters, for example `Shot` not `Shot?fields=code`."));
    }

    if entity.chars().any(|c| c.is_ascii_control()) {
        return Err(AppError::invalid_input(format!(
            "entity type name `{entity}` contains control characters (below ASCII 0x20), which are not permitted"
        ))
        .with_operation("validate_entity_type")
        .with_invalid_field("entity")
        .with_hint("Entity type names must contain only printable characters."));
    }

    Ok(())
}

/// Build a standard dry-run response wrapping a [`RequestPlan`].
///
/// Every entity write operation (`create`, `update`, `delete`, `revive`)
/// returns an identical `{ "dry_run": true, "plan": … }` envelope when
/// `--dry-run` is active.  This helper centralises that shape so that
/// changes to the dry-run format only need to be made in one place.
fn dry_run_response(plan: RequestPlan) -> Value {
    json!({
        "dry_run": true,
        "plan": plan,
    })
}
