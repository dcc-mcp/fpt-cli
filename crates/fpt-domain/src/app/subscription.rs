use fpt_core::{AppError, Result};
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    /// Read the subscription assigned to every user on the site.
    pub async fn subscription_user_list(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.subscription_user_list(&config).await
    }

    /// Assign subscription seats to users.
    ///
    /// The body is a map of user id to subscription name, for example
    /// `{"1554": "standard", "1588": "trial"}`.
    pub async fn subscription_user_assign(
        &self,
        overrides: ConnectionOverrides,
        body: Value,
    ) -> Result<Value> {
        validate_subscription_assignments(&body)?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .subscription_user_assign(&config, &body)
            .await
    }
}

/// Validate the user-subscription assignment payload.
///
/// The API expects a flat object keyed by user id whose values are subscription
/// names.  Rejecting anything else locally saves a round trip and gives agents a
/// precise, machine-readable error instead of a server-side 400.
fn validate_subscription_assignments(body: &Value) -> Result<()> {
    let object = body.as_object().ok_or_else(|| {
        AppError::invalid_input("subscription assign body must be a JSON object")
            .with_operation("subscription_user_assign")
            .with_expected_shape(
                "a JSON object mapping user ids to subscription names, e.g. {\"1554\": \"standard\"}",
            )
    })?;

    if object.is_empty() {
        return Err(
            AppError::invalid_input("subscription assign body cannot be empty")
                .with_operation("subscription_user_assign")
                .with_expected_shape(
                    "a JSON object with at least one user id mapped to a subscription name",
                )
                .with_hint("Provide at least one `\"<user_id>\": \"<subscription>\"` pair."),
        );
    }

    for (user_id, subscription) in object {
        if user_id.trim().is_empty() || !user_id.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(AppError::invalid_input(
                "subscription assign keys must be numeric user ids",
            )
            .with_operation("subscription_user_assign")
            .with_invalid_field("user_id")
            .with_received_value(user_id.clone())
            .with_expected_shape("a positive integer user id as a JSON object key"));
        }

        let is_string = subscription.as_str().is_some_and(|value| !value.is_empty());
        if !is_string {
            return Err(AppError::invalid_input(
                "subscription assign values must be non-empty subscription names",
            )
            .with_operation("subscription_user_assign")
            .with_invalid_field("subscription")
            .with_received_value(subscription.to_string())
            .with_expected_shape("a non-empty subscription name string"));
        }
    }

    Ok(())
}
