use fpt_domain::transport::{FindParams, RestTransport, ShotgridTransport};
use fpt_domain::{ConnectionSettings, Credentials};
use httpmock::Mock;
use httpmock::prelude::*;

use serde_json::json;

fn script_config(server: &MockServer) -> ConnectionSettings {
    ConnectionSettings {
        site: server.base_url(),
        credentials: Credentials::Script {
            script_name: "openclaw".to_string(),
            script_key: "secret-key".to_string(),
        },
        api_version: "v1.1".to_string(),
    }
}

fn mock_auth(server: &MockServer) -> Mock<'_> {
    server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/auth/access_token")
            .header("accept", "application/json")
            .body_includes("grant_type=client_credentials")
            .body_includes("client_id=openclaw")
            .body_includes("client_secret=secret-key");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "access_token": "token-123",
                "token_type": "Bearer",
                "expires_in": 3600
            }));
    })
}

#[tokio::test]
async fn auth_test_uses_script_oauth_form_payload() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let transport = RestTransport::default();

    let response = transport
        .auth_test(&script_config(&server))
        .await
        .expect("auth test succeeds");

    assert_eq!(response["ok"], true);
    assert_eq!(response["grant_type"], "client_credentials");
    assert_eq!(response["token_received"], true);
    assert_eq!(auth.calls(), 1);
}

#[tokio::test]
async fn schema_commands_reuse_cached_token_and_hit_expected_paths() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let schema_entities = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/schema")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": ["Asset", "Shot"]}));
    });
    let schema_fields = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/schema/Shot/fields")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"code": {"name": "Code"}}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    transport
        .schema_entities(&config)
        .await
        .expect("schema entities succeeds");
    transport
        .schema_fields(&config, "Shot")
        .await
        .expect("schema fields succeeds");

    assert_eq!(auth.calls(), 1, "access token should be reused in-process");
    assert_eq!(schema_entities.calls(), 1);
    assert_eq!(schema_fields.calls(), 1);
}

