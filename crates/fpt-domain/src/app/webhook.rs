use fpt_core::{AppError, Result};
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;
use super::query_helpers::build_query_params;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    /// List the webhooks registered on the site.
    ///
    /// Optional `--input` JSON is converted into query parameters, which is how
    /// the API accepts `status`, `url`, and pagination options.
    pub async fn webhook_hooks_list(
        &self,
        overrides: ConnectionOverrides,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport.webhook_hooks_list(&config, &params).await
    }

    /// Register a new webhook on the site.
    pub async fn webhook_hook_create(
        &self,
        overrides: ConnectionOverrides,
        body: Value,
    ) -> Result<Value> {
        validate_json_object(&body, "webhook_hook_create")?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.webhook_hook_create(&config, &body).await
    }

    /// Read a single webhook by its record id.
    pub async fn webhook_hook_read(
        &self,
        overrides: ConnectionOverrides,
        record_uuid: &str,
    ) -> Result<Value> {
        let record_uuid = require_identifier(record_uuid, "hook id", "webhook_hook_read")?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.webhook_hook_read(&config, record_uuid).await
    }

    /// Update an existing webhook by its record id.
    pub async fn webhook_hook_update(
        &self,
        overrides: ConnectionOverrides,
        record_uuid: &str,
        body: Value,
    ) -> Result<Value> {
        let record_uuid = require_identifier(record_uuid, "hook id", "webhook_hook_update")?;
        validate_json_object(&body, "webhook_hook_update")?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .webhook_hook_update(&config, record_uuid, &body)
            .await
    }

    /// Delete a webhook by its record id.
    ///
    /// This is irreversible — the webhook stops receiving deliveries immediately.
    pub async fn webhook_hook_delete(
        &self,
        overrides: ConnectionOverrides,
        record_uuid: &str,
    ) -> Result<Value> {
        let record_uuid = require_identifier(record_uuid, "hook id", "webhook_hook_delete")?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .webhook_hook_delete(&config, record_uuid)
            .await
    }

    /// Trigger a test delivery for a webhook to verify its target endpoint.
    pub async fn webhook_hook_test_connection(
        &self,
        overrides: ConnectionOverrides,
        record_uuid: &str,
    ) -> Result<Value> {
        let record_uuid =
            require_identifier(record_uuid, "hook id", "webhook_hook_test_connection")?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .webhook_hook_test_connection(&config, record_uuid)
            .await
    }

    /// List the deliveries recorded for a webhook.
    ///
    /// Optional `--input` JSON is converted into query parameters such as
    /// `status`, `from`, `to`, `entity_type`, and pagination options.
    pub async fn webhook_deliveries_list(
        &self,
        overrides: ConnectionOverrides,
        hook_id: &str,
        input: Option<Value>,
    ) -> Result<Value> {
        let hook_id = require_identifier(hook_id, "hook id", "webhook_deliveries_list")?;
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport
            .webhook_deliveries_list(&config, hook_id, &params)
            .await
    }

    /// Read a single webhook delivery by its record id.
    pub async fn webhook_delivery_read(
        &self,
        overrides: ConnectionOverrides,
        record_uuid: &str,
    ) -> Result<Value> {
        let record_uuid = require_identifier(record_uuid, "delivery id", "webhook_delivery_read")?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .webhook_delivery_read(&config, record_uuid)
            .await
    }

    /// Update a webhook delivery, for example to acknowledge a failed delivery.
    pub async fn webhook_delivery_update(
        &self,
        overrides: ConnectionOverrides,
        record_uuid: &str,
        body: Value,
    ) -> Result<Value> {
        let record_uuid =
            require_identifier(record_uuid, "delivery id", "webhook_delivery_update")?;
        validate_json_object(&body, "webhook_delivery_update")?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .webhook_delivery_update(&config, record_uuid, &body)
            .await
    }

    /// Re-deliver a webhook delivery that previously failed.
    pub async fn webhook_delivery_redeliver(
        &self,
        overrides: ConnectionOverrides,
        record_uuid: &str,
    ) -> Result<Value> {
        let record_uuid =
            require_identifier(record_uuid, "delivery id", "webhook_delivery_redeliver")?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .webhook_delivery_redeliver(&config, record_uuid)
            .await
    }
}

/// Validate that `body` is a JSON object, returning a structured error otherwise.
///
/// Webhook and delivery write endpoints always expect a JSON object payload, so
/// arrays and scalars are rejected before a network call is made.
fn validate_json_object(body: &Value, operation: &str) -> Result<()> {
    body.as_object().ok_or_else(|| {
        AppError::invalid_input(format!("{operation} requires a JSON object body"))
            .with_operation(operation)
            .with_expected_shape("a JSON object describing the fields to write")
    })?;
    Ok(())
}

/// Validate that a caller-supplied record identifier is a non-empty string.
///
/// Webhook and delivery identifiers are UUIDs, but the CLI accepts any
/// non-empty string so future identifier formats keep working.  Whitespace-only
/// values are rejected because they would produce an invalid request path.
fn require_identifier<'a>(value: &'a str, label: &str, operation: &str) -> Result<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid_input(format!("{label} cannot be empty"))
            .with_operation(operation)
            .with_invalid_field(label)
            .with_expected_shape("a non-empty webhook or delivery record id")
            .with_hint("Pass the record id returned by the webhook list command."));
    }
    Ok(trimmed)
}
