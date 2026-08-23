use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use fpt_core::Result;
use fpt_domain::config::ConnectionSettings;
use fpt_domain::transport::{FindParams, ShotgridTransport, UploadUrlRequest};
use fpt_domain::{App, AuthMode, ConnectionOverrides};
use rstest::rstest;
use serde_json::{Value, json};

// ---------------------------------------------------------------
// Shared recording transport
// ---------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct RecordedState {
    find_calls: Vec<FindParams>,
}

#[derive(Debug, Clone, Default)]
struct RecordingTransport {
    state: Arc<Mutex<RecordedState>>,
}

impl RecordingTransport {
    fn snapshot(&self) -> RecordedState {
        self.state.lock().expect("state lock").clone()
    }
}

#[async_trait]
impl ShotgridTransport for RecordingTransport {
    async fn auth_test(&self, _: &ConnectionSettings) -> Result<Value> {
        Ok(json!({}))
    }
    async fn server_info(&self, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_entities(&self, _: &ConnectionSettings) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_fields(&self, _: &ConnectionSettings, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_get(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: Option<Vec<String>>,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_find(
        &self,
        _: &ConnectionSettings,
        _: &str,
        params: FindParams,
    ) -> Result<Value> {
        self.state.lock().expect("lock").find_calls.push(params);
        Ok(json!({ "data": [] }))
    }
    async fn entity_summarize(&self, _: &ConnectionSettings, _: &str, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_create(&self, _: &ConnectionSettings, _: &str, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_update(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_delete(&self, _: &ConnectionSettings, _: &str, _: u64) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_revive(&self, _: &ConnectionSettings, _: &str, _: u64) -> Result<Value> {
        Ok(json!({}))
    }
    async fn work_schedule_read(&self, _: &ConnectionSettings, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn upload_url(&self, _: &ConnectionSettings, _: UploadUrlRequest<'_>) -> Result<Value> {
        Ok(json!({}))
    }
    async fn download_url(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &str,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn thumbnail_url(&self, _: &ConnectionSettings, _: &str, _: u64) -> Result<Value> {
        Ok(json!({}))
    }
    async fn activity_stream(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn event_log_entries(
        &self,
        _: &ConnectionSettings,
        _: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn preferences_get(&self, _: &ConnectionSettings) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_followers(&self, _: &ConnectionSettings, _: &str, _: u64) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_follow(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_unfollow(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn note_threads(
        &self,
        _: &ConnectionSettings,
        _: u64,
        _: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_field_create(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_field_update(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: &str,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_field_delete(&self, _: &ConnectionSettings, _: &str, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn hierarchy(&self, _: &ConnectionSettings, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_field_read(&self, _: &ConnectionSettings, _: &str, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn work_schedule_update(&self, _: &ConnectionSettings, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn text_search(&self, _: &ConnectionSettings, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn note_reply_create(&self, _: &ConnectionSettings, _: u64, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_relationships(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &str,
        _: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn user_following(
        &self,
        _: &ConnectionSettings,
        _: u64,
        _: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn project_update_last_accessed(&self, _: &ConnectionSettings, _: u64) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_entity_read(&self, _: &ConnectionSettings, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_entity_update(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_entity_delete(&self, _: &ConnectionSettings, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_field_revive(&self, _: &ConnectionSettings, _: &str, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_entity_create(&self, _: &ConnectionSettings, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schema_entity_revive(&self, _: &ConnectionSettings, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn current_user(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn note_reply_read(
        &self,
        _: &ConnectionSettings,
        _: u64,
        _: u64,
        _: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn filmstrip_thumbnail(&self, _: &ConnectionSettings, _: &str, _: u64) -> Result<Value> {
        Ok(json!({}))
    }
    async fn preferences_update(&self, _: &ConnectionSettings, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn note_reply_update(
        &self,
        _: &ConnectionSettings,
        _: u64,
        _: u64,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn note_reply_delete(&self, _: &ConnectionSettings, _: u64, _: u64) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_relationship_create(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &str,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_relationship_update(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &str,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_share(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn entity_relationship_delete(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &str,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn hierarchy_expand(&self, _: &ConnectionSettings, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules(
        &self,
        _: &ConnectionSettings,
        _: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules_update(
        &self,
        _: &ConnectionSettings,
        _: u64,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn license(&self, _: &ConnectionSettings) -> Result<Value> {
        Ok(json!({}))
    }
    async fn preferences_custom_entity(&self, _: &ConnectionSettings, _: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules_create(
        &self,
        _: &ConnectionSettings,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules_delete(
        &self,
        _: &ConnectionSettings,
        _: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules_read(&self, _: &ConnectionSettings, _: u64) -> Result<Value> {
        Ok(json!({}))
    }
    async fn thumbnail_upload(
        &self,
        _: &ConnectionSettings,
        _: &str,
        _: u64,
        _: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn rest_api_version(&self, _: &str, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn openapi_spec(&self, _: &str, _: &str, _: &str) -> Result<Value> {
        Ok(json!({}))
    }
}

fn test_overrides() -> ConnectionOverrides {
    ConnectionOverrides {
        site: Some("https://test.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::SessionToken),
        session_token: Some("test-token".to_string()),
        ..Default::default()
    }
}

// ---------------------------------------------------------------
// Issue #44: structured search consistency
// ---------------------------------------------------------------

#[rstest]
#[case::search_with_array_filters(
    json!({
        "search": {
            "filters": [["code", "is", "test"]]
        }
    }),
    json!({
        "filter_operator": "all",
        "filters": [["code", "is", "test"]]
    })
)]
#[case::search_with_object_filters(
    json!({
        "search": {
            "filters": {
                "filter_operator": "any",
                "filters": [["code", "is", "test"], ["id", "greater_than", 100]]
            }
        }
    }),
    json!({
        "filter_operator": "any",
        "filters": [["code", "is", "test"], ["id", "greater_than", 100]]
    })
)]
#[case::search_without_filters_gets_default(
    json!({
        "search": {
            "fields": ["code", "id"]
        }
    }),
    json!({
        "filter_operator": "all",
        "filters": []
    })
)]
#[case::search_with_nested_logical_group(
    json!({
        "search": {
            "filters": [
                ["sg_status_list", "is", "ip"],
                {
                    "logical_operator": "or",
                    "conditions": [
                        ["code", "contains", "bunny"],
                        ["id", "greater_than", 100]
                    ]
                }
            ]
        }
    }),
    json!({
        "filter_operator": "all",
        "filters": [
            ["sg_status_list", "is", "ip"],
            {
                "logical_operator": "or",
                "conditions": [
                    ["code", "contains", "bunny"],
                    ["id", "greater_than", 100]
                ]
            }
        ]
    })
)]
fn search_input_produces_expected_search_body(
    #[case] input: Value,
    #[case] expected_filters: Value,
) {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());
    let overrides = test_overrides();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let _ = app.entity_find(overrides, "Shot", Some(input), None).await;
    });

    let state = transport.snapshot();
    let params = state
        .find_calls
        .last()
        .expect("should have recorded a find call");
    let search = params
        .search
        .as_ref()
        .expect("search body should be present");
    assert_eq!(
        search.get("filters").unwrap(),
        &expected_filters,
        "search body filters mismatch"
    );
}

#[test]
fn search_input_must_be_object() {
    let transport = RecordingTransport::default();
    let app = App::new(transport);
    let overrides = test_overrides();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        app.entity_find(
            overrides,
            "Shot",
            Some(json!({ "search": "not-an-object" })),
            None,
        )
        .await
    });

    assert!(result.is_err());
    let msg = result.unwrap_err().envelope().message;
    assert!(msg.contains("search"), "error should mention search: {msg}");
}

#[rstest]
#[case::filter_dsl_only("code ~ 'test'")]
#[case::filter_dsl_with_entity_link("project is {\"type\": \"Project\", \"id\": 123}")]
fn filter_dsl_produces_search_body(#[case] dsl: &str) {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());
    let overrides = test_overrides();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let _ = app
            .entity_find(overrides, "Shot", None, Some(dsl.to_string()))
            .await;
    });

    let state = transport.snapshot();
    let params = state
        .find_calls
        .last()
        .expect("should have recorded a find call");
    assert!(
        params.search.is_some(),
        "filter_dsl should produce a search body"
    );
    let search = params.search.as_ref().unwrap();
    assert!(
        search.get("filters").is_some(),
        "search body should contain filters"
    );
}

#[test]
fn filter_dsl_and_search_are_mutually_exclusive() {
    let transport = RecordingTransport::default();
    let app = App::new(transport);
    let overrides = test_overrides();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        app.entity_find(
            overrides,
            "Shot",
            Some(json!({
                "search": {"filters": [["code", "is", "test"]]},
                "filter_dsl": "code ~ 'other'"
            })),
            None,
        )
        .await
    });

    assert!(result.is_err());
    let msg = result.unwrap_err().envelope().message;
    assert!(
        msg.contains("cannot be used together") || msg.contains("mutually exclusive"),
        "error should explain mutual exclusion: {msg}"
    );
}

// ---------------------------------------------------------------
// Consistency: find-one uses the same params as find
// ---------------------------------------------------------------

#[test]
fn find_one_uses_search_body_same_as_find() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());
    let overrides = test_overrides();

    let input = json!({
        "search": {
            "filters": [["code", "is", "hero_shot"]]
        }
    });

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let _ = app
            .entity_find_one(overrides, "Shot", Some(input), None)
            .await;
    });

    let state = transport.snapshot();
    let params = state
        .find_calls
        .last()
        .expect("should have recorded a find call");
    assert!(
        params.search.is_some(),
        "find_one should also produce a search body"
    );
    // find_one should add page[size]=1
    let has_page_size = params
        .query
        .iter()
        .any(|(k, v)| k == "page[size]" && v == "1");
    assert!(has_page_size, "find_one should limit page size to 1");
}

// ---------------------------------------------------------------
// Consistency: batch find uses the same normalization
// ---------------------------------------------------------------

#[test]
fn batch_find_normalizes_search_same_as_find() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());
    let overrides = test_overrides();

    let input = json!([
        {
            "search": {
                "filters": [["code", "is", "test"]]
            }
        }
    ]);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let _ = app.entity_batch_find(overrides, "Shot", input).await;
    });

    let state = transport.snapshot();
    assert!(
        !state.find_calls.is_empty(),
        "batch find should have called entity_find"
    );
    let params = state.find_calls.last().unwrap();
    assert!(
        params.search.is_some(),
        "batch find should produce search body"
    );
    let search = params.search.as_ref().unwrap();
    assert!(
        search.get("filters").is_some(),
        "batch find search body should contain filters"
    );
}