#[tokio::test]
async fn entity_get_and_find_use_expected_read_endpoints() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let entity_get = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/shots/42")
            .query_param("fields", "code,sg_status_list")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"id": 42}}));
    });
    let entity_find = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/assets")
            .query_param("fields", "code")
            .query_param("page[size]", "25")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"id": 7}]}));
    });
    let entity_search = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/entity/assets/_search")
            .query_param("fields", "code")
            .header("authorization", "Bearer token-123")
            .header("content-type", "application/vnd+shotgun.api3_hash+json")
            .json_body(json!({
                "filters": {
                    "logical_operator": "and",
                    "conditions": [
                        {"path": "code", "relation": "contains", "values": ["bunny"]}
                    ]
                }
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"id": 8}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    transport
        .entity_get(
            &config,
            "Shot",
            42,
            Some(vec!["code".to_string(), "sg_status_list".to_string()]),
        )
        .await
        .expect("entity get succeeds");
    transport
        .entity_find(
            &config,
            "Asset",
            FindParams {
                query: vec![
                    ("fields".to_string(), "code".to_string()),
                    ("page[size]".to_string(), "25".to_string()),
                ],
                search: None,
            },
        )
        .await
        .expect("entity find succeeds");
    transport
        .entity_find(
            &config,
            "Asset",
            FindParams {
                query: vec![("fields".to_string(), "code".to_string())],
                search: Some(json!({
                    "filters": {
                        "logical_operator": "and",
                        "conditions": [
                            {"path": "code", "relation": "contains", "values": ["bunny"]}
                        ]
                    }
                })),
            },
        )
        .await
        .expect("entity search succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(entity_get.calls(), 1);
    assert_eq!(entity_find.calls(), 1);
    assert_eq!(entity_search.calls(), 1);
}

#[tokio::test]
async fn note_threads_use_documented_note_thread_endpoint() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let note_threads = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/notes/100/thread_contents")
            .query_param("fields", "content,user")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": []}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    transport
        .note_threads(
            &config,
            100,
            &[("fields".to_string(), "content,user".to_string())],
        )
        .await
        .expect("note threads succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(note_threads.calls(), 1);
}

#[tokio::test]
async fn rpc_methods_use_expected_paths_and_payloads() {
    let server = MockServer::start();
    let revive = server.mock(|when, then| {
        when.method(POST).path("/api3/json").json_body(json!({
            "method_name": "revive",
            "params": [
                {
                    "script_name": "openclaw",
                    "script_key": "secret-key"
                },
                {
                    "type": "Shot",
                    "id": 860
                }
            ]
        }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"results": true}));
    });
    let work_schedule = server.mock(|when, then| {
        when.method(POST).path("/api3/json").json_body(json!({
            "method_name": "work_schedule_read",
            "params": [
                {
                    "script_name": "openclaw",
                    "script_key": "secret-key"
                },
                {
                    "start_date": "2026-03-16",
                    "end_date": "2026-03-20"
                }
            ]
        }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "results": {
                    "2026-03-16": {
                        "working": true,
                        "reason": "STUDIO_WORK_WEEK"
                    }
                }
            }));
    });
    let summarize = server.mock(|when, then| {
        when.method(POST).path("/api3/json").json_body(json!({
            "method_name": "summarize",
            "params": [
                {
                    "script_name": "openclaw",
                    "script_key": "secret-key"
                },
                {
                    "type": "Version",
                    "filters": {
                        "filter_operator": "all",
                        "filters": [["sg_status_list", "is", "ip"]]
                    },
                    "summaries": [
                        {
                            "field": "id",
                            "type": "record_count"
                        }
                    ]
                }
            ]
        }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "results": {
                    "summaries": {
                        "id": {
                            "record_count": 3
                        }
                    },
                    "groups": []
                }
            }));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let revive_response = transport
        .entity_revive(&config, "Shot", 860)
        .await
        .expect("entity revive succeeds");
    let work_schedule_response = transport
        .work_schedule_read(
            &config,
            &json!({
                "start_date": "2026-03-16",
                "end_date": "2026-03-20"
            }),
        )
        .await
        .expect("work schedule succeeds");
    let summarize_response = transport
        .entity_summarize(
            &config,
            "Version",
            &json!({
                "filters": {
                    "filter_operator": "all",
                    "filters": [["sg_status_list", "is", "ip"]]
                },
                "summaries": [
                    {
                        "field": "id",
                        "type": "record_count"
                    }
                ]
            }),
        )
        .await
        .expect("entity summarize succeeds");

    assert_eq!(revive.calls(), 1);
    assert_eq!(work_schedule.calls(), 1);
    assert_eq!(summarize.calls(), 1);
    assert_eq!(revive_response, json!(true));
    assert_eq!(work_schedule_response["2026-03-16"]["working"], true);
    assert_eq!(summarize_response["summaries"]["id"]["record_count"], 3);
}

#[tokio::test]
async fn entity_write_commands_use_expected_methods_and_parse_empty_delete() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let entity_create = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/entity/versions")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"data": {"type": "Version"}}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"id": 10}}));
    });
    let entity_update = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/entity/tasks/42")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"data": {"id": 42}}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"id": 42, "updated": true}}));
    });
    let entity_delete = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/v1.1/entity/playlists/99")
            .header("authorization", "Bearer token-123");
        then.status(204);
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let create_response = transport
        .entity_create(&config, "Version", &json!({"data": {"type": "Version"}}))
        .await
        .expect("entity create succeeds");
    let update_response = transport
        .entity_update(&config, "Task", 42, &json!({"data": {"id": 42}}))
        .await
        .expect("entity update succeeds");
    let delete_response = transport
        .entity_delete(&config, "Playlist", 99)
        .await
        .expect("entity delete succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(entity_create.calls(), 1);
    assert_eq!(entity_update.calls(), 1);
    assert_eq!(entity_delete.calls(), 1);
    assert_eq!(create_response["data"]["id"], 10);
    assert_eq!(update_response["data"]["updated"], true);
    assert_eq!(delete_response["ok"], true);
    assert_eq!(delete_response["status"], 204);
}

// --- Followers endpoints ---

#[tokio::test]
async fn entity_followers_uses_expected_get_endpoint() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let followers = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/shots/50/followers")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"type": "HumanUser", "id": 10}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .entity_followers(&config, "Shot", 50)
        .await
        .expect("entity followers succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(followers.calls(), 1);
    assert_eq!(response["data"][0]["id"], 10);
}

#[tokio::test]
async fn entity_follow_posts_user_to_followers_endpoint() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let follow = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/entity/tasks/33/followers")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"type": "HumanUser", "id": 7}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"type": "HumanUser", "id": 7}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .entity_follow(&config, "Task", 33, &json!({"type": "HumanUser", "id": 7}))
        .await
        .expect("entity follow succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(follow.calls(), 1);
    assert_eq!(response["data"]["id"], 7);
}

#[tokio::test]
async fn entity_unfollow_deletes_user_from_followers_endpoint() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let unfollow = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/v1.1/entity/shots/50/followers/7")
            .header("authorization", "Bearer token-123");
        then.status(204);
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .entity_unfollow(&config, "Shot", 50, &json!({"type": "HumanUser", "id": 7}))
        .await
        .expect("entity unfollow succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(unfollow.calls(), 1);
    assert_eq!(response["ok"], true);
    assert_eq!(response["status"], 204);
}

// --- Upload / Download / Thumbnail URL endpoints ---

