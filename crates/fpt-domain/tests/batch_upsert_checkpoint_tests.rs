//! Tests for the resumable bulk upsert workflow (issue #43).
//!
//! These tests validate that:
//! - A checkpoint file is created and populated during upsert
//! - A resumed upsert skips already-completed items
//! - The `--resume` flag without `--checkpoint` is rejected
//! - Partial checkpoint files (e.g. with malformed lines) are tolerated
//! - Stats include `resumed_skip_count` when resuming

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use fpt_core::{AppError, Result};
use fpt_domain::app::batch::OnConflict;
use fpt_domain::transport::{FindParams, ShotgridTransport, UploadUrlRequest};
use fpt_domain::{App, AuthMode, ConnectionOverrides};

use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn overrides() -> ConnectionOverrides {
    ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::Script),
        script_name: Some("openclaw".to_string()),
        script_key: Some("secret-key".to_string()),
        api_version: Some("v1.1".to_string()),
        ..Default::default()
    }
}

/// A mock transport that can simulate existing entities for upsert lookups.
///
/// When `entity_find` is called, it checks `existing_by_key` to decide whether
/// to return an empty data array (no match) or a pre-set entity record.
///
/// `entity_create` and `entity_update` record their calls for later assertion.
#[derive(Debug, Clone)]
struct UpsertTransport {
    /// Maps `key_value` (as JSON string) → existing entity record.
    existing_by_key: Arc<HashMap<String, Value>>,
    create_calls: Arc<Mutex<Vec<Value>>>,
    update_calls: Arc<Mutex<Vec<(u64, Value)>>>,
    find_calls: Arc<Mutex<Vec<FindParams>>>,
}

impl UpsertTransport {
    fn new(existing: HashMap<String, Value>) -> Self {
        Self {
            existing_by_key: Arc::new(existing),
            create_calls: Arc::default(),
            update_calls: Arc::default(),
            find_calls: Arc::default(),
        }
    }

    fn create_count(&self) -> usize {
        self.create_calls.lock().unwrap().len()
    }

    fn update_count(&self) -> usize {
        self.update_calls.lock().unwrap().len()
    }

    fn find_count(&self) -> usize {
        self.find_calls.lock().unwrap().len()
    }
}

#[async_trait]
impl ShotgridTransport for UpsertTransport {
    async fn auth_test(&self, _config: &fpt_domain::ConnectionSettings) -> Result<Value> {
        Ok(json!({"ok": true}))
    }

