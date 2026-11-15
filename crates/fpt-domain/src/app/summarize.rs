use fpt_core::{AppError, Result};
use serde_json::{Map, Value};

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;
use super::query_helpers::normalize_filters;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn entity_summarize(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let payload = normalize_summarize_input(input)?;
        self.transport
            .entity_summarize(&config, entity, &payload)
            .await
    }

    /// Summarize entity records using the REST `_summarize` endpoint.
    ///
    /// This is the REST-native alternative to the legacy RPC `summarize` method.
    /// The request body is forwarded directly to the
    /// `POST /api/{ver}/entity/{entity}/_summarize` endpoint.
    pub async fn entity_summarize_rest(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let body = validate_summarize_rest_input(input)?;
        self.transport
            .entity_summarize_rest(&config, entity, &body)
            .await
    }
}

fn normalize_summarize_input(input: Value) -> Result<Value> {
    let Value::Object(mut object) = input else {
        return Err(
            AppError::invalid_input("entity summarize input must be a JSON object")
                .with_operation("normalize_summarize_input")
                .with_expected_shape(
                    "a JSON object containing `filters`, `summary_fields`, and optional `grouping`",
                ),
        );
    };

    let filters = object.remove("filters").ok_or_else(|| {
        AppError::invalid_input("entity summarize requires a `filters` field")
            .with_operation("normalize_summarize_input")
            .with_missing_fields(["filters"])
            .with_expected_shape("a JSON array or object of ShotGrid filter conditions")
    })?;
    let filter_operator = object.remove("filter_operator");
    let normalized_filters = normalize_filters(filters, filter_operator)?;

    let summary_fields = object.remove("summary_fields").ok_or_else(|| {
        AppError::invalid_input("entity summarize requires a `summary_fields` field")
            .with_operation("normalize_summarize_input")
            .with_missing_fields(["summary_fields"])
            .with_expected_shape(
                "a non-empty JSON array of summary descriptor objects with `field` and `type`",
            )
    })?;
    let summaries = normalize_summary_fields(summary_fields)?;

    let mut payload = Map::new();
    payload.insert("filters".to_string(), normalized_filters);
    payload.insert("summaries".to_string(), summaries);

    if let Some(grouping) = object.remove("grouping") {
        payload.insert("grouping".to_string(), normalize_grouping(grouping)?);
    }

    if let Some(include_archived_projects) = object.remove("include_archived_projects") {
        let include_archived_projects = include_archived_projects.as_bool().ok_or_else(|| {
            AppError::invalid_input("`include_archived_projects` must be a boolean")
        })?;
        if !include_archived_projects {
            payload.insert(
                "include_archived_projects".to_string(),
                Value::Bool(include_archived_projects),
            );
        }
    }

    if let Some(unexpected_key) = object.keys().next().cloned() {
        return Err(AppError::invalid_input(format!(
            "unsupported entity summarize field `{unexpected_key}`"
        ))
        .with_operation("normalize_summarize_input")
        .with_invalid_field(&unexpected_key)
        .with_allowed_values([
            "filters",
            "filter_operator",
            "summary_fields",
            "grouping",
            "include_archived_projects",
        ]));
    }

    Ok(Value::Object(payload))
}

fn normalize_summary_fields(summary_fields: Value) -> Result<Value> {
    let fields = summary_fields.as_array().ok_or_else(|| {
        AppError::invalid_input("`summary_fields` must be an array of summary descriptors")
            .with_operation("normalize_summary_fields")
            .with_invalid_field("summary_fields")
            .with_expected_shape("a non-empty JSON array of objects with `field` and `type`")
    })?;
    if fields.is_empty() {
        return Err(AppError::invalid_input("`summary_fields` cannot be empty")
            .with_operation("normalize_summary_fields")
            .with_invalid_field("summary_fields")
            .with_hint("Provide at least one summary descriptor object with `field` and `type`."));
    }

    let normalized = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            normalize_named_object(field, index, "summary_fields", &["field", "type"])
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Value::Array(normalized))
}

fn normalize_grouping(grouping: Value) -> Result<Value> {
    let grouping = grouping
        .as_array()
        .ok_or_else(|| AppError::invalid_input("`grouping` must be an array")
            .with_operation("normalize_grouping")
            .with_invalid_field("grouping")
            .with_expected_shape("a JSON array of grouping descriptor objects with `field`, `type`, and optional `direction`")
        )?;

    let normalized = grouping
        .iter()
        .enumerate()
        .map(|(index, group)| {
            normalize_named_object(group, index, "grouping", &["field", "type", "direction"])
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Value::Array(normalized))
}

fn normalize_named_object(
    value: &Value,
    index: usize,
    field_name: &str,
    allowed_keys: &[&str],
) -> Result<Value> {
    let object = value.as_object().ok_or_else(|| {
        AppError::invalid_input(format!("`{field_name}[{index}]` must be an object"))
            .with_operation("normalize_named_object")
            .with_invalid_field(field_name)
            .with_expected_shape("a JSON object")
    })?;

    for key in allowed_keys {
        if !object.contains_key(*key) && *key != "direction" {
            return Err(AppError::invalid_input(format!(
                "`{field_name}[{index}]` requires a `{key}` field"
            ))
            .with_operation("normalize_named_object")
            .with_missing_fields([*key]));
        }
    }

    if let Some(unexpected_key) = object
        .keys()
        .find(|key| !allowed_keys.contains(&key.as_str()))
        .cloned()
    {
        return Err(AppError::invalid_input(format!(
            "unsupported `{field_name}[{index}]` field `{unexpected_key}`"
        ))
        .with_operation("normalize_named_object")
        .with_invalid_field(&unexpected_key)
        .with_allowed_values(allowed_keys.iter().copied()));
    }

    let mut normalized = Map::new();
    for (key, value) in object {
        let value = value.as_str().ok_or_else(|| {
            AppError::invalid_input(format!("`{field_name}[{index}].{key}` must be a string"))
                .with_operation("normalize_named_object")
                .with_invalid_field(key)
                .with_expected_shape("a non-empty string")
        })?;
        if value.trim().is_empty() {
            return Err(AppError::invalid_input(format!(
                "`{field_name}[{index}].{key}` cannot be empty"
            ))
            .with_operation("normalize_named_object")
            .with_invalid_field(key)
            .with_hint("Provide a non-empty string value."));
        }
        normalized.insert(key.clone(), Value::String(value.to_string()));
    }

    Ok(Value::Object(normalized))
}

/// Validate the input for the REST `_summarize` endpoint.
///
/// The body must be a JSON object; beyond that, the server validates the
/// payload structure (summary_fields, filters, grouping, etc.).  This
/// function only rejects obviously malformed input so the CLI fails fast
/// with a clear message rather than forwarding garbage to the server.
fn validate_summarize_rest_input(input: Value) -> Result<Value> {
    if !input.is_object() {
        return Err(
            AppError::invalid_input("entity summarize-rest input must be a JSON object")
                .with_operation("validate_summarize_rest_input")
                .with_expected_shape(
                    "a JSON object containing `summary_fields`, `filters`, and optional `grouping`",
                ),
        );
    }
    Ok(input)
}