#[tokio::test]
async fn upload_url_uses_expected_get_endpoint_with_query_params() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let upload = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/versions/10/sg_uploaded_movie/_upload")
            .query_param("filename", "clip.mov")
            .query_param("multipart_upload", "false")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"links": {"upload": "https://s3.example.com/upload"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .upload_url(
            &config,
            fpt_domain::transport::UploadUrlRequest {
                entity: "Version",
                id: 10,
                field_name: "sg_uploaded_movie",
                file_name: "clip.mov",
                content_type: None,
                multipart_upload: false,
            },
        )
        .await
        .expect("upload url succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(upload.calls(), 1);
    assert_eq!(response["links"]["upload"], "https://s3.example.com/upload");
}

#[tokio::test]
async fn download_url_uses_expected_get_endpoint() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let download = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/versions/10/sg_uploaded_movie/_download")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"links": {"download": "https://s3.example.com/download"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .download_url(&config, "Version", 10, "sg_uploaded_movie")
        .await
        .expect("download url succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(download.calls(), 1);
    assert_eq!(
        response["links"]["download"],
        "https://s3.example.com/download"
    );
}

#[tokio::test]
async fn thumbnail_url_uses_expected_get_endpoint() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let thumbnail = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/assets/5/image")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"links": {"thumb": "https://s3.example.com/thumb.jpg"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .thumbnail_url(&config, "Asset", 5)
        .await
        .expect("thumbnail url succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(thumbnail.calls(), 1);
    assert_eq!(
        response["links"]["thumb"],
        "https://s3.example.com/thumb.jpg"
    );
}

// --- Schema field CRUD endpoints ---

#[tokio::test]
async fn schema_field_create_posts_to_expected_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let field_create = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/schema/Shot/fields")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"data_type": "text", "properties": [{"property_name": "name", "value": "Custom Field"}]}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"name": "sg_custom_field"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_field_create(
            &config,
            "Shot",
            &json!({"data_type": "text", "properties": [{"property_name": "name", "value": "Custom Field"}]}),
        )
        .await
        .expect("schema field create succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(field_create.calls(), 1);
    assert_eq!(response["data"]["name"], "sg_custom_field");
}

#[tokio::test]
async fn schema_field_update_puts_to_expected_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let field_update = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/schema/Shot/fields/sg_custom_field")
            .header("authorization", "Bearer token-123")
            .json_body(
                json!({"properties": [{"property_name": "name", "value": "Renamed Field"}]}),
            );
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"name": "Renamed Field"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_field_update(
            &config,
            "Shot",
            "sg_custom_field",
            &json!({"properties": [{"property_name": "name", "value": "Renamed Field"}]}),
        )
        .await
        .expect("schema field update succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(field_update.calls(), 1);
    assert_eq!(response["data"]["name"], "Renamed Field");
}

#[tokio::test]
async fn schema_field_delete_uses_delete_method() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let field_delete = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/v1.1/schema/Shot/fields/sg_custom_field")
            .header("authorization", "Bearer token-123");
        then.status(204);
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_field_delete(&config, "Shot", "sg_custom_field")
        .await
        .expect("schema field delete succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(field_delete.calls(), 1);
    assert_eq!(response["ok"], true);
    assert_eq!(response["status"], 204);
}

#[tokio::test]
async fn schema_field_read_uses_get_method() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let field_read = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/schema/Shot/fields/code")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"data_type": "text", "name": "code"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_field_read(&config, "Shot", "code")
        .await
        .expect("schema field read succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(field_read.calls(), 1);
    assert_eq!(response["data"]["data_type"], "text");
}

#[tokio::test]
async fn schema_field_revive_posts_with_revive_query_param() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let field_revive = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/schema/Shot/fields/sg_custom_field")
            .query_param("revive", "true")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"name": "sg_custom_field", "revived": true}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_field_revive(&config, "Shot", "sg_custom_field")
        .await
        .expect("schema field revive succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(field_revive.calls(), 1);
    assert_eq!(response["data"]["revived"], true);
}

// --- Schema entity CRUD endpoints ---

#[tokio::test]
async fn schema_entity_read_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let entity_read = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/schema/CustomEntity01")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"name": "CustomEntity01"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_entity_read(&config, "CustomEntity01")
        .await
        .expect("schema entity read succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(entity_read.calls(), 1);
    assert_eq!(response["data"]["name"], "CustomEntity01");
}

#[tokio::test]
async fn schema_entity_update_uses_put_method() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let entity_update = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/schema/CustomEntity01")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"name": "Renamed Entity"}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"name": "Renamed Entity"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_entity_update(
            &config,
            "CustomEntity01",
            &json!({"name": "Renamed Entity"}),
        )
        .await
        .expect("schema entity update succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(entity_update.calls(), 1);
    assert_eq!(response["data"]["name"], "Renamed Entity");
}

#[tokio::test]
async fn schema_entity_delete_uses_delete_method() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let entity_delete = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/v1.1/schema/CustomEntity01")
            .header("authorization", "Bearer token-123");
        then.status(204);
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_entity_delete(&config, "CustomEntity01")
        .await
        .expect("schema entity delete succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(entity_delete.calls(), 1);
    assert_eq!(response["ok"], true);
    assert_eq!(response["status"], 204);
}

