use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use async_trait::async_trait;
use fpt_core::{AppError, Result};
use fpt_domain::transport::{FindParams, ShotgridTransport, UploadUrlRequest};
use fpt_domain::{App, AuthMode, ConnectionOverrides, ConnectionSettings};

use serde_json::{Value, json};
use tokio::time::sleep;

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityGetCall {
    entity: String,
    id: u64,
    fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
struct EntityUpdateCall {
    entity: String,
    id: u64,
    body: Value,
}

#[derive(Debug, Clone, Default)]
struct RecordedState {
    auth_calls: usize,
    server_info_sites: Vec<String>,
    schema_entities_calls: usize,
    schema_fields_entities: Vec<String>,
    entity_get_calls: Vec<EntityGetCall>,
    entity_find_calls: Vec<FindParams>,
    entity_summarize_calls: Vec<(String, Value)>,
    entity_create_calls: Vec<(String, Value)>,

    entity_update_calls: Vec<EntityUpdateCall>,
    entity_delete_calls: Vec<(String, u64)>,
    entity_revive_calls: Vec<(String, u64)>,
    work_schedule_calls: Vec<Value>,
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
    async fn auth_test(&self, config: &ConnectionSettings) -> Result<Value> {
        self.state.lock().expect("state lock").auth_calls += 1;
        Ok(json!({
            "ok": true,
            "site": config.site,
            "grant_type": config.auth_mode().grant_type(),
        }))
    }

    async fn server_info(&self, site: &str) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .server_info_sites
            .push(site.to_string());
        Ok(json!({"site": site, "full_version": [8, 40, 0, 0]}))
    }

    async fn schema_entities(&self, _config: &ConnectionSettings) -> Result<Value> {
        self.state.lock().expect("state lock").schema_entities_calls += 1;
        Ok(json!({"data": ["Asset", "Shot"]}))
    }

    async fn schema_fields(&self, _config: &ConnectionSettings, entity: &str) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .schema_fields_entities
            .push(entity.to_string());
        Ok(json!({"entity": entity, "fields": ["code", "sg_status_list"]}))
    }

    async fn entity_get(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        fields: Option<Vec<String>>,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_get_calls
            .push(EntityGetCall {
                entity: entity.to_string(),
                id,
                fields: fields.clone(),
            });
        Ok(json!({"entity": entity, "id": id, "fields": fields}))
    }

    async fn entity_find(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        params: FindParams,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_find_calls
            .push(params.clone());
        Ok(json!({
            "query": params.query,
            "search": params.search,
        }))
    }

    async fn entity_summarize(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_summarize_calls
            .push((entity.to_string(), body.clone()));
        Ok(json!({
            "entity": entity,
            "summaries": body["summaries"].clone(),
            "groups": body.get("grouping").cloned().unwrap_or_else(|| json!([])),
        }))
    }

    async fn entity_create(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_create_calls
            .push((entity.to_string(), body.clone()));
        if body.get("fail").and_then(Value::as_bool) == Some(true) {
            return Err(AppError::api("create failed"));
        }
        Ok(json!({"entity": entity, "body": body}))
    }

    async fn entity_update(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_update_calls
            .push(EntityUpdateCall {
                entity: entity.to_string(),
                id,
                body: body.clone(),
            });
        if id == 99 {
            return Err(AppError::api("update failed"));
        }
        Ok(json!({"entity": entity, "id": id, "body": body}))
    }

    async fn entity_delete(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_delete_calls
            .push((entity.to_string(), id));
        if id == 13 {
            return Err(AppError::api("delete failed"));
        }
        Ok(json!({"entity": entity, "id": id, "deleted": true}))
    }

    async fn entity_revive(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_revive_calls
            .push((entity.to_string(), id));
        Ok(json!(true))
    }

    async fn work_schedule_read(
        &self,
        _config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .work_schedule_calls
            .push(body.clone());
        Ok(json!({
            "2026-03-16": {
                "working": true,
                "reason": "STUDIO_WORK_WEEK"
            }
        }))
    }

    async fn upload_url(
        &self,
        _config: &ConnectionSettings,
        request: UploadUrlRequest<'_>,
    ) -> Result<Value> {
        Ok(json!({
            "entity": request.entity,
            "id": request.id,
            "field_name": request.field_name,
            "file_name": request.file_name
        }))
    }

    async fn download_url(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        field_name: &str,
    ) -> Result<Value> {
        Ok(
            json!({"entity": entity, "id": id, "field_name": field_name, "download_url": "https://example.com/file"}),
        )
    }

    async fn thumbnail_url(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "thumbnail_url": "https://example.com/thumb.jpg"}))
    }

    async fn activity_stream(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "data": []}))
    }

    async fn event_log_entries(
        &self,
        _config: &ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({"data": []}))
    }

    async fn preferences_get(&self, _config: &ConnectionSettings) -> Result<Value> {
        Ok(json!({"preferences": {}}))
    }

    async fn entity_followers(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "followers": []}))
    }

    async fn entity_follow(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "user": user}))
    }

    async fn entity_unfollow(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "user": user, "unfollowed": true}))
    }

    async fn note_threads(
        &self,
        _config: &ConnectionSettings,
        note_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({"note_id": note_id, "data": []}))
    }

    async fn schema_field_create(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "body": body}))
    }

    async fn schema_field_update(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "field_name": field_name, "body": body}))
    }

    async fn schema_field_delete(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "field_name": field_name, "deleted": true}))
    }

    async fn hierarchy(&self, _config: &ConnectionSettings, body: &Value) -> Result<Value> {
        Ok(json!({"data": body}))
    }

    async fn schema_field_read(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "field_name": field_name, "data_type": "text"}))
    }

    async fn work_schedule_update(
        &self,
        _config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"updated": true, "body": body}))
    }

    async fn text_search(&self, _config: &ConnectionSettings, body: &Value) -> Result<Value> {
        Ok(json!({"data": [], "text": body["text"].clone()}))
    }

    async fn note_reply_create(
        &self,
        _config: &ConnectionSettings,
        note_id: u64,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"note_id": note_id, "content": body["content"].clone(), "type": "Reply"}))
    }

    async fn entity_relationships(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "related_field": related_field, "data": []}))
    }

    async fn user_following(
        &self,
        _config: &ConnectionSettings,
        user_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({"user_id": user_id, "data": []}))
    }

    async fn project_update_last_accessed(
        &self,
        _config: &ConnectionSettings,
        project_id: u64,
    ) -> Result<Value> {
        Ok(json!({"project_id": project_id, "ok": true}))
    }

    async fn schema_entity_read(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "data": {}}))
    }

    async fn schema_entity_update(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "body": body, "updated": true}))
    }

    async fn schema_entity_delete(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "deleted": true}))
    }

    async fn schema_field_revive(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "field_name": field_name, "revived": true}))
    }

    async fn schema_entity_create(
        &self,
        _config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"body": body, "created": true}))
    }

    async fn schema_entity_revive(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "revived": true}))
    }

    async fn current_user(
        &self,
        _config: &ConnectionSettings,
        user_type: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({"user_type": user_type, "id": 1, "name": "Current User"}))
    }

    async fn note_reply_read(
        &self,
        _config: &ConnectionSettings,
        note_id: u64,
        reply_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(
            json!({"note_id": note_id, "reply_id": reply_id, "content": "Reply content", "type": "Reply"}),
        )
    }

    async fn filmstrip_thumbnail(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        Ok(
            json!({"entity": entity, "id": id, "filmstrip_url": "https://example.com/filmstrip.jpg"}),
        )
    }

    async fn preferences_update(
        &self,
        _config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"ok": true, "preferences": body}))
    }

    async fn note_reply_update(
        &self,
        _config: &ConnectionSettings,
        note_id: u64,
        reply_id: u64,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"note_id": note_id, "reply_id": reply_id, "updated": body}))
    }

    async fn note_reply_delete(
        &self,
        _config: &ConnectionSettings,
        note_id: u64,
        reply_id: u64,
    ) -> Result<Value> {
        Ok(json!({"ok": true, "note_id": note_id, "reply_id": reply_id, "status": 200}))
    }

    async fn entity_relationship_create(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "related_field": related_field, "created": body}))
    }

    async fn entity_relationship_update(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "related_field": related_field, "updated": body}))
    }

    async fn entity_relationship_delete(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "related_field": related_field, "deleted": body}))
    }

    async fn entity_share(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "shared": body}))
    }

    async fn hierarchy_expand(&self, _config: &ConnectionSettings, body: &Value) -> Result<Value> {
        Ok(json!({"data": body}))
    }

    async fn schedule_work_day_rules(
        &self,
        _config: &ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({"data": []}))
    }

    async fn schedule_work_day_rules_update(
        &self,
        _config: &ConnectionSettings,
        rule_id: u64,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"rule_id": rule_id, "updated": true, "body": body}))
    }

    async fn license(&self, _config: &ConnectionSettings) -> Result<Value> {
        Ok(json!({"data": {"current_users": 10, "max_users": 50}}))
    }

    async fn preferences_custom_entity(
        &self,
        _config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"ok": true, "enabled": body}))
    }

    async fn schedule_work_day_rules_create(
        &self,
        _config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"created": true, "body": body}))
    }

    async fn schedule_work_day_rules_delete(
        &self,
        _config: &ConnectionSettings,
        rule_id: u64,
    ) -> Result<Value> {
        Ok(json!({"deleted": true, "rule_id": rule_id}))
    }

    async fn schedule_work_day_rules_read(
        &self,
        _config: &ConnectionSettings,
        rule_id: u64,
    ) -> Result<Value> {
        Ok(json!({"id": rule_id}))
    }

    async fn thumbnail_upload(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value> {
        Ok(json!({"entity": entity, "id": id, "uploaded": true, "body": body}))
    }

    async fn rest_api_version(&self, _site: &str, _api_version: &str) -> Result<Value> {
        Ok(json!({"version": "v1.1"}))
    }

    async fn openapi_spec(&self, _site: &str, _api_version: &str, _format: &str) -> Result<Value> {
        Ok(json!({"format": _format}))
    }

    async fn entity_summarize_rest(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_batch_server(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
}

#[derive(Debug, Clone)]
struct FindOneTransport {
    response: Value,
    calls: Arc<Mutex<Vec<FindParams>>>,
}

impl FindOneTransport {
    fn new(response: Value) -> Self {
        Self {
            response,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn calls(&self) -> Vec<FindParams> {
        self.calls.lock().expect("calls lock").clone()
    }
}

#[async_trait]
impl ShotgridTransport for FindOneTransport {
    async fn auth_test(&self, _config: &ConnectionSettings) -> Result<Value> {
        Ok(json!({"ok": true}))
    }

    async fn server_info(&self, _site: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entities(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_fields(&self, _config: &ConnectionSettings, _entity: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_get(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _fields: Option<Vec<String>>,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_find(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        params: FindParams,
    ) -> Result<Value> {
        self.calls.lock().expect("calls lock").push(params);
        Ok(self.response.clone())
    }

    async fn entity_summarize(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn work_schedule_read(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn upload_url(
        &self,
        _config: &ConnectionSettings,
        _request: UploadUrlRequest<'_>,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn download_url(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn thumbnail_url(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn activity_stream(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn event_log_entries(
        &self,
        _config: &ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn preferences_get(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_followers(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_follow(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _user: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_unfollow(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _user: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_threads(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn hierarchy(&self, _config: &ConnectionSettings, _body: &Value) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_field_read(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn work_schedule_update(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn text_search(&self, _config: &ConnectionSettings, _body: &Value) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_create(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationships(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn user_following(
        &self,
        _config: &ConnectionSettings,
        _user_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn project_update_last_accessed(
        &self,
        _config: &ConnectionSettings,
        _project_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_read(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_create(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn current_user(
        &self,
        _config: &ConnectionSettings,
        _user_type: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_read(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn filmstrip_thumbnail(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn preferences_update(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_update(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_delete(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationship_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationship_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationship_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_share(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn hierarchy_expand(&self, _config: &ConnectionSettings, _body: &Value) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules(
        &self,
        _config: &ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules_update(
        &self,
        _config: &ConnectionSettings,
        _rule_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn license(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn preferences_custom_entity(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules_create(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules_delete(
        &self,
        _config: &ConnectionSettings,
        _rule_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules_read(
        &self,
        _config: &ConnectionSettings,
        _rule_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn thumbnail_upload(
        &self,
        _config: &ConnectionSettings,
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
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_batch_server(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
}

#[derive(Debug, Clone, Default)]
struct NoteThreadsNotFoundTransport;

#[async_trait]
impl ShotgridTransport for NoteThreadsNotFoundTransport {
    async fn auth_test(&self, _config: &ConnectionSettings) -> Result<Value> {
        Ok(json!({"ok": true}))
    }

    async fn server_info(&self, _site: &str) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_entities(&self, _config: &ConnectionSettings) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_fields(&self, _config: &ConnectionSettings, _entity: &str) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_get(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _fields: Option<Vec<String>>,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_find(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _params: FindParams,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_summarize(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn work_schedule_read(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn upload_url(
        &self,
        _config: &ConnectionSettings,
        _request: UploadUrlRequest<'_>,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn download_url(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _field_name: &str,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn thumbnail_url(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn activity_stream(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn event_log_entries(
        &self,
        _config: &ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn preferences_get(&self, _config: &ConnectionSettings) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_followers(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_follow(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _user: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_unfollow(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _user: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn note_threads(
        &self,
        _config: &ConnectionSettings,
        note_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::api("ShotGrid API request failed (404 Not Found)")
            .with_transport("rest")
            .with_details(json!({
                "errors": [
                    {
                        "status": 404,
                        "detail": format!("Note: {note_id} not found")
                    }
                ]
            })))
    }

    async fn schema_field_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_field_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_field_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn hierarchy(&self, _config: &ConnectionSettings, _body: &Value) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_field_read(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn work_schedule_update(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn text_search(&self, _config: &ConnectionSettings, _body: &Value) -> Result<Value> {
        Ok(json!({}))
    }

    async fn note_reply_create(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_relationships(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn user_following(
        &self,
        _config: &ConnectionSettings,
        _user_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn project_update_last_accessed(
        &self,
        _config: &ConnectionSettings,
        _project_id: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_entity_read(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_entity_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_entity_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_field_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_entity_create(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn schema_entity_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn current_user(
        &self,
        _config: &ConnectionSettings,
        _user_type: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn note_reply_read(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn filmstrip_thumbnail(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn preferences_update(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn note_reply_update(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn note_reply_delete(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_relationship_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_relationship_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_share(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_relationship_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn hierarchy_expand(&self, _config: &ConnectionSettings, _body: &Value) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules(
        &self,
        _config: &ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules_update(
        &self,
        _config: &ConnectionSettings,
        _rule_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn license(&self, _config: &ConnectionSettings) -> Result<Value> {
        Ok(json!({}))
    }
    async fn preferences_custom_entity(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules_create(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules_delete(
        &self,
        _config: &ConnectionSettings,
        _rule_id: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn schedule_work_day_rules_read(
        &self,
        _config: &ConnectionSettings,
        _rule_id: u64,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn thumbnail_upload(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
    async fn rest_api_version(&self, _site: &str, _api_version: &str) -> Result<Value> {
        Ok(json!({}))
    }
    async fn openapi_spec(&self, _site: &str, _api_version: &str, _format: &str) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_summarize_rest(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }

    async fn entity_batch_server(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Ok(json!({}))
    }
}

#[derive(Debug, Clone, Default)]
struct SlowGetTransport {
    current: Arc<AtomicUsize>,
    max_seen: Arc<AtomicUsize>,
}

impl SlowGetTransport {
    fn max_in_flight(&self) -> usize {
        self.max_seen.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl ShotgridTransport for SlowGetTransport {
    async fn auth_test(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn server_info(&self, _site: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entities(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_fields(&self, _config: &ConnectionSettings, _entity: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_get(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        _fields: Option<Vec<String>>,
    ) -> Result<Value> {
        let in_flight = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_seen.fetch_max(in_flight, Ordering::SeqCst);

        let delay_ms = match id {
            1 => 80,
            2 => 10,
            3 => 60,
            _ => 20,
        };
        sleep(Duration::from_millis(delay_ms)).await;

        self.current.fetch_sub(1, Ordering::SeqCst);
        Ok(json!({"entity": entity, "id": id}))
    }

    async fn entity_find(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _params: FindParams,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_summarize(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn work_schedule_read(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn upload_url(
        &self,
        _config: &ConnectionSettings,
        _request: UploadUrlRequest<'_>,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn download_url(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn thumbnail_url(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn activity_stream(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn event_log_entries(
        &self,
        _config: &ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn preferences_get(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_followers(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_follow(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _user: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_unfollow(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _user: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_threads(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn hierarchy(&self, _config: &ConnectionSettings, _body: &Value) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_read(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn work_schedule_update(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn text_search(&self, _config: &ConnectionSettings, _body: &Value) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_create(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationships(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn user_following(
        &self,
        _config: &ConnectionSettings,
        _user_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn project_update_last_accessed(
        &self,
        _config: &ConnectionSettings,
        _project_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_read(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_field_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _field_name: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_create(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entity_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn current_user(
        &self,
        _config: &ConnectionSettings,
        _user_type: &str,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_read(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn filmstrip_thumbnail(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn preferences_update(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_update(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn note_reply_delete(
        &self,
        _config: &ConnectionSettings,
        _note_id: u64,
        _reply_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationship_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationship_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_share(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_relationship_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _related_field: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn hierarchy_expand(&self, _config: &ConnectionSettings, _body: &Value) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules(
        &self,
        _config: &ConnectionSettings,
        _params: &[(String, String)],
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules_update(
        &self,
        _config: &ConnectionSettings,
        _rule_id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn license(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn preferences_custom_entity(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules_create(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules_delete(
        &self,
        _config: &ConnectionSettings,
        _rule_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn schedule_work_day_rules_read(
        &self,
        _config: &ConnectionSettings,
        _rule_id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
    async fn thumbnail_upload(
        &self,
        _config: &ConnectionSettings,
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
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_batch_server(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
}

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

fn query_value<'a>(params: &'a FindParams, key: &str) -> Option<&'a str> {
    params
        .query
        .iter()
        .find(|(item_key, _)| item_key == key)
        .map(|(_, value)| value.as_str())
}

#[tokio::test]
async fn auth_schema_and_entity_read_commands_delegate_to_transport() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    let auth = app
        .auth_test(overrides())
        .await
        .expect("auth test succeeds");
    let server_info = app
        .server_info(ConnectionOverrides {
            site: overrides().site,
            ..Default::default()
        })
        .await
        .expect("server info succeeds");
    let schema_entities = app
        .schema_entities(overrides())
        .await
        .expect("schema entities succeeds");
    let schema_fields = app
        .schema_fields(overrides(), "Shot")
        .await
        .expect("schema fields succeeds");
    let entity_get = app
        .entity_get(
            overrides(),
            "Shot",
            42,
            Some(vec!["code".to_string(), "sg_status_list".to_string()]),
        )
        .await
        .expect("entity get succeeds");
    let summarize = app
        .entity_summarize(
            overrides(),
            "Version",
            json!({
                "filters": [["sg_status_list", "is", "ip"]],
                "filter_operator": "any",
                "summary_fields": [
                    { "field": "id", "type": "record_count" }
                ],
                "grouping": [
                    { "field": "project", "type": "exact", "direction": "asc" }
                ],
                "include_archived_projects": false
            }),
        )
        .await
        .expect("entity summarize succeeds");
    let work_schedule = app
        .work_schedule_read(
            overrides(),
            json!({
                "start_date": "2026-03-16",
                "end_date": "2026-03-20",
                "project": { "type": "Project", "id": 123 },
                "user": { "type": "HumanUser", "id": 456 }
            }),
        )
        .await
        .expect("work schedule read succeeds");

    assert_eq!(auth["grant_type"], "client_credentials");
    assert_eq!(server_info["site"], "https://example.shotgrid.autodesk.com");
    assert_eq!(schema_entities["data"][0], "Asset");
    assert_eq!(schema_fields["entity"], "Shot");
    assert_eq!(entity_get["id"], 42);
    assert_eq!(summarize["entity"], "Version");
    assert_eq!(summarize["summaries"][0]["type"], "record_count");
    assert_eq!(work_schedule["2026-03-16"]["working"], true);

    let snapshot = transport.snapshot();
    assert_eq!(snapshot.auth_calls, 1);
    assert_eq!(
        snapshot.server_info_sites,
        vec!["https://example.shotgrid.autodesk.com".to_string()]
    );
    assert_eq!(snapshot.schema_entities_calls, 1);
    assert_eq!(snapshot.schema_fields_entities, vec!["Shot".to_string()]);
    assert_eq!(snapshot.entity_get_calls.len(), 1);
    assert_eq!(snapshot.entity_summarize_calls.len(), 1);
    assert_eq!(snapshot.work_schedule_calls.len(), 1);
    assert_eq!(
        snapshot.entity_summarize_calls[0],
        (
            "Version".to_string(),
            json!({
                "filters": {
                    "filter_operator": "any",
                    "filters": [["sg_status_list", "is", "ip"]]
                },
                "summaries": [
                    { "field": "id", "type": "record_count" }
                ],
                "grouping": [
                    { "field": "project", "type": "exact", "direction": "asc" }
                ],
                "include_archived_projects": false
            })
        )
    );
    assert_eq!(
        snapshot.work_schedule_calls[0],
        json!({
            "start_date": "2026-03-16",
            "end_date": "2026-03-20",
            "project": { "type": "Project", "id": 123 },
            "user": { "type": "HumanUser", "id": 456 }
        })
    );

    assert_eq!(
        snapshot.entity_get_calls[0],
        EntityGetCall {
            entity: "Shot".to_string(),
            id: 42,
            fields: Some(vec!["code".to_string(), "sg_status_list".to_string()]),
        }
    );
}

#[tokio::test]
async fn entity_find_builds_query_and_search_payload_from_input() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    app.entity_find(
        overrides(),
        "Asset",
        Some(json!({
            "fields": ["code", "sg_status_list"],
            "include": "project",
            "sort": "code",
            "page": { "number": 2, "size": 50 },
            "options": { "include_archived_projects": true },
            "query": { "extra": "keep" },
            "filter_dsl": "code ~ 'bunny'"
        })),
        None,
    )
    .await
    .expect("entity find succeeds");

    let snapshot = transport.snapshot();
    let params = snapshot
        .entity_find_calls
        .first()
        .expect("find params recorded");
    assert_eq!(query_value(params, "fields"), Some("code,sg_status_list"));
    assert_eq!(query_value(params, "include"), Some("project"));
    assert_eq!(query_value(params, "sort"), Some("code"));
    assert_eq!(query_value(params, "page[number]"), Some("2"));
    assert_eq!(query_value(params, "page[size]"), Some("50"));
    assert_eq!(
        query_value(params, "options[include_archived_projects]"),
        Some("true")
    );
    assert_eq!(query_value(params, "extra"), Some("keep"));
    assert!(params.search.is_some());
}

#[tokio::test]
async fn entity_find_accepts_structured_search_input() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    app.entity_find(
        overrides(),
        "Version",
        Some(json!({
            "fields": ["code", "sg_status_list"],
            "search": {
                "filters": {
                    "filter_operator": "all",
                    "filters": [
                        ["project", "is", {"type": "Project", "id": 123}],
                        ["sg_status_list", "is", "ip"]
                    ]
                },
                "additional_filter_presets": [
                    {
                        "preset_name": "LATEST",
                        "latest_by": "ENTITIES_CREATED_AT"
                    }
                ]
            }
        })),
        None,
    )
    .await
    .expect("structured search should succeed");

    let snapshot = transport.snapshot();
    let params = snapshot
        .entity_find_calls
        .first()
        .expect("find params recorded");
    assert_eq!(query_value(params, "fields"), Some("code,sg_status_list"));
    assert_eq!(
        params.search,
        Some(json!({
            "filters": {
                "filter_operator": "all",
                "filters": [
                    ["project", "is", {"type": "Project", "id": 123}],
                    ["sg_status_list", "is", "ip"]
                ]
            },
            "additional_filter_presets": [
                {
                    "preset_name": "LATEST",
                    "latest_by": "ENTITIES_CREATED_AT"
                }
            ]
        }))
    );
}

#[tokio::test]
async fn entity_find_merges_top_level_presets_with_filter_dsl() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    app.entity_find(
        overrides(),
        "Version",
        Some(json!({
            "additional_filter_presets": [
                {
                    "preset_name": "LATEST",
                    "latest_by": "ENTITIES_CREATED_AT"
                }
            ]
        })),
        Some("code ~ 'hero'".to_string()),
    )
    .await
    .expect("filter presets should merge with filter_dsl");

    let snapshot = transport.snapshot();
    let params = snapshot
        .entity_find_calls
        .first()
        .expect("find params recorded");
    assert_eq!(
        params.search,
        Some(json!({
            "additional_filter_presets": [
                {
                    "preset_name": "LATEST",
                    "latest_by": "ENTITIES_CREATED_AT"
                }
            ],
            "filters": {
                "logical_operator": "and",
                "conditions": [
                    ["code", "contains", "hero"]
                ]
            }
        }))
    );
}

#[tokio::test]
async fn entity_find_rejects_conflicting_filter_inputs() {
    let app = App::new(RecordingTransport::default());

    let conflicting_filters = app
        .entity_find(
            overrides(),
            "Asset",
            Some(json!({
                "filters": { "id": 42 },
                "filter_dsl": "id == 42"
            })),
            None,
        )
        .await
        .expect_err("conflicting filters should be rejected");
    let conflicting_search = app
        .entity_find(
            overrides(),
            "Asset",
            Some(json!({
                "search": { "filters": [] },
                "filter_dsl": "id == 42"
            })),
            None,
        )
        .await
        .expect_err("search and filter_dsl should conflict");
    let duplicate_presets = app
        .entity_find(
            overrides(),
            "Asset",
            Some(json!({
                "search": {
                    "additional_filter_presets": [{ "preset_name": "LATEST" }]
                },
                "additional_filter_presets": [{ "preset_name": "LATEST" }]
            })),
            None,
        )
        .await
        .expect_err("duplicate presets should be rejected");

    assert_eq!(conflicting_filters.envelope().code, "INVALID_INPUT");
    assert_eq!(conflicting_search.envelope().code, "INVALID_INPUT");
    assert_eq!(duplicate_presets.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_find_one_returns_first_match_and_forces_single_page() {
    let transport = FindOneTransport::new(json!({
        "data": [
            {"id": 7, "type": "Shot", "code": "sh010"},
            {"id": 8, "type": "Shot", "code": "sh020"}
        ]
    }));
    let app = App::new(transport.clone());

    let response = app
        .entity_find_one(
            overrides(),
            "Shot",
            Some(json!({
                "fields": ["code"],
                "page": { "number": 3, "size": 50 },
                "filter_dsl": "code ~ 'sh'"
            })),
            None,
        )
        .await
        .expect("find-one should succeed");

    assert_eq!(response["id"], 7);
    assert_eq!(response["code"], "sh010");

    let calls = transport.calls();
    let params = calls.first().expect("find params recorded");
    assert_eq!(query_value(params, "page[number]"), Some("3"));
    assert_eq!(query_value(params, "page[size]"), Some("1"));
}

#[tokio::test]
async fn entity_find_one_returns_null_when_no_match_exists() {
    let transport = FindOneTransport::new(json!({ "data": [] }));
    let app = App::new(transport);

    let response = app
        .entity_find_one(overrides(), "Shot", Some(json!({"fields": ["code"]})), None)
        .await
        .expect("empty find-one should still succeed");

    assert_eq!(response, Value::Null);
}

#[tokio::test]
async fn entity_write_commands_apply_dry_run_and_policy_guards() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    let create_dry_run = app
        .entity_create(
            overrides(),
            "Version",
            json!({"data": {"type": "Version"}}),
            true,
        )
        .await
        .expect("create dry-run succeeds");
    let update_dry_run = app
        .entity_update(
            overrides(),
            "Task",
            42,
            json!({"data": {"type": "Task", "id": 42}}),
            true,
        )
        .await
        .expect("update dry-run succeeds");
    let delete_dry_run = app
        .entity_delete(overrides(), "Playlist", 99, true, false)
        .await
        .expect("delete dry-run succeeds");
    let delete_error = app
        .entity_delete(overrides(), "Playlist", 99, false, false)
        .await
        .expect_err("delete without yes should be blocked");
    let revive_dry_run = app
        .entity_revive(overrides(), "Shot", 860, true)
        .await
        .expect("revive dry-run succeeds");
    let revive_response = app
        .entity_revive(overrides(), "Shot", 860, false)
        .await
        .expect("revive succeeds");

    assert_eq!(create_dry_run["dry_run"], true);
    assert_eq!(create_dry_run["plan"]["path"], "/api/v1.1/entity/versions");
    assert_eq!(update_dry_run["plan"]["path"], "/api/v1.1/entity/tasks/42");
    assert_eq!(
        delete_dry_run["plan"]["path"],
        "/api/v1.1/entity/playlists/99"
    );
    assert_eq!(revive_dry_run["plan"]["path"], "/api/v1.1/entity/shots/860");
    assert_eq!(revive_dry_run["plan"]["query"][0][0], "revive");
    assert_eq!(revive_dry_run["plan"]["query"][0][1], "true");
    assert_eq!(revive_dry_run["plan"]["body"], json!(null));
    assert_eq!(revive_response, json!(true));
    assert_eq!(delete_error.envelope().code, "POLICY_BLOCKED");
    assert_eq!(
        transport.snapshot().entity_revive_calls,
        vec![("Shot".to_string(), 860)]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn entity_batch_get_runs_concurrently_and_preserves_order() {
    let transport = SlowGetTransport::default();
    let app = App::new(transport.clone());

    let response = app
        .entity_batch_get(overrides(), "Shot", json!([1, 2, 3, 4]))
        .await
        .expect("batch get succeeds");

    let ids = response["results"]
        .as_array()
        .expect("results array")
        .iter()
        .map(|item| item["id"].as_u64().expect("id"))
        .collect::<Vec<_>>();

    assert_eq!(ids, vec![1, 2, 3, 4]);
    assert!(
        transport.max_in_flight() > 1,
        "batch get should execute concurrently"
    );
}

#[tokio::test]
async fn entity_batch_find_reports_item_level_errors() {
    let app = App::new(RecordingTransport::default());

    let response = app
        .entity_batch_find(
            overrides(),
            "Asset",
            json!([
                {"fields": ["code"], "query": {"page[size]": 10}},
                {"filter_dsl": 42}
            ]),
        )
        .await
        .expect("batch find returns aggregated result");

    assert_eq!(response["total"], 2);
    assert_eq!(response["success_count"], 1);
    assert_eq!(response["failure_count"], 1);
    assert_eq!(response["results"][0]["ok"], true);
    assert_eq!(response["results"][1]["ok"], false);
    assert_eq!(response["results"][1]["error"]["code"], "INVALID_INPUT");
}

#[tokio::test]
async fn entity_batch_write_commands_aggregate_partial_failures() {
    let app = App::new(RecordingTransport::default());

    let create_response = app
        .entity_batch_create(
            overrides(),
            "Version",
            json!([
                {"data": {"type": "Version"}},
                {"fail": true}
            ]),
            false,
        )
        .await
        .expect("batch create returns aggregated result");
    let update_response = app
        .entity_batch_update(
            overrides(),
            "Task",
            json!([
                {"id": 42, "body": {"data": {"id": 42}}},
                {"id": 99, "body": {"data": {"id": 99}}}
            ]),
            false,
        )
        .await
        .expect("batch update returns aggregated result");
    let delete_response = app
        .entity_batch_delete(overrides(), "Task", json!([12, 13]), false, true)
        .await
        .expect("batch delete returns aggregated result");

    assert_eq!(create_response["failure_count"], 1);
    assert_eq!(create_response["results"][1]["error"]["code"], "API_ERROR");
    assert_eq!(update_response["failure_count"], 1);
    assert_eq!(update_response["results"][1]["id"], 99);
    assert_eq!(delete_response["failure_count"], 1);
    assert_eq!(delete_response["results"][1]["id"], 13);
}

// ────────────────────────────────────────────────────────────────────
// New API coverage tests
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn upload_url_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .upload_url(
            overrides(),
            UploadUrlRequest {
                entity: "Version",
                id: 10,
                field_name: "sg_uploaded_movie",
                file_name: "clip.mp4",
                content_type: Some("video/mp4"),
                multipart_upload: false,
            },
        )
        .await
        .expect("upload url succeeds");
    assert_eq!(result["entity"], "Version");
    assert_eq!(result["id"], 10);
    assert_eq!(result["field_name"], "sg_uploaded_movie");
    assert_eq!(result["file_name"], "clip.mp4");
}

#[tokio::test]
async fn download_url_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .download_url(overrides(), "Attachment", 55, "this_file")
        .await
        .expect("download url succeeds");
    assert_eq!(result["entity"], "Attachment");
    assert_eq!(result["id"], 55);
    assert_eq!(result["download_url"], "https://example.com/file");
}

#[tokio::test]
async fn thumbnail_url_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .thumbnail_url(overrides(), "Shot", 77)
        .await
        .expect("thumbnail url succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 77);
    assert_eq!(result["thumbnail_url"], "https://example.com/thumb.jpg");
}

#[tokio::test]
async fn activity_stream_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .activity_stream(overrides(), "Shot", 1, None)
        .await
        .expect("activity stream succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 1);
    assert!(result["data"].as_array().expect("data array").is_empty());
}

#[tokio::test]
async fn event_log_entries_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .event_log_entries(overrides(), None)
        .await
        .expect("event log entries succeeds");
    assert!(result["data"].as_array().expect("data array").is_empty());
}

#[tokio::test]
async fn preferences_get_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .preferences_get(overrides())
        .await
        .expect("preferences get succeeds");
    assert!(result["preferences"].is_object());
}

#[tokio::test]
async fn entity_followers_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .entity_followers(overrides(), "Shot", 42)
        .await
        .expect("entity followers succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 42);
    assert!(
        result["followers"]
            .as_array()
            .expect("followers array")
            .is_empty()
    );
}

#[tokio::test]
async fn entity_follow_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let user = json!({"type": "HumanUser", "id": 456});
    let result = app
        .entity_follow(overrides(), "Shot", 42, user.clone())
        .await
        .expect("entity follow succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 42);
    assert_eq!(result["user"], user);
}

#[tokio::test]
async fn entity_unfollow_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let user = json!({"type": "HumanUser", "id": 456});
    let result = app
        .entity_unfollow(overrides(), "Shot", 42, user.clone())
        .await
        .expect("entity unfollow succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 42);
    assert_eq!(result["unfollowed"], true);
}

#[tokio::test]
async fn note_threads_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .note_threads(overrides(), 100, None)
        .await
        .expect("note threads succeeds");
    assert_eq!(result["note_id"], 100);
    assert!(result["data"].as_array().expect("data array").is_empty());
}

#[tokio::test]
async fn note_threads_not_found_error_mentions_top_level_note_id() {
    let app = App::new(NoteThreadsNotFoundTransport);
    let err = app
        .note_threads(overrides(), 1, None)
        .await
        .expect_err("missing note should be translated");

    assert_eq!(err.envelope().code, "API_ERROR");
    assert_eq!(err.envelope().transport.as_deref(), Some("rest"));
    assert!(
        err.envelope()
            .message
            .contains("not a top-level Note record id"),
        "unexpected message: {}",
        err.envelope().message
    );
}

#[tokio::test]
async fn schema_field_create_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({
        "data_type": "text",
        "properties": [{"property_name": "name", "value": "Custom Field"}]
    });
    let result = app
        .schema_field_create(overrides(), "Shot", body.clone())
        .await
        .expect("schema field create succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["body"], body);
}

#[tokio::test]
async fn schema_field_update_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({
        "properties": [{"property_name": "description", "value": "Updated"}]
    });
    let result = app
        .schema_field_update(overrides(), "Shot", "sg_custom", body.clone())
        .await
        .expect("schema field update succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["field_name"], "sg_custom");
    assert_eq!(result["body"], body);
}

#[tokio::test]
async fn schema_field_delete_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .schema_field_delete(overrides(), "Shot", "sg_obsolete")
        .await
        .expect("schema field delete succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["field_name"], "sg_obsolete");
    assert_eq!(result["deleted"], true);
}

#[tokio::test]
async fn hierarchy_search_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({
        "root_entity": {"type": "Project", "id": 123},
        "seed_entity_type": "Shot"
    });
    let result = app
        .hierarchy_search(overrides(), body.clone())
        .await
        .expect("hierarchy search succeeds");
    assert_eq!(result["data"], body);
}

// ────────────────────────────────────────────────────────────────────
// filter_dsl edge-case tests
// ────────────────────────────────────────────────────────────────────

#[test]
fn filter_dsl_empty_input_is_rejected() {
    let result = fpt_domain::parse_filter_dsl("");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[test]
fn filter_dsl_whitespace_only_is_rejected() {
    let result = fpt_domain::parse_filter_dsl("   \t\n  ");
    assert!(result.is_err());
}

#[test]
fn filter_dsl_trailing_content_is_rejected() {
    let result = fpt_domain::parse_filter_dsl("code == 'a' leftover");
    assert!(result.is_err());
    let msg = result.unwrap_err().envelope().message;
    assert!(
        msg.contains("trailing"),
        "error should mention trailing content: {msg}"
    );
}

#[test]
fn filter_dsl_nested_parentheses_work() {
    let result =
        fpt_domain::parse_filter_dsl("(code == 'a' and (status == 'ip' or status == 'fin'))")
            .unwrap();
    let conditions = result["conditions"].as_array().expect("root conditions");
    assert_eq!(conditions.len(), 2);
    let inner = &conditions[1];
    assert_eq!(inner["logical_operator"], "or");
}

#[test]
fn filter_dsl_not_in_operator() {
    let result = fpt_domain::parse_filter_dsl("status not in ['ip', 'fin']").unwrap();
    let cond = &result["conditions"][0];
    assert_eq!(cond[1], "not_in");
    assert_eq!(cond[2], json!(["ip", "fin"]));
}

#[test]
fn filter_dsl_null_and_boolean_values() {
    let result =
        fpt_domain::parse_filter_dsl("active == true and deleted == false and notes == null")
            .unwrap();
    let conditions = result["conditions"].as_array().expect("conditions");
    assert_eq!(conditions.len(), 3);
    assert_eq!(conditions[0][2], json!(true));
    assert_eq!(conditions[1][2], json!(false));
    assert_eq!(conditions[2][2], json!(null));
}

#[test]
fn filter_dsl_negative_number() {
    let result = fpt_domain::parse_filter_dsl("priority > -5").unwrap();
    let cond = &result["conditions"][0];
    assert_eq!(cond[0], "priority");
    assert_eq!(cond[1], "greater_than");
    assert_eq!(cond[2], json!(-5));
}

#[test]
fn filter_dsl_float_number() {
    let result = fpt_domain::parse_filter_dsl("score >= 3.141592653589793").unwrap();
    let cond = &result["conditions"][0];
    assert_eq!(cond[1], "greater_than_or_equal");
    assert_eq!(cond[2], json!(std::f64::consts::PI));
}

#[test]
fn filter_dsl_escaped_quotes_in_string() {
    let result = fpt_domain::parse_filter_dsl(r#"code == "it\'s \"fine\"""#).unwrap();
    let cond = &result["conditions"][0];
    assert_eq!(cond[2], "it's \"fine\"");
}

#[test]
fn filter_dsl_dotted_field_path() {
    let result = fpt_domain::parse_filter_dsl("project.name == 'MyProject'").unwrap();
    let cond = &result["conditions"][0];
    assert_eq!(cond[0], "project.name");
}

#[test]
fn filter_dsl_all_comparison_operators() {
    let cases = vec![
        ("x == 1", "is"),
        ("x != 1", "is_not"),
        ("x > 1", "greater_than"),
        ("x >= 1", "greater_than_or_equal"),
        ("x < 1", "less_than"),
        ("x <= 1", "less_than_or_equal"),
        ("x ~ 'a'", "contains"),
    ];
    for (input, expected_op) in cases {
        let result = fpt_domain::parse_filter_dsl(input).unwrap();
        let cond = &result["conditions"][0];
        assert_eq!(cond[1], expected_op, "failed for input: {input}");
    }
}

// ────────────────────────────────────────────────────────────────────
// config edge-case tests
// ────────────────────────────────────────────────────────────────────

#[test]
fn auth_mode_from_str_covers_all_aliases() {
    let cases = vec![
        ("script", AuthMode::Script),
        ("client_credentials", AuthMode::Script),
        ("user_password", AuthMode::UserPassword),
        ("user-password", AuthMode::UserPassword),
        ("password", AuthMode::UserPassword),
        ("user", AuthMode::UserPassword),
        ("session_token", AuthMode::SessionToken),
        ("session-token", AuthMode::SessionToken),
        ("session", AuthMode::SessionToken),
    ];
    for (input, expected) in cases {
        let parsed: AuthMode = input.parse().unwrap();
        assert_eq!(parsed, expected, "failed for input: {input}");
    }
}

#[test]
fn auth_mode_from_str_rejects_unknown() {
    let result: std::result::Result<AuthMode, _> = "magic".parse();
    assert!(result.is_err());
}

#[test]
fn auth_mode_grant_type_mapping() {
    assert_eq!(AuthMode::Script.grant_type(), "client_credentials");
    assert_eq!(AuthMode::UserPassword.grant_type(), "password");
    assert_eq!(AuthMode::SessionToken.grant_type(), "session_token");
}

#[test]
fn api_version_or_default_uses_v1_1() {
    assert_eq!(fpt_domain::api_version_or_default(None), "v1.1");
    assert_eq!(fpt_domain::api_version_or_default(Some("")), "v1.1");
    assert_eq!(fpt_domain::api_version_or_default(Some("  ")), "v1.1");
    assert_eq!(fpt_domain::api_version_or_default(Some("v1.0")), "v1.0");
}

#[test]
fn connection_settings_resolve_missing_site_reports_error() {
    let result = ConnectionSettings::resolve(ConnectionOverrides::default());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[test]
fn connection_settings_resolve_strips_trailing_slash_from_site() {
    let settings = ConnectionSettings::resolve(ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com/".to_string()),
        auth_mode: Some(AuthMode::Script),
        script_name: Some("test".to_string()),
        script_key: Some("key".to_string()),
        ..Default::default()
    })
    .expect("resolve succeeds");
    assert_eq!(settings.site, "https://example.shotgrid.autodesk.com");
}

#[test]
fn connection_settings_summary_contains_expected_fields() {
    let settings = ConnectionSettings::resolve(ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::Script),
        script_name: Some("test".to_string()),
        script_key: Some("key".to_string()),
        ..Default::default()
    })
    .expect("resolve succeeds");
    let summary = settings.summary();
    assert_eq!(summary.site, "https://example.shotgrid.autodesk.com");
    assert_eq!(summary.auth_mode, AuthMode::Script);
    assert_eq!(summary.principal, Some("test".to_string()));
    assert_eq!(summary.api_version, "v1.1");
}

// ────────────────────────────────────────────────────────────────────
// New API integration tests (schema.field-read, work-schedule.update,
// entity.text-search, note.reply-create, entity.batch.revive)
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn schema_field_read_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .schema_field_read(overrides(), "Shot", "code")
        .await
        .expect("schema field read succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["field_name"], "code");
    assert_eq!(result["data_type"], "text");
}

#[tokio::test]
async fn work_schedule_update_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .work_schedule_update(
            overrides(),
            json!({
                "date": "2026-04-01",
                "working": false
            }),
        )
        .await
        .expect("work schedule update succeeds");
    assert_eq!(result["updated"], true);
    assert_eq!(result["body"]["date"], "2026-04-01");
    assert_eq!(result["body"]["working"], false);
}

#[tokio::test]
async fn work_schedule_update_rejects_missing_date() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .work_schedule_update(overrides(), json!({"working": true}))
        .await
        .expect_err("missing date should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn work_schedule_update_rejects_missing_working() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .work_schedule_update(overrides(), json!({"date": "2026-04-01"}))
        .await
        .expect_err("missing working should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn work_schedule_update_rejects_non_bool_working() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .work_schedule_update(overrides(), json!({"date": "2026-04-01", "working": "yes"}))
        .await
        .expect_err("non-bool working should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn work_schedule_update_accepts_optional_project_and_user() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .work_schedule_update(
            overrides(),
            json!({
                "date": "2026-04-01",
                "working": false,
                "project": {"type": "Project", "id": 123},
                "user": {"type": "HumanUser", "id": 456}
            }),
        )
        .await
        .expect("work schedule update with project and user succeeds");
    assert_eq!(result["updated"], true);
}

#[tokio::test]
async fn text_search_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .text_search(
            overrides(),
            json!({
                "text": "hero shot",
                "entity_types": {"Shot": {}, "Asset": {}}
            }),
        )
        .await
        .expect("text search succeeds");
    assert_eq!(result["text"], "hero shot");
    assert!(result["data"].as_array().expect("data array").is_empty());
}

#[tokio::test]
async fn text_search_rejects_non_object_input() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .text_search(overrides(), json!("not an object"))
        .await
        .expect_err("non-object input should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn text_search_rejects_missing_text_field() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .text_search(overrides(), json!({"entity_types": {}}))
        .await
        .expect_err("missing text field should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn text_search_rejects_empty_text_field() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .text_search(overrides(), json!({"text": "  "}))
        .await
        .expect_err("empty text should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn note_reply_create_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .note_reply_create(overrides(), 456, json!({"content": "Looks great!"}))
        .await
        .expect("note reply create succeeds");
    assert_eq!(result["note_id"], 456);
    assert_eq!(result["content"], "Looks great!");
    assert_eq!(result["type"], "Reply");
}

#[tokio::test]
async fn note_reply_create_rejects_missing_content() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .note_reply_create(overrides(), 456, json!({"type": "Reply"}))
        .await
        .expect_err("missing content should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn note_reply_create_rejects_non_object_body() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .note_reply_create(overrides(), 456, json!("just a string"))
        .await
        .expect_err("non-object body should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_batch_revive_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .entity_batch_revive(overrides(), "Shot", json!({"ids": [860, 861]}), false)
        .await
        .expect("batch revive succeeds");
    assert_eq!(result["total"], 2);
    assert_eq!(result["success_count"], 2);
    assert_eq!(result["failure_count"], 0);
    let results = result["results"].as_array().expect("results array");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["ok"], true);
    assert_eq!(results[0]["id"], 860);
    assert_eq!(results[1]["ok"], true);
    assert_eq!(results[1]["id"], 861);
}

#[tokio::test]
async fn entity_batch_revive_dry_run_returns_plan() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .entity_batch_revive(overrides(), "Shot", json!({"ids": [860, 861]}), true)
        .await
        .expect("batch revive dry-run succeeds");
    assert_eq!(result["dry_run"], true);
    let plans = result["plans"].as_array().expect("plans array");
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0]["path"], "/api/v1.1/entity/shots/860");
}

#[tokio::test]
async fn entity_batch_revive_accepts_plain_ids_array() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .entity_batch_revive(overrides(), "Shot", json!([860, 861, 862]), false)
        .await
        .expect("batch revive with plain array succeeds");
    assert_eq!(result["total"], 3);
    assert_eq!(result["success_count"], 3);
}

// ────────────────────────────────────────────────────────────────────
// New endpoint integration tests: entity_relationships, user_following,
// project_update_last_accessed, schema_entity_read, schema_field_revive
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn entity_relationships_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .entity_relationships(overrides(), "Shot", 42, "assets", None)
        .await
        .expect("entity relationships succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 42);
    assert_eq!(result["related_field"], "assets");
    assert!(result["data"].as_array().expect("data array").is_empty());
}

#[tokio::test]
async fn entity_relationships_passes_query_params() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .entity_relationships(
            overrides(),
            "Task",
            7,
            "entity",
            Some(json!({"fields": "code,id", "page": {"size": 10}})),
        )
        .await
        .expect("entity relationships with params succeeds");
    assert_eq!(result["entity"], "Task");
    assert_eq!(result["related_field"], "entity");
}

#[tokio::test]
async fn user_following_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .user_following(overrides(), 456, None)
        .await
        .expect("user following succeeds");
    assert_eq!(result["user_id"], 456);
    assert!(result["data"].as_array().expect("data array").is_empty());
}

#[tokio::test]
async fn user_following_passes_query_params() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .user_following(
            overrides(),
            456,
            Some(json!({"fields": "code,id", "page": {"size": 25}})),
        )
        .await
        .expect("user following with params succeeds");
    assert_eq!(result["user_id"], 456);
}

#[tokio::test]
async fn project_update_last_accessed_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .project_update_last_accessed(overrides(), 123)
        .await
        .expect("project update last accessed succeeds");
    assert_eq!(result["project_id"], 123);
    assert_eq!(result["ok"], true);
}

#[tokio::test]
async fn schema_entity_read_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .schema_entity_read(overrides(), "Shot")
        .await
        .expect("schema entity read succeeds");
    assert_eq!(result["entity"], "Shot");
    assert!(result["data"].is_object());
}

#[tokio::test]
async fn schema_field_revive_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .schema_field_revive(overrides(), "Shot", "sg_deleted_field")
        .await
        .expect("schema field revive succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["field_name"], "sg_deleted_field");
    assert_eq!(result["revived"], true);
}

#[tokio::test]
async fn capabilities_includes_new_endpoint_specs() {
    let app = App::new(RecordingTransport::default());
    let result = app.capabilities(env!("CARGO_PKG_VERSION"));
    let commands = result["commands"].as_array().expect("commands array");
    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(
        names.contains(&"entity.relationship"),
        "should include entity.relationship"
    );
    assert!(
        names.contains(&"entity.update-last-accessed"),
        "should include entity.update-last-accessed"
    );
    assert!(
        names.contains(&"followers.following"),
        "should include followers.following"
    );
    assert!(
        names.contains(&"schema.entity-read"),
        "should include schema.entity-read"
    );
    assert!(
        names.contains(&"schema.field-revive"),
        "should include schema.field-revive"
    );
}

#[tokio::test]
async fn schema_entity_update_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"name": {"value": "Renamed Entity"}});
    let result = app
        .schema_entity_update(overrides(), "CustomEntity", body.clone())
        .await
        .expect("schema entity update succeeds");
    assert_eq!(result["entity"], "CustomEntity");
    assert_eq!(result["body"], body);
    assert_eq!(result["updated"], true);
}

#[tokio::test]
async fn schema_entity_delete_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .schema_entity_delete(overrides(), "CustomEntity")
        .await
        .expect("schema entity delete succeeds");
    assert_eq!(result["entity"], "CustomEntity");
    assert_eq!(result["deleted"], true);
}

#[tokio::test]
async fn batch_find_one_returns_first_matching_record() {
    let transport = RecordingTransport::default();
    let app = App::new(transport);
    let input = json!([
        {"fields": ["code"], "filters": [["sg_status_list", "is", "ip"]]},
        {"fields": ["code"], "filters": [["sg_status_list", "is", "fin"]]}
    ]);
    let result = app
        .entity_batch_find_one(overrides(), "Shot", input)
        .await
        .expect("batch find-one succeeds");

    assert_eq!(result["operation"], "entity.batch.find-one");
    assert_eq!(result["entity"], "Shot");
    let results = result["results"].as_array().expect("results array");
    assert_eq!(results.len(), 2);
    // Both should have been processed (ok might be false due to missing "data" in mock response)
    for item in results {
        assert!(item.get("index").is_some(), "each result has an index");
        assert!(item.get("request").is_some(), "each result has a request");
    }
}

#[tokio::test]
async fn batch_find_one_with_data_extracts_first() {
    // Use FindOneTransport which returns a response with "data" array
    let transport = FindOneTransport::new(json!({"data": [{"id": 42, "code": "hero_shot"}]}));
    let app = App::new(transport);
    let input = json!([{"fields": ["code"]}]);
    let result = app
        .entity_batch_find_one(overrides(), "Shot", input)
        .await
        .expect("batch find-one succeeds");
    let results = result["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["ok"], true);
    assert_eq!(results[0]["response"]["id"], 42);
    assert_eq!(results[0]["response"]["code"], "hero_shot");
}

#[tokio::test]
async fn batch_find_one_with_empty_data_returns_null() {
    let transport = FindOneTransport::new(json!({"data": []}));
    let app = App::new(transport);
    let input = json!([{"fields": ["code"]}]);
    let result = app
        .entity_batch_find_one(overrides(), "Shot", input)
        .await
        .expect("batch find-one succeeds");
    let results = result["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["ok"], true);
    assert!(results[0]["response"].is_null());
}

#[tokio::test]
async fn capabilities_includes_new_schema_and_batch_specs() {
    let app = App::new(RecordingTransport::default());
    let result = app.capabilities(env!("CARGO_PKG_VERSION"));
    let commands = result["commands"].as_array().expect("commands array");
    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(
        names.contains(&"schema.entity-update"),
        "should include schema.entity-update"
    );
    assert!(
        names.contains(&"schema.entity-delete"),
        "should include schema.entity-delete"
    );
    assert!(
        names.contains(&"entity.batch.find-one"),
        "should include entity.batch.find-one"
    );
}

// ────────────────────────────────────────────────────────────────────
// schema.entity-create, schema.entity-revive, entity.count,
// entity.batch.upsert capability, entity.batch.summarize
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn schema_entity_create_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"name": {"value": "CustomEntity01"}});
    let result = app
        .schema_entity_create(overrides(), body.clone())
        .await
        .expect("schema entity create succeeds");
    assert_eq!(result["body"], body);
    assert_eq!(result["created"], true);
}

#[tokio::test]
async fn schema_entity_revive_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .schema_entity_revive(overrides(), "CustomEntity01")
        .await
        .expect("schema entity revive succeeds");
    assert_eq!(result["entity"], "CustomEntity01");
    assert_eq!(result["revived"], true);
}

#[tokio::test]
async fn entity_count_without_filters_returns_record_count() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());
    let result = app
        .entity_count(overrides(), "Shot", None, None)
        .await
        .expect("entity count without filters succeeds");
    // The mock summarize returns the entity and summaries.
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["summaries"][0]["type"], "record_count");

    let snapshot = transport.snapshot();
    assert_eq!(snapshot.entity_summarize_calls.len(), 1);
    let (entity, payload) = &snapshot.entity_summarize_calls[0];
    assert_eq!(entity, "Shot");
    assert_eq!(payload["summaries"][0]["field"], "id");
    assert_eq!(payload["summaries"][0]["type"], "record_count");
    assert_eq!(payload["filters"]["filter_operator"], "all");
}

#[tokio::test]
async fn entity_count_with_filter_dsl() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());
    let result = app
        .entity_count(
            overrides(),
            "Task",
            None,
            Some("sg_status_list == 'ip'".to_string()),
        )
        .await
        .expect("entity count with filter_dsl succeeds");
    assert_eq!(result["entity"], "Task");

    let snapshot = transport.snapshot();
    let (_, payload) = &snapshot.entity_summarize_calls[0];
    assert_eq!(payload["filters"]["filter_operator"], "all");
    let filters = payload["filters"]["filters"]
        .as_array()
        .expect("filters array");
    assert_eq!(filters.len(), 1);
    assert_eq!(filters[0][0], "sg_status_list");
}

#[tokio::test]
async fn entity_count_with_json_input() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());
    let result = app
        .entity_count(
            overrides(),
            "Version",
            Some(json!({
                "filters": [["sg_status_list", "is", "ip"]],
                "filter_operator": "any"
            })),
            None,
        )
        .await
        .expect("entity count with JSON input succeeds");
    assert_eq!(result["entity"], "Version");

    let snapshot = transport.snapshot();
    let (_, payload) = &snapshot.entity_summarize_calls[0];
    assert_eq!(payload["filters"]["filter_operator"], "any");
}

#[tokio::test]
async fn entity_count_rejects_input_and_filter_dsl_together() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .entity_count(
            overrides(),
            "Shot",
            Some(json!({"filters": []})),
            Some("code == 'a'".to_string()),
        )
        .await
        .expect_err("input and filter_dsl together should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_batch_summarize_aggregates_results() {
    let app = App::new(RecordingTransport::default());
    let input = json!([
        {
            "entity": "Shot",
            "payload": {
                "filters": {"filter_operator": "all", "filters": []},
                "summaries": [{"field": "id", "type": "record_count"}]
            }
        },
        {
            "entity": "Task",
            "payload": {
                "filters": {"filter_operator": "all", "filters": []},
                "summaries": [{"field": "id", "type": "record_count"}]
            }
        }
    ]);
    let result = app
        .entity_batch_summarize(overrides(), input)
        .await
        .expect("batch summarize succeeds");
    assert_eq!(result["operation"], "entity.batch.summarize");
    assert_eq!(result["total"], 2);
    assert_eq!(result["success_count"], 2);
    assert_eq!(result["failure_count"], 0);
    let results = result["results"].as_array().expect("results array");
    assert_eq!(results[0]["ok"], true);
    assert_eq!(results[1]["ok"], true);
}

#[tokio::test]
async fn capabilities_includes_all_new_specs() {
    let app = App::new(RecordingTransport::default());
    let result = app.capabilities(env!("CARGO_PKG_VERSION"));
    let commands = result["commands"].as_array().expect("commands array");
    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(
        names.contains(&"schema.entity-create"),
        "should include schema.entity-create"
    );
    assert!(
        names.contains(&"schema.entity-revive"),
        "should include schema.entity-revive"
    );
    assert!(
        names.contains(&"entity.batch.upsert"),
        "should include entity.batch.upsert"
    );
    assert!(
        names.contains(&"entity.count"),
        "should include entity.count"
    );
    assert!(
        names.contains(&"entity.batch.summarize"),
        "should include entity.batch.summarize"
    );
}

// ────────────────────────────────────────────────────────────────────
// inspect.command tests
// ────────────────────────────────────────────────────────────────────

#[test]
fn inspect_command_returns_spec_for_known_command() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .inspect_command("entity.get")
        .expect("known command should succeed");
    assert_eq!(result["name"], "entity.get");
    assert!(result["summary"].is_string());
    assert!(result["risk"].is_string());
    assert!(result["implemented"].is_boolean());
    assert!(result["examples"].is_array());
}

#[test]
fn inspect_command_rejects_unknown_command() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .inspect_command("nonexistent.command")
        .expect_err("unknown command should fail");
    assert_eq!(err.envelope().code, "UNSUPPORTED_CAPABILITY");
    assert!(
        err.envelope().message.contains("nonexistent.command"),
        "error message should include the unknown command name"
    );
}

#[test]
fn inspect_command_normalizes_name() {
    let app = App::new(RecordingTransport::default());
    // Command names with spaces are normalized to dots; uppercase to lowercase.
    let result = app
        .inspect_command("Entity Get")
        .expect("normalized name should resolve");
    assert_eq!(result["name"], "entity.get");
}

// ────────────────────────────────────────────────────────────────────
// entity.batch.summarize error-path tests
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn entity_batch_summarize_rejects_non_array_non_object_input() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .entity_batch_summarize(overrides(), json!("not valid"))
        .await
        .expect_err("string input should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_batch_summarize_rejects_empty_array() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .entity_batch_summarize(overrides(), json!([]))
        .await
        .expect_err("empty array should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_batch_summarize_rejects_item_missing_entity() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .entity_batch_summarize(overrides(), json!([{"payload": {"filters": []}}]))
        .await
        .expect_err("missing entity should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
    assert!(
        err.envelope().message.contains("entity"),
        "error should mention missing `entity` field"
    );
}

#[tokio::test]
async fn entity_batch_summarize_rejects_item_missing_payload() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .entity_batch_summarize(overrides(), json!([{"entity": "Shot"}]))
        .await
        .expect_err("missing payload should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
    assert!(
        err.envelope().message.contains("payload"),
        "error should mention missing `payload` field"
    );
}

#[tokio::test]
async fn entity_batch_summarize_rejects_non_object_item() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .entity_batch_summarize(overrides(), json!([42]))
        .await
        .expect_err("non-object item should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_batch_summarize_object_form_rejects_missing_requests() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .entity_batch_summarize(overrides(), json!({"not_requests": []}))
        .await
        .expect_err("object without `requests` should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
    assert!(
        err.envelope().message.contains("requests"),
        "error should mention missing `requests` field"
    );
}

#[tokio::test]
async fn entity_batch_summarize_object_form_rejects_non_array_requests() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .entity_batch_summarize(overrides(), json!({"requests": "not an array"}))
        .await
        .expect_err("non-array `requests` should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

// ────────────────────────────────────────────────────────────────────
// current_user, note.reply-read, filmstrip.url
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn current_user_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .current_user(overrides(), "human", None)
        .await
        .expect("current_user succeeds");
    assert_eq!(result["user_type"], "human");
    assert_eq!(result["id"], 1);
    assert_eq!(result["name"], "Current User");
}

#[tokio::test]
async fn current_user_with_api_type() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .current_user(overrides(), "api", None)
        .await
        .expect("current_user with api type succeeds");
    assert_eq!(result["user_type"], "api");
}

#[tokio::test]
async fn current_user_rejects_invalid_user_type() {
    let app = App::new(RecordingTransport::default());
    let err = app
        .current_user(overrides(), "invalid_type", None)
        .await
        .expect_err("invalid user_type should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
    assert!(
        err.envelope().message.contains("unsupported user type"),
        "error should mention unsupported user type"
    );
}

#[tokio::test]
async fn note_reply_read_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .note_reply_read(overrides(), 456, 789, None)
        .await
        .expect("note_reply_read succeeds");
    assert_eq!(result["note_id"], 456);
    assert_eq!(result["reply_id"], 789);
    assert_eq!(result["content"], "Reply content");
    assert_eq!(result["type"], "Reply");
}

#[tokio::test]
async fn filmstrip_thumbnail_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .filmstrip_thumbnail(overrides(), "Version", 456)
        .await
        .expect("filmstrip_thumbnail succeeds");
    assert_eq!(result["entity"], "Version");
    assert_eq!(result["id"], 456);
    assert_eq!(result["filmstrip_url"], "https://example.com/filmstrip.jpg");
}

#[tokio::test]
async fn capabilities_includes_new_user_note_filmstrip_specs() {
    let app = App::new(RecordingTransport::default());
    let result = app.capabilities(env!("CARGO_PKG_VERSION"));
    let commands = result["commands"].as_array().expect("commands array");
    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(
        names.contains(&"user.current"),
        "should include user.current"
    );
    assert!(
        names.contains(&"note.reply-read"),
        "should include note.reply-read"
    );
    assert!(
        names.contains(&"filmstrip.url"),
        "should include filmstrip.url"
    );
}

// New API endpoint tests: preferences_update, note_reply_update, note_reply_delete
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn preferences_update_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"format_date_fields": "YYYY-MM-DD"});
    let result = app
        .preferences_update(overrides(), body.clone())
        .await
        .expect("preferences_update succeeds");
    assert_eq!(result["ok"], true);
    assert_eq!(result["preferences"], body);
}

#[tokio::test]
async fn note_reply_update_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"content": "Updated reply"});
    let result = app
        .note_reply_update(overrides(), 456, 789, body.clone())
        .await
        .expect("note_reply_update succeeds");
    assert_eq!(result["note_id"], 456);
    assert_eq!(result["reply_id"], 789);
    assert_eq!(result["updated"], body);
}

#[tokio::test]
async fn note_reply_update_requires_content_field() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"type": "Reply"});
    let err = app
        .note_reply_update(overrides(), 456, 789, body)
        .await
        .expect_err("missing content should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
    assert!(
        err.envelope().message.contains("content"),
        "error should mention missing content field"
    );
}

#[tokio::test]
async fn note_reply_update_requires_json_object() {
    let app = App::new(RecordingTransport::default());
    let body = json!("just a string");
    let err = app
        .note_reply_update(overrides(), 456, 789, body)
        .await
        .expect_err("non-object should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn note_reply_delete_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .note_reply_delete(overrides(), 456, 789)
        .await
        .expect("note_reply_delete succeeds");
    assert_eq!(result["ok"], true);
    assert_eq!(result["note_id"], 456);
    assert_eq!(result["reply_id"], 789);
}

#[tokio::test]
async fn capabilities_includes_new_preferences_and_note_specs() {
    let app = App::new(RecordingTransport::default());
    let result = app.capabilities(env!("CARGO_PKG_VERSION"));
    let commands = result["commands"].as_array().expect("commands array");
    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(
        names.contains(&"preferences.update"),
        "should include preferences.update"
    );
    assert!(
        names.contains(&"note.reply-update"),
        "should include note.reply-update"
    );
    assert!(
        names.contains(&"note.reply-delete"),
        "should include note.reply-delete"
    );
}

// ────────────────────────────────────────────────────────────────────
// New API endpoint tests: entity_relationship_create, entity_relationship_update, entity_share
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn entity_relationship_create_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"data": [{"type": "Asset", "id": 7}]});
    let result = app
        .entity_relationship_create(overrides(), "Shot", 42, "assets", body.clone())
        .await
        .expect("entity_relationship_create succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 42);
    assert_eq!(result["related_field"], "assets");
    assert_eq!(result["created"], body);
}

#[tokio::test]
async fn entity_relationship_create_requires_data_field() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"links": [{"type": "Asset", "id": 7}]});
    let err = app
        .entity_relationship_create(overrides(), "Shot", 42, "assets", body)
        .await
        .expect_err("missing data field should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
    assert!(
        err.envelope().message.contains("data"),
        "error should mention missing data field"
    );
}

#[tokio::test]
async fn entity_relationship_create_requires_json_object() {
    let app = App::new(RecordingTransport::default());
    let body = json!([{"type": "Asset", "id": 7}]);
    let err = app
        .entity_relationship_create(overrides(), "Shot", 42, "assets", body)
        .await
        .expect_err("non-object should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_relationship_update_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"data": [{"type": "Asset", "id": 10}]});
    let result = app
        .entity_relationship_update(overrides(), "Shot", 42, "assets", body.clone())
        .await
        .expect("entity_relationship_update succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 42);
    assert_eq!(result["related_field"], "assets");
    assert_eq!(result["updated"], body);
}

#[tokio::test]
async fn entity_relationship_update_requires_data_field() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"entities": [{"type": "Asset", "id": 10}]});
    let err = app
        .entity_relationship_update(overrides(), "Shot", 42, "assets", body)
        .await
        .expect_err("missing data field should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_share_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"entities": [{"type": "Project", "id": 85}]});
    let result = app
        .entity_share(overrides(), "Shot", 42, body.clone())
        .await
        .expect("entity_share succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 42);
    assert_eq!(result["shared"], body);
}

#[tokio::test]
async fn entity_share_requires_json_object() {
    let app = App::new(RecordingTransport::default());
    let body = json!("not an object");
    let err = app
        .entity_share(overrides(), "Shot", 42, body)
        .await
        .expect_err("non-object should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn capabilities_includes_new_relationship_and_share_specs() {
    let app = App::new(RecordingTransport::default());
    let result = app.capabilities(env!("CARGO_PKG_VERSION"));
    let commands = result["commands"].as_array().expect("commands array");
    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(
        names.contains(&"entity.relationship-create"),
        "should include entity.relationship-create"
    );
    assert!(
        names.contains(&"entity.relationship-update"),
        "should include entity.relationship-update"
    );
    assert!(
        names.contains(&"entity.relationship-delete"),
        "should include entity.relationship-delete"
    );
    assert!(
        names.contains(&"entity.share"),
        "should include entity.share"
    );
}

#[tokio::test]
async fn entity_relationship_delete_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"data": [{"type": "Asset", "id": 7}]});
    let result = app
        .entity_relationship_delete(overrides(), "Shot", 42, "assets", body.clone())
        .await
        .expect("entity_relationship_delete succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 42);
    assert_eq!(result["related_field"], "assets");
    assert_eq!(result["deleted"], body);
}

#[tokio::test]
async fn entity_relationship_delete_requires_data_field() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"links": [{"type": "Asset", "id": 7}]});
    let err = app
        .entity_relationship_delete(overrides(), "Shot", 42, "assets", body)
        .await
        .expect_err("missing data field should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_relationship_delete_requires_json_object() {
    let app = App::new(RecordingTransport::default());
    let body = json!([{"type": "Asset", "id": 7}]);
    let err = app
        .entity_relationship_delete(overrides(), "Shot", 42, "assets", body)
        .await
        .expect_err("non-object should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

// ---------------------------------------------------------------
// Hierarchy expand
// ---------------------------------------------------------------

#[tokio::test]
async fn hierarchy_expand_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"path": "/Project/65", "entity_fields": {"Shot": ["code"]}});
    let result = app
        .hierarchy_expand(overrides(), body.clone())
        .await
        .expect("hierarchy_expand succeeds");
    assert_eq!(result["data"], body);
}

// ---------------------------------------------------------------
// Schedule work day rules
// ---------------------------------------------------------------

#[tokio::test]
async fn schedule_work_day_rules_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .schedule_work_day_rules(overrides(), None)
        .await
        .expect("schedule_work_day_rules succeeds");
    assert!(result.get("data").is_some());
}

#[tokio::test]
async fn schedule_work_day_rules_read_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .schedule_work_day_rules_read(overrides(), 42)
        .await
        .expect("schedule_work_day_rules_read succeeds");
    assert_eq!(result["id"], 42);
}

#[tokio::test]
async fn schedule_work_day_rules_update_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"date": "2026-04-01", "is_working": false, "description": "Holiday"});
    let result = app
        .schedule_work_day_rules_update(overrides(), 42, body.clone())
        .await
        .expect("schedule_work_day_rules_update succeeds");
    assert_eq!(result["rule_id"], 42);
    assert_eq!(result["updated"], true);
    assert_eq!(result["body"], body);
}

#[tokio::test]
async fn schedule_work_day_rules_update_rejects_non_object() {
    let app = App::new(RecordingTransport::default());
    let body = json!("not an object");
    let err = app
        .schedule_work_day_rules_update(overrides(), 42, body)
        .await
        .expect_err("non-object should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

// ---------------------------------------------------------------
// License
// ---------------------------------------------------------------

#[tokio::test]
async fn license_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app.license(overrides()).await.expect("license succeeds");
    assert_eq!(result["data"]["current_users"], 10);
    assert_eq!(result["data"]["max_users"], 50);
}

// ---------------------------------------------------------------
// Preferences custom entity
// ---------------------------------------------------------------

#[tokio::test]
async fn preferences_custom_entity_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"entity_type": "CustomEntity01"});
    let result = app
        .preferences_custom_entity(overrides(), body.clone())
        .await
        .expect("preferences_custom_entity succeeds");
    assert_eq!(result["ok"], true);
    assert_eq!(result["enabled"], body);
}

// ---------------------------------------------------------------
// Schedule work day rules create
// ---------------------------------------------------------------

#[tokio::test]
async fn schedule_work_day_rules_create_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"date": "2026-12-25", "description": "Christmas"});
    let result = app
        .schedule_work_day_rules_create(overrides(), body.clone())
        .await
        .expect("schedule_work_day_rules_create succeeds");
    assert_eq!(result["created"], true);
    assert_eq!(result["body"], body);
}

#[tokio::test]
async fn schedule_work_day_rules_create_rejects_non_object() {
    let app = App::new(RecordingTransport::default());
    let body = json!("not an object");
    let err = app
        .schedule_work_day_rules_create(overrides(), body)
        .await
        .expect_err("non-object should be rejected");
    assert_eq!(err.envelope().code, "INVALID_INPUT");
}

// ---------------------------------------------------------------
// Schedule work day rules delete
// ---------------------------------------------------------------

#[tokio::test]
async fn schedule_work_day_rules_delete_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let result = app
        .schedule_work_day_rules_delete(overrides(), 42)
        .await
        .expect("schedule_work_day_rules_delete succeeds");
    assert_eq!(result["deleted"], true);
    assert_eq!(result["rule_id"], 42);
}

// ---------------------------------------------------------------
// Thumbnail upload
// ---------------------------------------------------------------

#[tokio::test]
async fn thumbnail_upload_delegates_to_transport() {
    let app = App::new(RecordingTransport::default());
    let body = json!({"filename": "thumb.jpg"});
    let result = app
        .thumbnail_upload(overrides(), "Shot", 42, body.clone())
        .await
        .expect("thumbnail_upload succeeds");
    assert_eq!(result["entity"], "Shot");
    assert_eq!(result["id"], 42);
    assert_eq!(result["uploaded"], true);
    assert_eq!(result["body"], body);
}

// ---------------------------------------------------------------
// Capabilities include new commands
// ---------------------------------------------------------------

#[tokio::test]
async fn capabilities_includes_new_api_specs() {
    let app = App::new(RecordingTransport::default());
    let result = app.capabilities(env!("CARGO_PKG_VERSION"));
    let commands = result["commands"].as_array().expect("commands array");
    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();

    assert!(
        names.contains(&"hierarchy.expand"),
        "should include hierarchy.expand"
    );
    assert!(
        names.contains(&"schedule.work-day-rules"),
        "should include schedule.work-day-rules"
    );
    assert!(
        names.contains(&"schedule.work-day-rules-update"),
        "should include schedule.work-day-rules-update"
    );
    assert!(
        names.contains(&"schedule.work-day-rules-create"),
        "should include schedule.work-day-rules-create"
    );
    assert!(
        names.contains(&"schedule.work-day-rules-delete"),
        "should include schedule.work-day-rules-delete"
    );
    assert!(
        names.contains(&"thumbnail.upload"),
        "should include thumbnail.upload"
    );
    assert!(names.contains(&"license.get"), "should include license.get");
    assert!(
        names.contains(&"preferences.custom-entity"),
        "should include preferences.custom-entity"
    );
}
