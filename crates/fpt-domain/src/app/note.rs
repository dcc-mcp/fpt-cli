use fpt_core::{AppError, ErrorCode, Result};
use serde_json::{Value, json};

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;
use super::query_helpers::build_query_params;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn note_threads(
        &self,
        overrides: ConnectionOverrides,
        note_id: u64,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport
            .note_threads(&config, note_id, &params)
            .await
            .map_err(|error| translate_note_threads_error(error, note_id))
    }

    pub async fn note_reply_create(
        &self,
        overrides: ConnectionOverrides,
        note_id: u64,
        body: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        validate_note_reply_body(&body)?;
        self.transport
            .note_reply_create(&config, note_id, &body)
            .await
    }

    pub async fn note_reply_read(
        &self,
        overrides: ConnectionOverrides,
        note_id: u64,
        reply_id: u64,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport
            .note_reply_read(&config, note_id, reply_id, &params)
            .await
    }

    pub async fn note_reply_update(
        &self,
        overrides: ConnectionOverrides,
        note_id: u64,
        reply_id: u64,
        body: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        validate_note_reply_body(&body)?;
        self.transport
            .note_reply_update(&config, note_id, reply_id, &body)
            .await
    }

    pub async fn note_reply_delete(
        &self,
        overrides: ConnectionOverrides,
        note_id: u64,
        reply_id: u64,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .note_reply_delete(&config, note_id, reply_id)
            .await
    }
}

fn translate_note_threads_error(error: AppError, note_id: u64) -> AppError {
    let envelope = error.envelope();
    let not_found = envelope.code == ErrorCode::ApiError.as_str()
        && envelope
            .details
            .as_ref()
            .and_then(|details| details.get("errors"))
            .and_then(Value::as_array)
            .is_some_and(|errors| {
                errors.iter().any(|entry| {
                    entry.get("status") == Some(&json!(404))
                        && entry
                            .get("detail")
                            .and_then(Value::as_str)
                            .is_some_and(|detail| {
                                detail.contains(&format!("Note: {note_id} not found"))
                            })
                })
            });

    if not_found {
        return AppError::api(format!(
            "Note thread lookup failed: `{note_id}` is not a top-level Note record id or the Note does not exist"
        ))
        .with_operation("note_threads")
        .with_transport(envelope.transport.unwrap_or_else(|| "rest".to_string()))
        .with_resource(format!("Note/{note_id}"))
        .with_detail("note_id", note_id)
        .with_hint("Verify that the id belongs to a top-level Note entity, not a reply or a different entity type.")
        .with_details(
            envelope
                .details
                .unwrap_or_else(|| json!({ "note_id": note_id })),
        );
    }

    error
}

fn validate_note_reply_body(body: &Value) -> Result<()> {
    let object = body.as_object().ok_or_else(|| {
        AppError::invalid_input("note reply body must be a JSON object")
            .with_operation("note_reply_create")
            .with_expected_shape(
                "a JSON object containing `content` (string) and optional `type` for the reply",
            )
    })?;

    if !object.contains_key("content") {
        return Err(AppError::invalid_input(
            "note reply body must contain a `content` field with the reply text",
        )
        .with_operation("note_reply_create")
        .with_missing_fields(["content"])
        .with_expected_shape("a JSON object containing at least a `content` string field"));
    }

    Ok(())
}