#[tokio::test]
async fn schema_entity_create_posts_to_schema_root() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let entity_create = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/schema")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"name": "CustomEntity02", "schema_field_type": "entity"}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"name": "CustomEntity02"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_entity_create(
            &config,
            &json!({"name": "CustomEntity02", "schema_field_type": "entity"}),
        )
        .await
        .expect("schema entity create succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(entity_create.calls(), 1);
    assert_eq!(response["data"]["name"], "CustomEntity02");
}

#[tokio::test]
async fn schema_entity_revive_posts_with_revive_query_param() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let entity_revive = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/schema/CustomEntity01")
            .query_param("revive", "true")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"name": "CustomEntity01", "revived": true}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schema_entity_revive(&config, "CustomEntity01")
        .await
        .expect("schema entity revive succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(entity_revive.calls(), 1);
    assert_eq!(response["data"]["revived"], true);
}

// --- Hierarchy, text search, activity stream, event log, preferences ---

#[tokio::test]
async fn hierarchy_search_posts_to_expected_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let hierarchy = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/hierarchy/_search")
            .header("authorization", "Bearer token-123")
            .json_body(
                json!({"root": {"type": "Project", "id": 100}, "seed_entity_field": "entity"}),
            );
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"name": "root", "children": []}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .hierarchy(
            &config,
            &json!({"root": {"type": "Project", "id": 100}, "seed_entity_field": "entity"}),
        )
        .await
        .expect("hierarchy search succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(hierarchy.calls(), 1);
    assert_eq!(response["data"][0]["name"], "root");
}

#[tokio::test]
async fn text_search_posts_to_expected_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let text_search = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/entity/_text_search")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"text": "bunny", "entity_types": {"Shot": []}}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"type": "Shot", "id": 1}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .text_search(
            &config,
            &json!({"text": "bunny", "entity_types": {"Shot": []}}),
        )
        .await
        .expect("text search succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(text_search.calls(), 1);
    assert_eq!(response["data"][0]["type"], "Shot");
}

#[tokio::test]
async fn activity_stream_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let activity = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/shots/42/activity_stream")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": []}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .activity_stream(&config, "Shot", 42, &[])
        .await
        .expect("activity stream succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(activity.calls(), 1);
    assert_eq!(response["data"], json!([]));
}

#[tokio::test]
async fn event_log_entries_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let event_log = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/event_log_entries")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": []}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .event_log_entries(&config, &[])
        .await
        .expect("event log entries succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(event_log.calls(), 1);
    assert_eq!(response["data"], json!([]));
}

#[tokio::test]
async fn preferences_get_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let prefs = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/preferences")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"format_currency_field": "USD"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .preferences_get(&config)
        .await
        .expect("preferences get succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(prefs.calls(), 1);
    assert_eq!(response["data"]["format_currency_field"], "USD");
}

// --- Entity relationships, user following, project update last accessed ---

#[tokio::test]
async fn entity_relationships_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let relationships = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/shots/42/relationships/assets")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"type": "Asset", "id": 1}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .entity_relationships(&config, "Shot", 42, "assets", &[])
        .await
        .expect("entity relationships succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(relationships.calls(), 1);
    assert_eq!(response["data"][0]["type"], "Asset");
}

#[tokio::test]
async fn user_following_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let following = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/human_users/7/following")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": []}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .user_following(&config, 7, &[])
        .await
        .expect("user following succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(following.calls(), 1);
    assert_eq!(response["data"], json!([]));
}

#[tokio::test]
async fn project_update_last_accessed_uses_expected_put_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let update_accessed = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/entity/projects/100/_update_last_accessed")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"ok": true}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .project_update_last_accessed(&config, 100)
        .await
        .expect("project update last accessed succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(update_accessed.calls(), 1);
    assert_eq!(response["data"]["ok"], true);
}

// --- Note reply create ---

#[tokio::test]
async fn note_reply_create_posts_to_expected_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let reply = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/entity/notes/100/thread_contents")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"content": "Great work!"}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"type": "Reply", "id": 200}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .note_reply_create(&config, 100, &json!({"content": "Great work!"}))
        .await
        .expect("note reply create succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(reply.calls(), 1);
    assert_eq!(response["data"]["type"], "Reply");
}

// --- Work schedule update (RPC) ---