    async fn server_info(&self, _site: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entities(&self, _config: &fpt_domain::ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_fields(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_get(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _fields: Option<Vec<String>>,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_find(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        params: FindParams,
    ) -> Result<Value> {
        self.find_calls.lock().unwrap().push(params.clone());

        // Try to extract the key value from the search body to match existing records.
        let data = if let Some(search) = &params.search {
            let filters = search
                .get("filters")
                .and_then(|f| f.get("filters"))
                .and_then(|f| f.as_array());
            if let Some(conditions) = filters {
                // Look for [key, "is", value] pattern
                let mut found = None;
                for cond in conditions {
                    if let Some(arr) = cond.as_array() {
                        if arr.len() == 3 {
                            let lookup_key = serde_json::to_string(&arr[2]).unwrap_or_default();
                            if let Some(existing) = self.existing_by_key.get(&lookup_key) {
                                found = Some(existing.clone());
                                break;
                            }
                        }
                    }
                }
                match found {
                    Some(entity) => json!([entity]),
                    None => json!([]),
                }
            } else {
                json!([])
            }
        } else {
            json!([])
        };

        Ok(json!({"data": data}))
    }

    async fn entity_summarize(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_create(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        body: &Value,
    ) -> Result<Value> {
        self.create_calls.lock().unwrap().push(body.clone());
        Ok(json!({"data": {"id": 9000, "type": "Asset"}}))
    }

    async fn entity_update(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value> {
        self.update_calls.lock().unwrap().push((id, body.clone()));
        Ok(json!({"data": {"id": id, "type": "Asset"}}))
    }

    async fn entity_delete(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_revive(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn work_schedule_read(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn upload_url(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _request: UploadUrlRequest<'_>,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn download_url(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn thumbnail_url(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn activity_stream(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn event_log_entries(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn preferences_get(&self, _config: &fpt_domain::ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_followers(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_follow(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _user: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_unfollow(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _user: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_threads(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _note_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_create(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_update(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _field_name: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_delete(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn hierarchy(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_read(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn work_schedule_update(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn text_search(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_create(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _note_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationships(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn user_following(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _user_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn project_update_last_accessed(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _project_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_read(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_update(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_delete(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_revive(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_create(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_revive(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn current_user(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _user_type: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_read(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn filmstrip_thumbnail(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn preferences_update(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_update(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_delete(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationship_create(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationship_update(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_share(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationship_delete(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn hierarchy_expand(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schedule_work_day_rules(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schedule_work_day_rules_update(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _rule_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn license(&self, _config: &fpt_domain::ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn preferences_custom_entity(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schedule_work_day_rules_create(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schedule_work_day_rules_delete(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _rule_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schedule_work_day_rules_read(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _rule_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn thumbnail_upload(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn rest_api_version(&self, _site: &str, _api_version: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn openapi_spec(&self, _site: &str, _api_version: &str, _format: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_summarize_rest(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_batch_server(
        &self,
        _config: &fpt_domain::ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upsert_creates_checkpoint_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let checkpoint_path = dir.path().join("checkpoint.jsonl");
    let checkpoint_str = checkpoint_path.to_string_lossy().to_string();

    let transport = UpsertTransport::new(HashMap::new());
    let app = App::new(transport.clone());

    let input = json!([
        {"code": "ALPHA", "description": "first"},
        {"code": "BETA", "description": "second"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Skip,
            false,
            Some(checkpoint_str.clone()),
            false,
        )
        .await
        .expect("upsert succeeds");

    // Both items should be created (no existing records)
    assert_eq!(result["created_count"], 2);
    assert_eq!(result["success_count"], 2);
    assert_eq!(result["failure_count"], 0);
    assert_eq!(result["resumed_skip_count"], 0);
    assert_eq!(result["checkpoint"], checkpoint_str);

    // The checkpoint file should exist and contain 2 JSONL lines
    let content = std::fs::read_to_string(&checkpoint_path).expect("read checkpoint");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2, "checkpoint should have 2 lines");

    for line in &lines {
        let entry: Value = serde_json::from_str(line).expect("valid JSONL");
        assert_eq!(entry["ok"], true);
        assert_eq!(entry["action"], "created");
    }

    // Verify transport calls
    assert_eq!(transport.create_count(), 2);
    assert_eq!(transport.find_count(), 2);
    assert_eq!(transport.update_count(), 0);
}

#[tokio::test]
async fn upsert_resumes_from_checkpoint() {
    let dir = tempfile::tempdir().expect("temp dir");
    let checkpoint_path = dir.path().join("checkpoint.jsonl");
    let checkpoint_str = checkpoint_path.to_string_lossy().to_string();

    // Pre-seed a checkpoint file with index 0 already completed.
    std::fs::write(
        &checkpoint_path,
        r#"{"index":0,"ok":true,"action":"created","request":{"code":"ALPHA"}}"#.to_string() + "\n",
    )
    .expect("write checkpoint");

    let transport = UpsertTransport::new(HashMap::new());
    let app = App::new(transport.clone());

    let input = json!([
        {"code": "ALPHA", "description": "first"},
        {"code": "BETA", "description": "second"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Skip,
            false,
            Some(checkpoint_str.clone()),
            true, // resume
        )
        .await
        .expect("resume upsert succeeds");

    // Index 0 was already in the checkpoint → resumed_skip
    // Index 1 is new → created
    assert_eq!(result["total"], 2);
    assert_eq!(result["resumed_skip_count"], 1);
    assert_eq!(result["created_count"], 1);
    assert_eq!(result["success_count"], 2);
    assert_eq!(result["failure_count"], 0);

    // Only index 1 should have triggered a find + create
    assert_eq!(
        transport.find_count(),
        1,
        "only non-skipped item should be queried"
    );
    assert_eq!(
        transport.create_count(),
        1,
        "only non-skipped item should be created"
    );
}

#[tokio::test]
async fn upsert_resume_without_checkpoint_is_error() {
    let transport = UpsertTransport::new(HashMap::new());
    let app = App::new(transport);

    let input = json!([{"code": "ALPHA"}]);

    let err = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Skip,
            false,
            None, // no checkpoint
            true, // resume
        )
        .await
        .expect_err("resume without checkpoint should fail");

    let msg = format!("{err}");
    assert!(
        msg.contains("--resume"),
        "error should mention --resume flag: {msg}"
    );
    assert!(
        msg.contains("--checkpoint"),
        "error should mention --checkpoint flag: {msg}"
    );
}

#[tokio::test]
async fn upsert_resume_with_missing_checkpoint_file_is_error() {
    let transport = UpsertTransport::new(HashMap::new());
    let app = App::new(transport);

    let input = json!([{"code": "ALPHA"}]);

    let err = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Skip,
            false,
            Some("nonexistent_file.jsonl".to_string()),
            true, // resume
        )
        .await
        .expect_err("resume with missing file should fail");

    let msg = format!("{err}");
    assert!(
        msg.contains("does not exist"),
        "error should indicate file does not exist: {msg}"
    );
}

#[tokio::test]
async fn upsert_resume_tolerates_malformed_checkpoint_lines() {
    let dir = tempfile::tempdir().expect("temp dir");
    let checkpoint_path = dir.path().join("checkpoint.jsonl");
    let checkpoint_str = checkpoint_path.to_string_lossy().to_string();

    // Write a checkpoint with one valid line, one malformed line, and one empty line
    let content = [
        r#"{"index":0,"ok":true,"action":"created"}"#,
        "this is not valid json",
        "",
        r#"{"index":2,"ok":true,"action":"updated"}"#,
    ]
    .join("\n");
    std::fs::write(&checkpoint_path, content).expect("write checkpoint");

    let transport = UpsertTransport::new(HashMap::new());
    let app = App::new(transport.clone());

    let input = json!([
        {"code": "A"},
        {"code": "B"},
        {"code": "C"},
        {"code": "D"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Skip,
            false,
            Some(checkpoint_str),
            true, // resume
        )
        .await
        .expect("upsert with partial checkpoint succeeds");

    // Indices 0 and 2 were in the checkpoint → resumed_skip
    // Indices 1 and 3 are new → created
    assert_eq!(result["total"], 4);
    assert_eq!(result["resumed_skip_count"], 2);
    assert_eq!(result["created_count"], 2);
    assert_eq!(result["success_count"], 4);
    assert_eq!(result["failure_count"], 0);

    // Only 2 items should hit the transport
    assert_eq!(transport.find_count(), 2);
    assert_eq!(transport.create_count(), 2);
}

#[tokio::test]
async fn upsert_with_on_conflict_update_and_checkpoint() {
    let dir = tempfile::tempdir().expect("temp dir");
    let checkpoint_path = dir.path().join("checkpoint.jsonl");
    let checkpoint_str = checkpoint_path.to_string_lossy().to_string();

    // "EXISTING" is already in ShotGrid
    let mut existing = HashMap::new();
    existing.insert(
        r#""EXISTING""#.to_string(),
        json!({"id": 42, "type": "Asset", "code": "EXISTING"}),
    );

    let transport = UpsertTransport::new(existing);
    let app = App::new(transport.clone());

    let input = json!([
        {"code": "NEW_ITEM", "description": "brand new"},
        {"code": "EXISTING", "description": "updated description"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Update,
            false,
            Some(checkpoint_str.clone()),
            false,
        )
        .await
        .expect("upsert succeeds");

    assert_eq!(result["total"], 2);
    assert_eq!(result["created_count"], 1);
    assert_eq!(result["updated_count"], 1);
    assert_eq!(result["success_count"], 2);
    assert_eq!(result["failure_count"], 0);

    // Verify checkpoint file
    let content = std::fs::read_to_string(&checkpoint_path).expect("read checkpoint");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2);

    // Verify transport calls
    assert_eq!(transport.create_count(), 1);
    assert_eq!(transport.update_count(), 1);
    assert_eq!(transport.find_count(), 2);
}

#[tokio::test]
async fn upsert_with_on_conflict_skip_and_existing_entity() {
    let mut existing = HashMap::new();
    existing.insert(
        r#""EXISTING""#.to_string(),
        json!({"id": 42, "type": "Asset", "code": "EXISTING"}),
    );

    let transport = UpsertTransport::new(existing);
    let app = App::new(transport.clone());

    let input = json!([
        {"code": "EXISTING", "description": "should be skipped"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Skip,
            false,
            None,
            false,
        )
        .await
        .expect("upsert succeeds");

    assert_eq!(result["total"], 1);
    assert_eq!(result["skipped_count"], 1);
    assert_eq!(result["created_count"], 0);
    assert_eq!(result["updated_count"], 0);

    // No create or update should have been called
    assert_eq!(transport.create_count(), 0);
    assert_eq!(transport.update_count(), 0);
    // But find should have been called to check for existing
    assert_eq!(transport.find_count(), 1);
}

#[tokio::test]
async fn upsert_with_on_conflict_error_and_existing_entity() {
    let mut existing = HashMap::new();
    existing.insert(
        r#""EXISTING""#.to_string(),
        json!({"id": 42, "type": "Asset", "code": "EXISTING"}),
    );

    let transport = UpsertTransport::new(existing);
    let app = App::new(transport.clone());

    let input = json!([
        {"code": "EXISTING", "description": "should cause conflict"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Error,
            false,
            None,
            false,
        )
        .await
        .expect("upsert completes (with item-level error)");

    assert_eq!(result["total"], 1);
    assert_eq!(result["failure_count"], 1);
    assert_eq!(result["ok"], false);

    let item = &result["results"][0];
    assert_eq!(item["ok"], false);
    assert_eq!(item["action"], "conflict");
    assert!(
        item["error"]["code"]
            .as_str()
            .unwrap()
            .contains("POLICY_BLOCKED"),
        "conflict should use POLICY_BLOCKED code"
    );
}

#[tokio::test]
async fn upsert_missing_key_field_reports_item_error() {
    let transport = UpsertTransport::new(HashMap::new());
    let app = App::new(transport.clone());

    let input = json!([
        {"code": "OK_ITEM"},
        {"description": "missing code field"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Skip,
            false,
            None,
            false,
        )
        .await
        .expect("upsert completes");

    assert_eq!(result["total"], 2);
    assert_eq!(result["success_count"], 1);
    assert_eq!(result["failure_count"], 1);

    // The second item (index=1) should have the error about missing key field
    let results = result["results"].as_array().unwrap();
    let error_item = results.iter().find(|r| r["index"] == 1).unwrap();
    assert_eq!(error_item["ok"], false);
    assert_eq!(error_item["action"], "skipped");
    assert!(
        error_item["error"]["message"]
            .as_str()
            .unwrap()
            .contains("code"),
        "error should mention the missing key field name"
    );
}

#[tokio::test]
async fn upsert_dry_run_does_not_create_checkpoint() {
    let dir = tempfile::tempdir().expect("temp dir");
    let checkpoint_path = dir.path().join("checkpoint.jsonl");
    let checkpoint_str = checkpoint_path.to_string_lossy().to_string();

    let transport = UpsertTransport::new(HashMap::new());
    let app = App::new(transport.clone());

    let input = json!([
        {"code": "ALPHA"},
        {"code": "BETA"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Skip,
            true, // dry_run
            Some(checkpoint_str),
            false,
        )
        .await
        .expect("dry run succeeds");

    assert_eq!(result["dry_run"], true);
    assert_eq!(result["count"], 2);

    // Dry run should not create a checkpoint file
    assert!(
        !checkpoint_path.exists(),
        "dry run should not create checkpoint file"
    );

    // Dry run should not call any transport methods
    assert_eq!(transport.find_count(), 0);
    assert_eq!(transport.create_count(), 0);
}

#[tokio::test]
async fn upsert_without_checkpoint_flag_works() {
    let transport = UpsertTransport::new(HashMap::new());
    let app = App::new(transport.clone());

    let input = json!([
        {"code": "SOLO"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Skip,
            false,
            None,  // no checkpoint
            false, // no resume
        )
        .await
        .expect("upsert without checkpoint succeeds");

    assert_eq!(result["total"], 1);
    assert_eq!(result["created_count"], 1);
    assert_eq!(result["checkpoint"], Value::Null);
}

#[tokio::test]
async fn upsert_checkpoint_records_all_actions() {
    let dir = tempfile::tempdir().expect("temp dir");
    let checkpoint_path = dir.path().join("checkpoint.jsonl");
    let checkpoint_str = checkpoint_path.to_string_lossy().to_string();

    // One item exists, one is new, one has missing key
    let mut existing = HashMap::new();
    existing.insert(
        r#""EXISTING""#.to_string(),
        json!({"id": 10, "type": "Asset", "code": "EXISTING"}),
    );

    let transport = UpsertTransport::new(existing);
    let app = App::new(transport);

    let input = json!([
        {"code": "NEW_ITEM"},
        {"code": "EXISTING", "description": "update me"},
        {"description": "no code field"},
    ]);

    let result = app
        .entity_batch_upsert(
            overrides(),
            "Asset",
            input,
            "code",
            OnConflict::Update,
            false,
            Some(checkpoint_str.clone()),
            false,
        )
        .await
        .expect("upsert succeeds");

    assert_eq!(result["total"], 3);

    // Read checkpoint — should have 3 entries (one per item)
    let content = std::fs::read_to_string(&checkpoint_path).expect("read checkpoint");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 3, "checkpoint should record all 3 items");

    // Parse each line and verify actions are recorded
    let entries: Vec<Value> = lines
        .iter()
        .map(|l| serde_json::from_str(l).expect("valid JSONL"))
        .collect();

    let actions: Vec<&str> = entries
        .iter()
        .map(|e| e["action"].as_str().unwrap_or("unknown"))
        .collect();

    assert!(actions.contains(&"created"), "should have a created action");
    assert!(
        actions.contains(&"updated"),
        "should have an updated action"
    );
    assert!(
        actions.contains(&"skipped"),
        "should have a skipped action (missing key)"
    );
}
