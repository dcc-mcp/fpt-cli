use fpt_core::{AppError, Result};
use serde_json::Value;

/// Normalize a `filters` value and an optional `filter_operator` into the
/// canonical RPC shape expected by the `ShotGrid` REST API.
///
/// Accepts either:
/// * A JSON array of filter conditions (wrapped with the resolved operator,
///   defaulting to `"all"`)
/// * A JSON object that already contains `filter_operator` and `filters` keys
///
/// The optional `filter_operator` (when supplied at the top level) is validated
/// and merged into the result.  It is an error to specify `filter_operator`
/// both at the top level **and** inside a `filters` object.
pub(crate) fn normalize_filters(filters: Value, filter_operator: Option<Value>) -> Result<Value> {
    match filters {
        Value::Array(items) => {
            let filter_operator = normalize_filter_operator(filter_operator.as_ref())?
                .unwrap_or_else(|| "all".to_string());
            Ok(serde_json::json!({
                "filter_operator": filter_operator,
                "filters": items,
            }))
        }
        Value::Object(mut object) => {
            let filter_operator = normalize_filter_operator(filter_operator.as_ref())?;
            if let Some(filter_operator) = filter_operator {
                if object.contains_key("filter_operator") {
                    return Err(AppError::invalid_input(
                        "`filter_operator` cannot be set both at the top level and inside `filters`",
                    )
                    .with_operation("normalize_filters")
                    .with_conflicting_fields(["filter_operator", "filters.filter_operator"]));
                }
                object.insert(
                    "filter_operator".to_string(),
                    Value::String(filter_operator),
                );
            }
            Ok(Value::Object(object))
        }
        _ => Err(AppError::invalid_input(
            "`filters` must be a JSON array or object",
        )
        .with_operation("normalize_filters")
        .with_invalid_field("filters")
        .with_expected_shape("a JSON array of filter conditions, or a JSON object with `filter_operator` and `filters`")),
    }
}

/// Validate and normalize an optional `filter_operator` value.
///
/// Returns `Ok(None)` when the operator is absent, `Ok(Some("all" | "any"))`
/// when it is valid, or an error for any other value.
fn normalize_filter_operator(filter_operator: Option<&Value>) -> Result<Option<String>> {
    let Some(filter_operator) = filter_operator else {
        return Ok(None);
    };

    let filter_operator = filter_operator.as_str().ok_or_else(|| {
        AppError::invalid_input("`filter_operator` must be `all` or `any`")
            .with_operation("normalize_filter_operator")
            .with_invalid_field("filter_operator")
            .with_allowed_values(["all", "any"])
    })?;

    match filter_operator {
        "all" | "any" => Ok(Some(filter_operator.to_string())),
        _ => Err(
            AppError::invalid_input("`filter_operator` must be `all` or `any`")
                .with_operation("normalize_filter_operator")
                .with_invalid_field("filter_operator")
                .with_received_value(filter_operator)
                .with_allowed_values(["all", "any"]),
        ),
    }
}

/// Convert an optional JSON object into a flat list of query string key-value
/// pairs.
///
/// This builder is used by `activity_stream`, `event_log_entries`,
/// `entity_relationships`, `user_following`, and `note_threads` — any endpoint
/// that accepts optional query parameters in a uniform shape.
///
/// Supported shapes:
/// - `{"fields": "id,event_type"}` or `{"fields": ["id", "event_type"]}`
/// - `{"sort": "-created_at"}`
/// - `{"page": {"number": 1, "size": 50}}`
/// - `{"entity_fields": "code,sg_status_list"}`
/// - Any other top-level string/number/bool scalar key is forwarded as-is.
pub(crate) fn build_query_params(input: Option<Value>) -> Result<Vec<(String, String)>> {
    let Some(input) = input else {
        return Ok(Vec::new());
    };

    let object = input.as_object().ok_or_else(|| {
        AppError::invalid_input("query input must be a JSON object containing query parameters")
            .with_operation("build_query_params")
            .with_expected_shape(
                "a JSON object containing fields like `fields`, `entity_fields`, `sort`, or `page`",
            )
    })?;

    let mut params: Vec<(String, String)> = Vec::new();

    for (key, value) in object {
        match key.as_str() {
            "page" => {
                push_page_params(&mut params, value, "build_query_params")?;
            }
            "fields" | "entity_fields" => {
                let joined = string_list_to_csv(value, key)?;
                if !joined.is_empty() {
                    params.push((key.clone(), joined));
                }
            }
            _ => {
                params.push((key.clone(), scalar_to_string(value, key)?));
            }
        }
    }

    Ok(params)
}

/// Parse a `page` JSON value and push `page[number]` / `page[size]` query
/// parameters into the given list.
///
/// The `page` value must be a JSON object like `{"number": 1, "size": 25}`.
/// Both `number` and `size` are optional; when present they are converted via
/// [`scalar_to_string`].
///
/// `operation` is the caller's operation name used in error messages so that
/// validation errors point back to the right context.
pub(crate) fn push_page_params(
    params: &mut Vec<(String, String)>,
    page: &Value,
    operation: &str,
) -> Result<()> {
    let page = page.as_object().ok_or_else(|| {
        AppError::invalid_input("`page` must be a JSON object like `{\"number\": 1, \"size\": 25}`")
            .with_operation(operation)
            .with_invalid_field("page")
            .with_expected_shape("a JSON object like `{\"number\": 1, \"size\": 25}`")
    })?;
    if let Some(number) = page.get("number") {
        params.push((
            "page[number]".to_string(),
            scalar_to_string(number, "page.number")?,
        ));
    }
    if let Some(size) = page.get("size") {
        params.push((
            "page[size]".to_string(),
            scalar_to_string(size, "page.size")?,
        ));
    }
    Ok(())
}

/// Convert a JSON scalar (string, number, or bool) to its string representation.
///
/// Used for query-string parameter encoding where all values must be strings.
pub(crate) fn scalar_to_string(value: &Value, field_name: &str) -> Result<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => Err(AppError::invalid_input(format!(
            "`{field_name}` must be a scalar query value of type string, number, or boolean"
        ))
        .with_invalid_field(field_name)
        .with_expected_shape("a scalar value of type string, number, or boolean")),
    }
}

/// Convert a JSON value that is either a comma-separated string or an array of
/// strings into a single comma-joined string.
///
/// This is the variant used by activity and note query parameter builders.
/// Delegates to [`string_list`] for the actual parsing, then joins with `","`.
fn string_list_to_csv(value: &Value, field_name: &str) -> Result<String> {
    Ok(string_list(value, field_name)?.join(","))
}

/// Parse a JSON value into a `Vec<String>`.
///
/// Accepts either a comma-separated string or a JSON array of strings.
/// Empty entries are filtered out.
pub(crate) fn string_list(value: &Value, field_name: &str) -> Result<Vec<String>> {
    if let Some(value) = value.as_str() {
        let items = value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect();
        return Ok(items);
    }

    let array = value.as_array().ok_or_else(|| {
        AppError::invalid_input(format!(
            "`{field_name}` must be either a comma-separated string or an array of strings"
        ))
        .with_invalid_field(field_name)
        .with_expected_shape("either a comma-separated string or an array of strings")
    })?;

    array
        .iter()
        .map(|value| {
            value.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` array items must all be strings"))
                    .with_invalid_field(field_name)
                    .with_expected_shape("an array containing only strings")
            })
        })
        .collect()
}