#[tokio::test]
async fn work_schedule_update_uses_rpc_method() {
    let server = MockServer::start();
    let ws_update = server.mock(|when, then| {
        when.method(POST).path("/api3/json").json_body(json!({
            "method_name": "work_schedule_update",
            "params": [
                {
                    "script_name": "openclaw",
                    "script_key": "secret-key"
                },
                {
                    "date": "2026-03-16",
                    "working": false,
                    "reason": "Studio Holiday"
                }
            ]
        }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"results": {"ok": true}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .work_schedule_update(
            &config,
            &json!({"date": "2026-03-16", "working": false, "reason": "Studio Holiday"}),
        )
        .await
        .expect("work schedule update succeeds");

    assert_eq!(ws_update.calls(), 1);
    assert_eq!(response["ok"], true);
}

// --- Server info (RPC) ---

#[tokio::test]
async fn server_info_uses_rpc_info_method() {
    let server = MockServer::start();
    let info = server.mock(|when, then| {
        when.method(POST).path("/api3/json").json_body(json!({
            "method_name": "info",
            "params": []
        }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"results": {"version": [8, 40, 0, 0]}}));
    });
    let transport = RestTransport::default();

    let response = transport
        .server_info(&server.base_url())
        .await
        .expect("server info succeeds");

    assert_eq!(info.calls(), 1);
    assert_eq!(response["version"], json!([8, 40, 0, 0]));
}

// --- Error mapping ---

#[tokio::test]
async fn rest_errors_map_auth_and_api_failures() {
    let auth_server = MockServer::start();
    let auth_failure = auth_server.mock(|when, then| {
        when.method(POST).path("/api/v1.1/auth/access_token");
        then.status(400)
            .header("content-type", "application/json")
            .json_body(json!({"errors": ["Can't authenticate script 'openclaw'"]}));
    });
    let auth_transport = RestTransport::default();
    let auth_error = auth_transport
        .auth_test(&script_config(&auth_server))
        .await
        .expect_err("auth failure should be surfaced");

    let api_server = MockServer::start();
    let api_auth = mock_auth(&api_server);
    let api_failure = api_server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/shots/404")
            .header("authorization", "Bearer token-123");
        then.status(404)
            .header("content-type", "application/json")
            .json_body(json!({"errors": ["record not found"]}));
    });
    let api_transport = RestTransport::default();
    let api_error = api_transport
        .entity_get(&script_config(&api_server), "Shot", 404, None)
        .await
        .expect_err("api failure should be surfaced");

    assert_eq!(auth_failure.calls(), 1);
    assert_eq!(auth_error.envelope().code, "AUTH_FAILED");
    assert_eq!(auth_error.envelope().transport.as_deref(), Some("rest"));
    assert_eq!(api_auth.calls(), 1);
    assert_eq!(api_failure.calls(), 1);
    assert_eq!(api_error.envelope().code, "API_ERROR");
    assert_eq!(api_error.envelope().transport.as_deref(), Some("rest"));
}

// --- Upload URL edge cases ---

#[tokio::test]
async fn upload_url_with_content_type_and_multipart_upload() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let upload = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/versions/10/sg_uploaded_movie/_upload")
            .query_param("filename", "clip.mov")
            .query_param("multipart_upload", "true")
            .query_param("content_type", "video/quicktime")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "links": {
                    "upload": "https://s3.example.com/multipart-upload",
                    "complete_upload": "https://s3.example.com/complete"
                }
            }));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .upload_url(
            &config,
            fpt_domain::transport::UploadUrlRequest {
                entity: "Version",
                id: 10,
                field_name: "sg_uploaded_movie",
                file_name: "clip.mov",
                content_type: Some("video/quicktime"),
                multipart_upload: true,
            },
        )
        .await
        .expect("upload url with content_type and multipart succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(upload.calls(), 1);
    assert_eq!(
        response["links"]["upload"],
        "https://s3.example.com/multipart-upload"
    );
    assert_eq!(
        response["links"]["complete_upload"],
        "https://s3.example.com/complete"
    );
}

// --- Entity unfollow error case ---

#[tokio::test]
async fn entity_unfollow_rejects_user_without_id() {
    let server = MockServer::start();
    let _auth = mock_auth(&server);
    let transport = RestTransport::default();
    let config = script_config(&server);

    let error = transport
        .entity_unfollow(&config, "Shot", 50, &json!({"type": "HumanUser"}))
        .await
        .expect_err("unfollow without user id should fail");

    assert_eq!(error.envelope().code, "INVALID_INPUT");
}

// --- Activity stream with query parameters ---

#[tokio::test]
async fn activity_stream_passes_query_parameters() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let activity = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/shots/42/activity_stream")
            .query_param("entity_fields", "Shot.code,Shot.sg_status_list")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"id": 1, "type": "activity"}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .activity_stream(
            &config,
            "Shot",
            42,
            &[(
                "entity_fields".to_string(),
                "Shot.code,Shot.sg_status_list".to_string(),
            )],
        )
        .await
        .expect("activity stream with params succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(activity.calls(), 1);
    assert_eq!(response["data"][0]["type"], "activity");
}

// --- Event log with query parameters ---

#[tokio::test]
async fn event_log_entries_passes_query_parameters() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let event_log = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/event_log_entries")
            .query_param("fields", "event_type,created_at")
            .query_param("sort", "-created_at")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"id": 100, "event_type": "Shotgun_Shot_Change"}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .event_log_entries(
            &config,
            &[
                ("fields".to_string(), "event_type,created_at".to_string()),
                ("sort".to_string(), "-created_at".to_string()),
            ],
        )
        .await
        .expect("event log entries with params succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(event_log.calls(), 1);
    assert_eq!(response["data"][0]["event_type"], "Shotgun_Shot_Change");
}

