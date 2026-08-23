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
    pub async fn schedule_work_day_rules(
        &self,
        overrides: ConnectionOverrides,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport
            .schedule_work_day_rules(&config, &params)
            .await
    }

    pub async fn schedule_work_day_rules_update(
        &self,
        overrides: ConnectionOverrides,
        rule_id: u64,
        body: Value,
    ) -> Result<Value> {
        validate_work_day_rule_body(&body)?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schedule_work_day_rules_update(&config, rule_id, &body)
            .await
    }

    pub async fn schedule_work_day_rules_create(
        &self,
        overrides: ConnectionOverrides,
        body: Value,
    ) -> Result<Value> {
        validate_work_day_rule_body(&body)?;
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schedule_work_day_rules_create(&config, &body)
            .await
    }

    pub async fn schedule_work_day_rules_read(
        &self,
        overrides: ConnectionOverrides,
        rule_id: u64,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schedule_work_day_rules_read(&config, rule_id)
            .await
    }

    pub async fn schedule_work_day_rules_delete(
        &self,
        overrides: ConnectionOverrides,
        rule_id: u64,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .schedule_work_day_rules_delete(&config, rule_id)
            .await
    }
}

fn validate_work_day_rule_body(body: &Value) -> Result<()> {
    body.as_object().ok_or_else(|| {
        AppError::invalid_input("work day rule update body must be a JSON object")
            .with_operation("schedule_work_day_rules_update")
            .with_expected_shape(
                "a JSON object containing work day rule fields such as `date`, `description`, or `is_working`",
            )
    })?;
    Ok(())
}