// --- User following with query parameters ---

#[tokio::test]
async fn user_following_passes_query_parameters() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let following = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/human_users/7/following")
            .query_param("fields", "code,type")
            .query_param("page[size]", "10")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"type": "Shot", "id": 1, "code": "shot_010"}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .user_following(
            &config,
            7,
            &[
                ("fields".to_string(), "code,type".to_string()),
                ("page[size]".to_string(), "10".to_string()),
            ],
        )
        .await
        .expect("user following with params succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(following.calls(), 1);
    assert_eq!(response["data"][0]["code"], "shot_010");
}

// --- Entity relationships with query parameters ---

#[tokio::test]
async fn entity_relationships_passes_query_parameters() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let relationships = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/shots/42/relationships/assets")
            .query_param("fields", "code,sg_status_list")
            .query_param("page[size]", "50")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"type": "Asset", "id": 5, "code": "hero_prop"}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .entity_relationships(
            &config,
            "Shot",
            42,
            "assets",
            &[
                ("fields".to_string(), "code,sg_status_list".to_string()),
                ("page[size]".to_string(), "50".to_string()),
            ],
        )
        .await
        .expect("entity relationships with params succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(relationships.calls(), 1);
    assert_eq!(response["data"][0]["code"], "hero_prop");
}

// --- Note threads with query parameters ---

#[tokio::test]
async fn note_threads_passes_query_parameters() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let note_threads = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/notes/100/thread_contents")
            .query_param("fields", "content,user,created_at")
            .query_param("page[size]", "5")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"type": "Reply", "id": 200, "content": "Looks good"}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .note_threads(
            &config,
            100,
            &[
                ("fields".to_string(), "content,user,created_at".to_string()),
                ("page[size]".to_string(), "5".to_string()),
            ],
        )
        .await
        .expect("note threads with params succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(note_threads.calls(), 1);
    assert_eq!(response["data"][0]["content"], "Looks good");
}

// --- Current user endpoint ---

#[tokio::test]
async fn current_user_uses_expected_get_path_for_human() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let current_user = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/human_users/current")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"type": "HumanUser", "id": 1, "name": "Alice"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .current_user(&config, "human", &[])
        .await
        .expect("current_user human succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(current_user.calls(), 1);
    assert_eq!(response["data"]["name"], "Alice");
}

#[tokio::test]
async fn current_user_uses_expected_get_path_for_api() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let current_user = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/api_users/current")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"type": "ApiUser", "id": 10, "name": "Bot"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .current_user(&config, "api", &[])
        .await
        .expect("current_user api succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(current_user.calls(), 1);
    assert_eq!(response["data"]["type"], "ApiUser");
}

#[tokio::test]
async fn current_user_passes_query_parameters() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let current_user = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/human_users/current")
            .query_param("fields", "login,name,email")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"type": "HumanUser", "id": 1}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .current_user(
            &config,
            "human",
            &[("fields".to_string(), "login,name,email".to_string())],
        )
        .await
        .expect("current_user with params succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(current_user.calls(), 1);
    assert_eq!(response["data"]["type"], "HumanUser");
}

// --- Note reply read endpoint ---

#[tokio::test]
async fn note_reply_read_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let reply_read = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/notes/456/thread_contents/789")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"type": "Reply", "id": 789, "content": "Great work!"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .note_reply_read(&config, 456, 789, &[])
        .await
        .expect("note_reply_read succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(reply_read.calls(), 1);
    assert_eq!(response["data"]["id"], 789);
    assert_eq!(response["data"]["content"], "Great work!");
}

#[tokio::test]
async fn note_reply_read_passes_query_parameters() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let reply_read = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/notes/100/thread_contents/200")
            .query_param("fields", "content,user")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"type": "Reply", "id": 200}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .note_reply_read(
            &config,
            100,
            200,
            &[("fields".to_string(), "content,user".to_string())],
        )
        .await
        .expect("note_reply_read with params succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(reply_read.calls(), 1);
    assert_eq!(response["data"]["id"], 200);
}

// --- Filmstrip thumbnail endpoint ---

#[tokio::test]
async fn filmstrip_thumbnail_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let filmstrip = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/versions/456/filmstrip_image")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"image": "https://sg-media.com/filmstrip/456.jpg"}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .filmstrip_thumbnail(&config, "Version", 456)
        .await
        .expect("filmstrip_thumbnail succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(filmstrip.calls(), 1);
    assert_eq!(response["image"], "https://sg-media.com/filmstrip/456.jpg");
}

// --- Preferences update endpoint ---

#[tokio::test]
async fn preferences_update_uses_expected_put_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let prefs = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/preferences")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"ok": true}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"format_date_fields": "YYYY-MM-DD"});

    let response = transport
        .preferences_update(&config, &body)
        .await
        .expect("preferences_update succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(prefs.calls(), 1);
    assert_eq!(response["ok"], true);
}

// --- Note reply update endpoint ---

#[tokio::test]
async fn note_reply_update_uses_expected_put_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let reply_update = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/entity/notes/100/thread_contents/200")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(
                json!({"data": {"type": "Reply", "id": 200, "attributes": {"content": "Updated"}}}),
            );
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"content": "Updated"});

    let response = transport
        .note_reply_update(&config, 100, 200, &body)
        .await
        .expect("note_reply_update succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(reply_update.calls(), 1);
    assert_eq!(response["data"]["id"], 200);
}

// --- Note reply delete endpoint ---

#[tokio::test]
async fn note_reply_delete_uses_expected_delete_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let reply_delete = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/v1.1/entity/notes/100/thread_contents/200")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"ok": true}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .note_reply_delete(&config, 100, 200)
        .await
        .expect("note_reply_delete succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(reply_delete.calls(), 1);
    assert_eq!(response["ok"], true);
}

// --- Entity relationship create endpoint ---

#[tokio::test]
async fn entity_relationship_create_uses_expected_post_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let rel_create = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/entity/shots/42/relationships/assets")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"type": "Asset", "id": 7}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"data": [{"type": "Asset", "id": 7}]});

    let response = transport
        .entity_relationship_create(&config, "Shot", 42, "assets", &body)
        .await
        .expect("entity_relationship_create succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(rel_create.calls(), 1);
    assert_eq!(response["data"][0]["type"], "Asset");
    assert_eq!(response["data"][0]["id"], 7);
}

// --- Entity relationship update endpoint ---

#[tokio::test]
async fn entity_relationship_update_uses_expected_put_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let rel_update = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/entity/shots/42/relationships/assets")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"type": "Asset", "id": 10}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"data": [{"type": "Asset", "id": 10}]});

    let response = transport
        .entity_relationship_update(&config, "Shot", 42, "assets", &body)
        .await
        .expect("entity_relationship_update succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(rel_update.calls(), 1);
    assert_eq!(response["data"][0]["type"], "Asset");
    assert_eq!(response["data"][0]["id"], 10);
}

// --- Entity share endpoint ---

#[tokio::test]
async fn entity_share_uses_expected_post_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let share = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/entity/shots/42/_share")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"ok": true, "shared_to": [{"type": "Project", "id": 85}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"entities": [{"type": "Project", "id": 85}]});

    let response = transport
        .entity_share(&config, "Shot", 42, &body)
        .await
        .expect("entity_share succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(share.calls(), 1);
    assert_eq!(response["ok"], true);
    assert_eq!(response["shared_to"][0]["type"], "Project");
}

// --- Entity relationship delete endpoint ---

#[tokio::test]
async fn entity_relationship_delete_uses_expected_delete_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let rel_delete = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/v1.1/entity/shots/42/relationships/assets")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"data": [{"type": "Asset", "id": 7}]}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": []}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"data": [{"type": "Asset", "id": 7}]});

    let response = transport
        .entity_relationship_delete(&config, "Shot", 42, "assets", &body)
        .await
        .expect("entity_relationship_delete succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(rel_delete.calls(), 1);
    assert_eq!(response["data"], json!([]));
}

// ---------------------------------------------------------------
// Hierarchy expand
// ---------------------------------------------------------------

#[tokio::test]
async fn hierarchy_expand_uses_expected_post_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let hierarchy_expand = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/hierarchy/expand")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"children": []}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"path": "/Project/65"});

    let response = transport
        .hierarchy_expand(&config, &body)
        .await
        .expect("hierarchy_expand succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(hierarchy_expand.calls(), 1);
    assert!(response.get("data").is_some());
}

// ---------------------------------------------------------------
// Schedule work day rules
// ---------------------------------------------------------------

#[tokio::test]
async fn schedule_work_day_rules_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let rules = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/schedule/work_day_rules")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": []}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schedule_work_day_rules(&config, &[])
        .await
        .expect("schedule_work_day_rules succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(rules.calls(), 1);
    assert!(response.get("data").is_some());
}

#[tokio::test]
async fn schedule_work_day_rules_update_uses_expected_put_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let rule_update = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/schedule/work_day_rules/42")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"ok": true}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"date": "2026-04-01", "is_working": false});

    let response = transport
        .schedule_work_day_rules_update(&config, 42, &body)
        .await
        .expect("schedule_work_day_rules_update succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(rule_update.calls(), 1);
    assert_eq!(response["ok"], true);
}

// ---------------------------------------------------------------
// License
// ---------------------------------------------------------------

#[tokio::test]
async fn license_uses_expected_get_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let license = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/license")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"current_users": 10, "max_users": 50}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport.license(&config).await.expect("license succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(license.calls(), 1);
    assert_eq!(response["data"]["current_users"], 10);
}

// ---------------------------------------------------------------
// Preferences custom entity
// ---------------------------------------------------------------

#[tokio::test]
async fn preferences_custom_entity_uses_expected_post_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let custom_entity = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/preferences/custom_entity")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"ok": true}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"entity_type": "CustomEntity01"});

    let response = transport
        .preferences_custom_entity(&config, &body)
        .await
        .expect("preferences_custom_entity succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(custom_entity.calls(), 1);
    assert_eq!(response["ok"], true);
}

// ---------------------------------------------------------------
// Schedule work day rules create
// ---------------------------------------------------------------

#[tokio::test]
async fn schedule_work_day_rules_create_uses_expected_post_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let rule_create = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/schedule/work_day_rules")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"id": 99, "date": "2026-12-25"}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"date": "2026-12-25", "description": "Christmas", "project": {"type": "Project", "id": 65}});

    let response = transport
        .schedule_work_day_rules_create(&config, &body)
        .await
        .expect("schedule_work_day_rules_create succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(rule_create.calls(), 1);
    assert_eq!(response["data"]["id"], 99);
    assert_eq!(response["data"]["date"], "2026-12-25");
}

// ---------------------------------------------------------------
// Schedule work day rules delete
// ---------------------------------------------------------------

#[tokio::test]
async fn schedule_work_day_rules_delete_uses_expected_delete_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let rule_delete = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/v1.1/schedule/work_day_rules/42")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"ok": true, "status": 200}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let response = transport
        .schedule_work_day_rules_delete(&config, 42)
        .await
        .expect("schedule_work_day_rules_delete succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(rule_delete.calls(), 1);
    assert_eq!(response["ok"], true);
}

// ---------------------------------------------------------------
// Thumbnail upload
// ---------------------------------------------------------------

#[tokio::test]
async fn thumbnail_upload_uses_expected_put_path() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let thumb_upload = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/entity/shots/42/image")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"upload_url": "https://sg-media.com/upload/thumb"}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);
    let body = json!({"filename": "thumb.jpg"});

    let response = transport
        .thumbnail_upload(&config, "Shot", 42, &body)
        .await
        .expect("thumbnail_upload succeeds");

    assert_eq!(auth.calls(), 1);
    assert_eq!(thumb_upload.calls(), 1);
    assert_eq!(response["upload_url"], "https://sg-media.com/upload/thumb");
}

// ---------------------------------------------------------------
// REST API version endpoint (unauthenticated)
// ---------------------------------------------------------------

#[tokio::test]
async fn rest_api_version_uses_expected_get_path_no_auth() {
    let server = MockServer::start();
    let version_endpoint = server.mock(|when, then| {
        when.method(GET).path("/api/v1.1/").header("accept", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "data": {
                    "shotgun_version": "v8.40.0.0",
                    "api_version": "v1.1"
                }
            }));
    });
    let transport = RestTransport::default();

    let response = transport
        .rest_api_version(&server.base_url(), "v1.1")
        .await
        .expect("rest_api_version succeeds");

    assert_eq!(version_endpoint.calls(), 1);
    assert_eq!(response["data"]["api_version"], "v1.1");
    assert_eq!(response["data"]["shotgun_version"], "v8.40.0.0");
}

// ---------------------------------------------------------------
// OpenAPI spec endpoint (unauthenticated) — JSON format
// ---------------------------------------------------------------

#[tokio::test]
async fn openapi_spec_json_uses_expected_get_path_no_auth() {
    let server = MockServer::start();
    let spec_endpoint = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/spec.json")
            .header("accept", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "openapi": "3.0.0",
                "info": {"title": "ShotGrid REST API", "version": "1.1"}
            }));
    });
    let transport = RestTransport::default();

    let response = transport
        .openapi_spec(&server.base_url(), "v1.1", "json")
        .await
        .expect("openapi_spec json succeeds");

    assert_eq!(spec_endpoint.calls(), 1);
    assert_eq!(response["openapi"], "3.0.0");
    assert_eq!(response["info"]["title"], "ShotGrid REST API");
}

// ---------------------------------------------------------------
// OpenAPI spec endpoint (unauthenticated) — YAML format
// ---------------------------------------------------------------

#[tokio::test]
async fn openapi_spec_yaml_wraps_response_in_json_envelope() {
    let server = MockServer::start();
    let spec_endpoint = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/spec.yaml")
            .header("accept", "application/x-yaml");
        then.status(200)
            .header("content-type", "application/x-yaml")
            .body("openapi: '3.0.0'\ninfo:\n  title: ShotGrid REST API\n  version: '1.1'\n");
    });
    let transport = RestTransport::default();

    let response = transport
        .openapi_spec(&server.base_url(), "v1.1", "yaml")
        .await
        .expect("openapi_spec yaml succeeds");

    assert_eq!(spec_endpoint.calls(), 1);
    assert_eq!(response["format"], "yaml");
    assert!(response["content"].as_str().unwrap().contains("openapi: '3.0.0'"));
}
