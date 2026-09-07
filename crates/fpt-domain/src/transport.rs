use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use fpt_core::{AppError, Result, RiskLevel};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{Client, Method, Response, StatusCode};
use serde::Serialize;
use serde_json::{Value, json};
use url::Url;

use crate::config::{ConnectionSettings, Credentials};

/// Maximum number of retry attempts for rate-limited or transient failures.
const MAX_RETRY_ATTEMPTS: u32 = 5;
/// Base delay for exponential backoff (milliseconds).
const RETRY_BASE_DELAY_MS: u64 = 500;
/// Maximum backoff delay cap (milliseconds).
const RETRY_MAX_DELAY_MS: u64 = 30_000;
/// Shared dry-run note used by all request plan builders.
const DRY_RUN_NOTE: &str = "dry-run: shows the planned request without making a network call";

/// Transport label for the REST API surface.
pub(crate) const TRANSPORT_REST: &str = "rest";

/// Transport label for the legacy JSON-RPC API surface.
const TRANSPORT_RPC: &str = "rpc";

/// Custom `Content-Type` header required by the `ShotGrid` REST `_search`
/// endpoint.  Using a named constant avoids scattering the vendor media-type
/// across multiple call sites.
const SHOTGRID_SEARCH_CONTENT_TYPE: &str = "application/vnd+shotgun.api3_hash+json";

/// Environment variable that, when set, enables verbose debug output for
/// transport-level operations (e.g. retry logging).
const ENV_FPT_DEBUG: &str = "FPT_DEBUG";

/// Environment variable for overriding the maximum number of retry attempts.
const ENV_FPT_MAX_RETRIES: &str = "FPT_MAX_RETRIES";

/// Error message used when the in-memory token cache `Mutex` is poisoned.
const TOKEN_CACHE_POISONED: &str = "token cache is poisoned";

/// Safety margin (in seconds) subtracted from the access-token TTL so that
/// the token is refreshed before it actually expires on the server.
const TOKEN_EXPIRY_MARGIN_SECS: u64 = 30;

/// Query-string and search-body parameters for entity find operations.
///
/// When `search` is `Some`, the find request uses the `ShotGrid` REST `_search`
/// endpoint (POST with a vendor content type).  Otherwise a standard GET with
/// query-string filters is used.
#[derive(Debug, Clone, Default)]
pub struct FindParams {
    pub query: Vec<(String, String)>,
    pub search: Option<Value>,
}

/// Parameters for requesting a pre-signed upload URL from `ShotGrid`.
#[derive(Debug, Clone, Copy)]
pub struct UploadUrlRequest<'a> {
    pub entity: &'a str,
    pub id: u64,
    pub field_name: &'a str,
    pub file_name: &'a str,
    pub content_type: Option<&'a str>,
    pub multipart_upload: bool,
}

/// Serializable description of an HTTP request that *would* be made.
///
/// Returned by `--dry-run` so agents and operators can inspect the planned
/// request (method, path, body, risk level) without actually executing it.
#[derive(Debug, Clone, Serialize)]
pub struct RequestPlan {
    pub transport: &'static str,
    pub method: &'static str,
    pub path: String,
    pub risk: RiskLevel,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query: Vec<(String, String)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
    #[serde(default, skip_serializing_if = "<[&str]>::is_empty")]
    pub notes: Vec<&'static str>,
}

#[derive(Debug, Clone)]
struct AccessTokenPayload {
    access_token: String,
    token_type: Option<String>,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
struct CachedAccessToken {
    cache_key: String,
    payload: AccessTokenPayload,
    expires_at: Option<Instant>,
}

/// Abstraction over the `ShotGrid`/FPT network transport layer.
///
/// Each method corresponds to a single `ShotGrid` API operation.  The default
/// production implementation is [`RestTransport`], which speaks both the REST
/// API and the legacy JSON-RPC surface.  Test code can supply a recording or
/// stub transport by implementing this trait.
#[async_trait]
pub trait ShotgridTransport {
    async fn auth_test(&self, config: &ConnectionSettings) -> Result<Value>;
    async fn server_info(&self, site: &str) -> Result<Value>;
    async fn schema_entities(&self, config: &ConnectionSettings) -> Result<Value>;
    async fn schema_fields(&self, config: &ConnectionSettings, entity: &str) -> Result<Value>;
    async fn entity_get(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        fields: Option<Vec<String>>,
    ) -> Result<Value>;
    async fn entity_find(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        params: FindParams,
    ) -> Result<Value>;
    async fn entity_summarize(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value>;
    async fn entity_revive(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value>;
    async fn work_schedule_read(&self, config: &ConnectionSettings, body: &Value) -> Result<Value>;
    async fn upload_url(
        &self,
        config: &ConnectionSettings,
        request: UploadUrlRequest<'_>,
    ) -> Result<Value>;
    async fn download_url(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        field_name: &str,
    ) -> Result<Value>;
    async fn thumbnail_url(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value>;
    async fn activity_stream(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn event_log_entries(
        &self,
        config: &ConnectionSettings,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn preferences_get(&self, config: &ConnectionSettings) -> Result<Value>;
    async fn entity_followers(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value>;
    async fn entity_follow(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value>;
    async fn entity_unfollow(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value>;
    async fn note_threads(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn schema_field_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn schema_field_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn schema_field_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value>;
    async fn hierarchy(&self, config: &ConnectionSettings, body: &Value) -> Result<Value>;
    async fn schema_field_read(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value>;
    async fn work_schedule_update(
        &self,
        config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value>;
    async fn text_search(&self, config: &ConnectionSettings, body: &Value) -> Result<Value>;
    async fn note_reply_create(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_relationships(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn user_following(
        &self,
        config: &ConnectionSettings,
        user_id: u64,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn project_update_last_accessed(
        &self,
        config: &ConnectionSettings,
        project_id: u64,
    ) -> Result<Value>;
    async fn schema_entity_read(&self, config: &ConnectionSettings, entity: &str) -> Result<Value>;
    async fn schema_entity_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn schema_entity_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
    ) -> Result<Value>;
    async fn schema_field_revive(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value>;
    async fn schema_entity_create(
        &self,
        config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value>;
    async fn schema_entity_revive(
        &self,
        config: &ConnectionSettings,
        entity: &str,
    ) -> Result<Value>;
    async fn current_user(
        &self,
        config: &ConnectionSettings,
        user_type: &str,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn note_reply_read(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        reply_id: u64,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn filmstrip_thumbnail(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value>;
    async fn preferences_update(&self, config: &ConnectionSettings, body: &Value) -> Result<Value>;
    async fn note_reply_update(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        reply_id: u64,
        body: &Value,
    ) -> Result<Value>;
    async fn note_reply_delete(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        reply_id: u64,
    ) -> Result<Value>;
    async fn entity_relationship_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_relationship_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_share(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_relationship_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn hierarchy_expand(&self, config: &ConnectionSettings, body: &Value) -> Result<Value>;
    async fn schedule_work_day_rules(
        &self,
        config: &ConnectionSettings,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn schedule_work_day_rules_read(
        &self,
        config: &ConnectionSettings,
        rule_id: u64,
    ) -> Result<Value>;
    async fn schedule_work_day_rules_update(
        &self,
        config: &ConnectionSettings,
        rule_id: u64,
        body: &Value,
    ) -> Result<Value>;
    async fn schedule_work_day_rules_create(
        &self,
        config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value>;
    async fn schedule_work_day_rules_delete(
        &self,
        config: &ConnectionSettings,
        rule_id: u64,
    ) -> Result<Value>;
    async fn thumbnail_upload(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value>;
    async fn license(&self, config: &ConnectionSettings) -> Result<Value>;
    async fn preferences_custom_entity(
        &self,
        config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value>;
    /// Fetch the REST API version information from the base endpoint.
    ///
    /// This is an unauthenticated endpoint (`GET /api/{version}/`) that returns
    /// the server and REST API version metadata.
    async fn rest_api_version(&self, site: &str, api_version: &str) -> Result<Value>;
    /// Download the OpenAPI specification for the `ShotGrid` REST API.
    ///
    /// This is an unauthenticated endpoint (`GET /api/{version}/spec.{format}`)
    /// that returns the OpenAPI v3 spec in the requested format (json or yaml).
    async fn openapi_spec(&self, site: &str, api_version: &str, format: &str) -> Result<Value>;

    /// Summarize entity records using the REST `_summarize` endpoint.
    ///
    /// This is the REST-native counterpart to the legacy RPC `summarize` method.
    /// It supports summary_fields, filters, grouping, and
    /// additional_filter_presets via `POST /api/{ver}/entity/{entity}/_summarize`.
    async fn entity_summarize_rest(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value>;

    /// Execute multiple create/update/delete operations in a single server-side
    /// transaction via `POST /api/{ver}/entity/_batch`.
    ///
    /// Each request in the batch array describes one operation.  All operations
    /// succeed or fail together — the server rolls back if any request fails.
    async fn entity_batch_server(&self, config: &ConnectionSettings, body: &Value)
    -> Result<Value>;
}

#[derive(Debug, Clone)]
pub struct RestTransport {
    client: Client,
    token_cache: Arc<Mutex<Option<CachedAccessToken>>>,
}

impl Default for RestTransport {
    fn default() -> Self {
        Self {
            client: Client::new(),
            token_cache: Arc::new(Mutex::new(None)),
        }
    }
}

impl RestTransport {
    /// Returns `true` when `FPT_DEBUG` is set in the environment.
    ///
    /// The result is cached on first access via [`OnceLock`] so that
    /// repeated calls in hot paths avoid redundant env-var lookups.
    fn is_debug() -> bool {
        static CACHED: OnceLock<bool> = OnceLock::new();
        *CACHED.get_or_init(|| std::env::var(ENV_FPT_DEBUG).is_ok())
    }

    fn build_url(
        &self,
        config: &ConnectionSettings,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Url> {
        let mut url = Url::parse(&format!("{}/api/{}/", config.site, config.api_version))
            .map_err(|error| {
                AppError::invalid_input(format!("invalid ShotGrid site URL: {error}"))
                    .with_operation("build_rest_url")
                    .with_invalid_field("site")
                    .with_received_value(&config.site)
                    .with_hint("Provide a valid ShotGrid base URL, for example `https://example.shotgrid.autodesk.com`.")
            })?;
        url = url.join(path.trim_start_matches('/')).map_err(|error| {
            AppError::internal(format!("could not build REST URL `{path}`: {error}"))
                .with_operation("build_rest_url")
                .with_resource(path)
        })?;

        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }

        Ok(url)
    }

    fn build_rpc_url(&self, site: &str) -> Result<Url> {
        let normalized_site = site.trim_end_matches('/');
        let mut url = Url::parse(&format!("{normalized_site}/")).map_err(|error| {
            AppError::invalid_input(format!("invalid ShotGrid site URL: {error}"))
                .with_operation("build_rpc_url")
                .with_invalid_field("site")
                .with_received_value(site)
                .with_hint("Provide a valid ShotGrid base URL, for example `https://example.shotgrid.autodesk.com`.")
        })?;
        url = url.join("api3/json").map_err(|error| {
            AppError::internal(format!("could not build RPC URL: {error}"))
                .with_operation("build_rpc_url")
                .with_resource("api3/json")
        })?;
        Ok(url)
    }

    fn rpc_auth_params(config: &ConnectionSettings) -> Value {
        match &config.credentials {
            Credentials::Script {
                script_name,
                script_key,
            } => json!({
                "script_name": script_name,
                "script_key": script_key,
            }),
            Credentials::UserPassword {
                username,
                password,
                auth_token,
            } => {
                let mut payload = json!({
                    "user_login": username,
                    "user_password": password,
                });
                if let Some(auth_token) = auth_token {
                    payload["auth_token"] = Value::String(auth_token.clone());
                }
                payload
            }
            Credentials::SessionToken { session_token } => json!({
                "session_token": session_token,
                "reject_if_expired": true,
            }),
        }
    }

    fn extract_rpc_results(response: Value) -> Value {
        match response {
            Value::Object(mut map) => map.remove("results").unwrap_or(Value::Object(map)),
            other => other,
        }
    }

    fn token_cache_key(config: &ConnectionSettings) -> String {
        format!(
            "{}|{}|{}|{:?}",
            config.site,
            config.api_version,
            config.auth_mode().grant_type(),
            config.credentials.principal(),
        )
    }

    fn cached_access_token(
        &self,
        config: &ConnectionSettings,
    ) -> Result<Option<AccessTokenPayload>> {
        let cache = self.token_cache.lock().map_err(|_| {
            AppError::internal("token cache is poisoned").with_operation("read_token_cache")
        })?;
        let Some(cached) = cache.as_ref() else {
            return Ok(None);
        };

        if cached.cache_key != Self::token_cache_key(config) {
            return Ok(None);
        }

        if let Some(expires_at) = cached.expires_at
            && Instant::now() >= expires_at
        {
            return Ok(None);
        }

        Ok(Some(cached.payload.clone()))
    }

    fn store_access_token(
        &self,
        config: &ConnectionSettings,
        payload: &AccessTokenPayload,
    ) -> Result<()> {
        let expires_at = payload.expires_in.map(|seconds| {
            let effective_seconds = seconds.saturating_sub(TOKEN_EXPIRY_MARGIN_SECS).max(1);
            Instant::now() + Duration::from_secs(effective_seconds)
        });

        let mut cache = self.token_cache.lock().map_err(|_| {
            AppError::internal(TOKEN_CACHE_POISONED).with_operation("write_token_cache")
        })?;
        *cache = Some(CachedAccessToken {
            cache_key: Self::token_cache_key(config),
            payload: payload.clone(),
            expires_at,
        });
        Ok(())
    }

    /// Retrieve a cached refresh token regardless of access-token expiry.
    ///
    /// Unlike [`cached_access_token`] — which returns `None` when the access
    /// token is expired — this method looks through the cache for a refresh
    /// token even when the access token has already lapsed.  This allows the
    /// caller to attempt a lightweight `refresh_token` grant before falling
    /// back to a full credential-based authentication.
    fn cached_refresh_token(&self, config: &ConnectionSettings) -> Result<Option<String>> {
        let cache = self.token_cache.lock().map_err(|_| {
            AppError::internal(TOKEN_CACHE_POISONED).with_operation("read_token_cache")
        })?;
        let Some(cached) = cache.as_ref() else {
            return Ok(None);
        };
        if cached.cache_key != Self::token_cache_key(config) {
            return Ok(None);
        }
        Ok(cached.payload.refresh_token.clone())
    }

    /// Extract an [`AccessTokenPayload`] from a parsed JSON token response.
    ///
    /// Shared by both [`request_fresh_access_token`] and [`try_refresh_token`]
    /// to avoid duplicating the field-extraction logic.
    fn extract_token_payload(body: &Value) -> Result<AccessTokenPayload> {
        let access_token = body
            .get("access_token")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::auth("ShotGrid access token response is missing `access_token`")
                    .with_operation("extract_token_payload")
                    .with_transport(TRANSPORT_REST)
                    .with_resource("auth/access_token")
                    .with_expected_shape(
                        "a JSON object containing a string field `access_token`",
                    )
                    .with_hint(
                        "Verify the credentials and auth mode, then retry the authentication request.",
                    )
                    .with_detail("response_body", body.clone())
            })?;

        Ok(AccessTokenPayload {
            access_token: access_token.to_string(),
            token_type: body
                .get("token_type")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            expires_in: body.get("expires_in").and_then(Value::as_u64),
            refresh_token: body
                .get("refresh_token")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        })
    }

    /// Attempt a lightweight token refresh using a cached `refresh_token`.
    ///
    /// Returns `Ok(Some(payload))` on success, `Ok(None)` when no refresh
    /// token is available, and silently returns `Ok(None)` on server-side
    /// rejection so the caller can fall back to full credential auth.
    async fn try_refresh_token(
        &self,
        config: &ConnectionSettings,
        debug: bool,
    ) -> Result<Option<AccessTokenPayload>> {
        let refresh_token = match self.cached_refresh_token(config)? {
            Some(rt) => rt,
            None => return Ok(None),
        };

        let url = self.build_url(config, "auth/access_token", &[])?;
        let form: Vec<(&str, &str)> = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
        ];

        if debug {
            eprintln!(
                "[debug] attempting token refresh at {} (refresh_token len={})",
                url,
                refresh_token.len()
            );
        }

        let response = match self
            .client
            .post(url)
            .header(ACCEPT, "application/json")
            .form(&form)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(error) => {
                if debug {
                    eprintln!("[debug] token refresh network error, falling back: {error}");
                }
                return Ok(None);
            }
        };

        // A non-success status means the refresh token was rejected (expired,
        // revoked, or unsupported by the server).  Fall back silently.
        if !response.status().is_success() {
            if debug {
                eprintln!(
                    "[debug] token refresh rejected (HTTP {}), falling back to credential auth",
                    response.status()
                );
            }
            return Ok(None);
        }

        let body = Self::parse_response(response, TRANSPORT_REST).await?;
        let payload = Self::extract_token_payload(&body)?;
        self.store_access_token(config, &payload)?;

        if debug {
            eprintln!("[debug] token refresh succeeded for {}", config.site);
        }

        Ok(Some(payload))
    }

    /// Perform a full credential-based authentication exchange.
    ///
    /// Builds the form parameters from [`ConnectionSettings::credentials`],
    /// POSTs to `auth/access_token`, extracts the payload, and caches it.
    async fn request_fresh_access_token(
        &self,
        config: &ConnectionSettings,
        debug: bool,
    ) -> Result<AccessTokenPayload> {
        let url = self.build_url(config, "auth/access_token", &[])?;

        let mut form: Vec<(&str, &str)> = Vec::new();
        match &config.credentials {
            Credentials::Script {
                script_name,
                script_key,
            } => {
                form.push(("grant_type", "client_credentials"));
                form.push(("client_id", script_name));
                form.push(("client_secret", script_key));
            }
            Credentials::UserPassword {
                username,
                password,
                auth_token,
            } => {
                form.push(("grant_type", "password"));
                form.push(("username", username));
                form.push(("password", password));
                if let Some(token) = auth_token {
                    form.push(("auth_token", token));
                }
            }
            Credentials::SessionToken { session_token } => {
                form.push(("grant_type", "session_token"));
                form.push(("session_token", session_token));
            }
        }

        if debug {
            let masked_form: Vec<String> = form
                .iter()
                .map(|(k, v)| {
                    if *k == "client_secret" || *k == "password" || *k == "session_token" {
                        format!("{k}=***REDACTED(len={})***", v.len())
                    } else {
                        format!("{k}={v}")
                    }
                })
                .collect();
            eprintln!("[debug] POST {} form=[{}]", url, masked_form.join("&"));
        }

        let response = self
            .client
            .post(url)
            .header(ACCEPT, "application/json")
            .form(&form)
            .send()
            .await
            .map_err(|error| {
                AppError::network(format!(
                    "could not request a ShotGrid access token: {error}"
                ))
                .with_operation("request_access_token")
                .with_transport(TRANSPORT_REST)
                .with_resource("auth/access_token")
                .with_retryable_reason(
                    "transient network failure while requesting an access token",
                )
                .retryable(true)
            })?;

        let body = Self::parse_response(response, TRANSPORT_REST).await?;
        let payload = Self::extract_token_payload(&body)?;
        self.store_access_token(config, &payload)?;
        Ok(payload)
    }

    /// Obtain a valid access token, using the fastest available strategy:
    ///
    /// 1. **Cached** — return immediately if a non-expired token is cached.
    /// 2. **Refresh** — attempt a lightweight `refresh_token` grant when a
    ///    cached refresh token exists (best-effort; failure falls through).
    /// 3. **Credential auth** — full `client_credentials` / `password` /
    ///    `session_token` exchange as the final fallback.
    async fn access_token_response(
        &self,
        config: &ConnectionSettings,
    ) -> Result<AccessTokenPayload> {
        let debug = Self::is_debug();

        // 1. Fast path — reuse a cached, non-expired token.
        if let Some(cached) = self.cached_access_token(config)? {
            if debug {
                eprintln!("[debug] reuse cached access token for {}", config.site);
            }
            return Ok(cached);
        }

        // 2. Best-effort refresh — cheaper than a full credential exchange.
        if let Some(refreshed) = self.try_refresh_token(config, debug).await? {
            return Ok(refreshed);
        }

        // 3. Full credential-based authentication.
        self.request_fresh_access_token(config, debug).await
    }

    /// Check whether the response is a 429 rate-limit and, if eligible for
    /// retry, sleep for the appropriate backoff duration.
    ///
    /// Returns `true` when the caller should `continue` its retry loop.
    /// Returns `false` when the response should be parsed normally.
    async fn should_retry_rate_limit(
        status: StatusCode,
        attempt: u32,
        max_attempts: u32,
        debug: bool,
    ) -> bool {
        if status == StatusCode::TOO_MANY_REQUESTS && attempt < max_attempts {
            let delay = Self::backoff_delay(attempt);
            if debug {
                eprintln!(
                    "[debug] rate-limited (429) on attempt {attempt}/{max_attempts}, retrying after {}ms",
                    delay.as_millis()
                );
            }
            tokio::time::sleep(delay).await;
            return true;
        }
        false
    }

    async fn authorized_json_request(
        &self,
        config: &ConnectionSettings,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<&Value>,
    ) -> Result<Value> {
        let debug = Self::is_debug();
        let max_attempts = Self::max_retry_attempts();
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let token = self.access_token_response(config).await?;
            let url = self.build_url(config, path, query)?;
            let mut request = self
                .client
                .request(method.clone(), url)
                .header(ACCEPT, "application/json")
                .bearer_auth(token.access_token);

            if let Some(body) = body {
                request = request.json(body);
            }

            let response = request.send().await.map_err(|error| {
                AppError::network(format!("could not send the ShotGrid REST request: {error}"))
                    .with_operation("authorized_json_request")
                    .with_transport(TRANSPORT_REST)
                    .with_resource(path)
                    .with_retryable_reason("transient network failure while sending a REST request")
                    .retryable(true)
            })?;

            let status = response.status();
            if Self::should_retry_rate_limit(status, attempt, max_attempts, debug).await {
                continue;
            }

            return Self::parse_response(response, TRANSPORT_REST).await;
        }
    }

    async fn authorized_search_request(
        &self,
        config: &ConnectionSettings,
        path: &str,
        query: &[(String, String)],
        body: &Value,
    ) -> Result<Value> {
        let serialized_body = serde_json::to_vec(body).map_err(|error| {
            AppError::internal(format!(
                "could not serialize the `_search` request body as JSON: {error}"
            ))
            .with_operation("serialize_search_request")
            .with_transport(TRANSPORT_REST)
            .with_resource(path)
            .with_expected_shape(
                "a JSON object or array accepted by the ShotGrid `_search` endpoint",
            )
        })?;

        let debug = Self::is_debug();
        let max_attempts = Self::max_retry_attempts();
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let token = self.access_token_response(config).await?;
            let url = self.build_url(config, path, query)?;

            let response = self
                .client
                .request(Method::POST, url)
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, SHOTGRID_SEARCH_CONTENT_TYPE)
                .bearer_auth(token.access_token)
                .body(serialized_body.clone())
                .send()
                .await
                .map_err(|error| {
                    AppError::network(format!(
                        "could not send the ShotGrid REST `_search` request: {error}"
                    ))
                    .with_operation("authorized_search_request")
                    .with_transport(TRANSPORT_REST)
                    .with_resource(path)
                    .with_retryable_reason(
                        "transient network failure while sending a `_search` request",
                    )
                    .retryable(true)
                })?;

            let status = response.status();
            if Self::should_retry_rate_limit(status, attempt, max_attempts, debug).await {
                continue;
            }

            return Self::parse_response(response, TRANSPORT_REST).await;
        }
    }

    /// Returns the maximum number of retry attempts, configurable via `FPT_MAX_RETRIES`.
    ///
    /// The result is cached on first access via [`OnceLock`] so that
    /// repeated calls in retry loops avoid redundant env-var lookups.
    fn max_retry_attempts() -> u32 {
        static CACHED: OnceLock<u32> = OnceLock::new();
        *CACHED.get_or_init(|| {
            std::env::var(ENV_FPT_MAX_RETRIES)
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(MAX_RETRY_ATTEMPTS)
        })
    }

    /// Computes the exponential backoff delay for a given attempt number (1-based).
    ///
    /// Uses deterministic jitter: the base exponential delay is reduced by a
    /// fraction derived from the attempt number, giving spread across retries
    /// without requiring a random source.
    fn backoff_delay(attempt: u32) -> Duration {
        let exp = RETRY_BASE_DELAY_MS.saturating_mul(1u64 << attempt.min(10));
        let capped = exp.min(RETRY_MAX_DELAY_MS);
        // Deterministic jitter: keep between 50%-100% of the capped delay so
        // the backoff is never zero and still provides spread across attempts.
        let jitter_fraction = (u64::from(attempt) * 137 + 42) % 50; // 0..49
        let delay = capped - (capped * jitter_fraction / 100);
        Duration::from_millis(delay.max(RETRY_BASE_DELAY_MS))
    }

    async fn rpc_request(
        &self,
        site: &str,
        method_name: &str,
        params: Vec<Value>,
    ) -> Result<Value> {
        let url = self.build_rpc_url(site)?;
        let body = json!({
            "method_name": method_name,
            "params": params,
        });

        let response = self
            .client
            .request(Method::POST, url)
            .header(ACCEPT, "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|error| {
                AppError::network(format!(
                    "could not send ShotGrid RPC request `{method_name}`: {error}"
                ))
                .with_operation("rpc_request")
                .with_transport(TRANSPORT_RPC)
                .with_resource(method_name)
                .with_retryable_reason("transient network failure while sending an RPC request")
                .retryable(true)
            })?;

        let parsed = Self::parse_response(response, TRANSPORT_RPC).await?;
        Ok(Self::extract_rpc_results(parsed))
    }

    async fn parse_response(response: Response, transport: &'static str) -> Result<Value> {
        let status = response.status();
        let text = response.text().await.map_err(|error| {
            AppError::network(format!(
                "could not read ShotGrid {transport} response body: {error}"
            ))
            .with_operation("parse_response")
            .with_transport(transport)
            .with_retryable_reason("transient network failure while reading the response body")
            .retryable(true)
        })?;

        if status.is_success() {
            if text.trim().is_empty() {
                return Ok(json!({
                    "ok": true,
                    "status": status.as_u16(),
                }));
            }

            return serde_json::from_str(&text).map_err(|error| {
                AppError::api(format!(
                    "ShotGrid {transport} response was not valid JSON: {error}"
                ))
                .with_operation("parse_response")
                .with_transport(transport)
                .with_expected_shape("a valid JSON response body")
                .with_hint("Inspect the raw response body to confirm whether the remote service returned HTML, plain text, or malformed JSON.")
                .with_detail("raw_response", text.clone())
            });
        }

        let details = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
        let is_auth_error = matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
            || (status == StatusCode::BAD_REQUEST && text.contains("authenticate"));

        let error = if is_auth_error {
            AppError::auth(format!(
                "ShotGrid rejected the {transport} request with HTTP status {status}"
            ))
        } else {
            AppError::api(format!(
                "ShotGrid returned HTTP status {status} for the {transport} request"
            ))
        };

        Err(error
            .with_operation("parse_response")
            .with_transport(transport)
            .with_http_status(status.as_u16())
            .with_hint("Inspect the response details and verify the request path, permissions, and payload.")
            .with_detail("response_body", details))
    }
}

#[async_trait]
impl ShotgridTransport for RestTransport {
    async fn auth_test(&self, config: &ConnectionSettings) -> Result<Value> {
        let token = self.access_token_response(config).await?;
        Ok(json!({
            "ok": true,
            "transport": TRANSPORT_REST,
            "profile": config.summary(),
            "grant_type": config.auth_mode().grant_type(),
            "token_received": !token.access_token.is_empty(),
            "token_type": token.token_type,
            "expires_in": token.expires_in,
            "refresh_token_received": token.refresh_token.is_some(),
        }))
    }

    async fn server_info(&self, site: &str) -> Result<Value> {
        self.rpc_request(site, "info", Vec::new()).await
    }

    async fn schema_entities(&self, config: &ConnectionSettings) -> Result<Value> {
        self.authorized_json_request(config, Method::GET, "schema", &[], None)
            .await
    }

    async fn schema_fields(&self, config: &ConnectionSettings, entity: &str) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::GET,
            &format!("schema/{entity}/fields"),
            &[],
            None,
        )
        .await
    }

    async fn entity_get(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        fields: Option<Vec<String>>,
    ) -> Result<Value> {
        let mut query = Vec::new();
        if let Some(fields) = fields.filter(|fields| !fields.is_empty()) {
            query.push(("fields".to_string(), fields.join(",")));
        }

        self.authorized_json_request(
            config,
            Method::GET,
            &entity_instance_path(entity, id),
            &query,
            None,
        )
        .await
    }

    async fn entity_find(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        params: FindParams,
    ) -> Result<Value> {
        let path = format!("entity/{}", entity_collection_path(entity));

        if let Some(search_body) = params.search {
            return self
                .authorized_search_request(
                    config,
                    &format!("{path}/_search"),
                    &params.query,
                    &search_body,
                )
                .await;
        }

        self.authorized_json_request(config, Method::GET, &path, &params.query, None)
            .await
    }

    async fn entity_summarize(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        let mut payload = body.clone();
        payload["type"] = Value::String(entity.to_string());
        self.rpc_request(
            &config.site,
            "summarize",
            vec![Self::rpc_auth_params(config), payload],
        )
        .await
    }

    async fn entity_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::POST,
            &format!("entity/{}", entity_collection_path(entity)),
            &[],
            Some(body),
        )
        .await
    }

    async fn entity_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::PUT,
            &entity_instance_path(entity, id),
            &[],
            Some(body),
        )
        .await
    }

    async fn entity_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::DELETE,
            &entity_instance_path(entity, id),
            &[],
            None,
        )
        .await
    }

    async fn entity_revive(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        let path = entity_instance_path(entity, id);
        self.authorized_json_request(config, Method::POST, &path, &revive_query(), None)
            .await
    }

    async fn work_schedule_read(&self, config: &ConnectionSettings, body: &Value) -> Result<Value> {
        self.rpc_request(
            &config.site,
            "work_schedule_read",
            vec![Self::rpc_auth_params(config), body.clone()],
        )
        .await
    }

    async fn upload_url(
        &self,
        config: &ConnectionSettings,
        request: UploadUrlRequest<'_>,
    ) -> Result<Value> {
        let path = format!(
            "{}/{}/_upload",
            entity_instance_path(request.entity, request.id),
            request.field_name
        );
        let mut query = vec![
            ("filename".to_string(), request.file_name.to_string()),
            (
                "multipart_upload".to_string(),
                request.multipart_upload.to_string(),
            ),
        ];
        if let Some(ct) = request.content_type {
            query.push(("content_type".to_string(), ct.to_string()));
        }
        self.authorized_json_request(config, Method::GET, &path, &query, None)
            .await
    }

    async fn download_url(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        field_name: &str,
    ) -> Result<Value> {
        let path = format!(
            "{}/{}/_download",
            entity_instance_path(entity, id),
            field_name
        );
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn thumbnail_url(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        let path = format!("{}/image", entity_instance_path(entity, id));
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn activity_stream(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        params: &[(String, String)],
    ) -> Result<Value> {
        let path = format!("{}/activity_stream", entity_instance_path(entity, id));
        self.authorized_json_request(config, Method::GET, &path, params, None)
            .await
    }

    async fn event_log_entries(
        &self,
        config: &ConnectionSettings,
        params: &[(String, String)],
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::GET,
            "entity/event_log_entries",
            params,
            None,
        )
        .await
    }

    async fn preferences_get(&self, config: &ConnectionSettings) -> Result<Value> {
        self.authorized_json_request(config, Method::GET, "preferences", &[], None)
            .await
    }

    async fn entity_followers(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        let path = format!("{}/followers", entity_instance_path(entity, id));
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn entity_follow(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value> {
        let path = format!("{}/followers", entity_instance_path(entity, id));
        self.authorized_json_request(config, Method::POST, &path, &[], Some(user))
            .await
    }

    async fn entity_unfollow(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value> {
        let user_id = user.get("id").and_then(Value::as_u64).ok_or_else(|| {
            AppError::invalid_input("user object must contain a numeric `id`")
                .with_operation("entity_unfollow")
                .with_invalid_field("id")
                .with_expected_shape("a user JSON object containing a numeric field `id`")
                .with_detail("received_user", user.clone())
        })?;
        let path = format!("{}/followers/{}", entity_instance_path(entity, id), user_id);
        self.authorized_json_request(config, Method::DELETE, &path, &[], None)
            .await
    }

    async fn note_threads(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        params: &[(String, String)],
    ) -> Result<Value> {
        let path = format!("entity/notes/{note_id}/thread_contents");
        self.authorized_json_request(config, Method::GET, &path, params, None)
            .await
    }

    async fn schema_field_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("schema/{entity}/fields");
        self.authorized_json_request(config, Method::POST, &path, &[], Some(body))
            .await
    }

    async fn schema_field_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("schema/{entity}/fields/{field_name}");
        self.authorized_json_request(config, Method::PUT, &path, &[], Some(body))
            .await
    }

    async fn schema_field_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        let path = format!("schema/{entity}/fields/{field_name}");
        self.authorized_json_request(config, Method::DELETE, &path, &[], None)
            .await
    }

    async fn hierarchy(&self, config: &ConnectionSettings, body: &Value) -> Result<Value> {
        self.authorized_json_request(config, Method::POST, "hierarchy/_search", &[], Some(body))
            .await
    }

    async fn schema_field_read(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        let path = format!("schema/{entity}/fields/{field_name}");
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn work_schedule_update(
        &self,
        config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        self.rpc_request(
            &config.site,
            "work_schedule_update",
            vec![Self::rpc_auth_params(config), body.clone()],
        )
        .await
    }

    async fn text_search(&self, config: &ConnectionSettings, body: &Value) -> Result<Value> {
        self.authorized_json_request(config, Method::POST, "entity/_text_search", &[], Some(body))
            .await
    }

    async fn note_reply_create(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("entity/notes/{note_id}/thread_contents");
        self.authorized_json_request(config, Method::POST, &path, &[], Some(body))
            .await
    }

    async fn entity_relationships(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        params: &[(String, String)],
    ) -> Result<Value> {
        let path = format!(
            "{}/relationships/{}",
            entity_instance_path(entity, id),
            related_field
        );
        self.authorized_json_request(config, Method::GET, &path, params, None)
            .await
    }

    async fn user_following(
        &self,
        config: &ConnectionSettings,
        user_id: u64,
        params: &[(String, String)],
    ) -> Result<Value> {
        let path = format!("entity/human_users/{user_id}/following");
        self.authorized_json_request(config, Method::GET, &path, params, None)
            .await
    }

    async fn project_update_last_accessed(
        &self,
        config: &ConnectionSettings,
        project_id: u64,
    ) -> Result<Value> {
        let path = format!("entity/projects/{project_id}/_update_last_accessed");
        self.authorized_json_request(config, Method::PUT, &path, &[], None)
            .await
    }

    async fn schema_entity_read(&self, config: &ConnectionSettings, entity: &str) -> Result<Value> {
        let path = format!("schema/{entity}");
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn schema_entity_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("schema/{entity}");
        self.authorized_json_request(config, Method::PUT, &path, &[], Some(body))
            .await
    }

    async fn schema_entity_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
    ) -> Result<Value> {
        let path = format!("schema/{entity}");
        self.authorized_json_request(config, Method::DELETE, &path, &[], None)
            .await
    }

    async fn schema_field_revive(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        let path = format!("schema/{entity}/fields/{field_name}");
        self.authorized_json_request(config, Method::POST, &path, &revive_query(), None)
            .await
    }

    async fn schema_entity_create(
        &self,
        config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        self.authorized_json_request(config, Method::POST, "schema", &[], Some(body))
            .await
    }

    async fn schema_entity_revive(
        &self,
        config: &ConnectionSettings,
        entity: &str,
    ) -> Result<Value> {
        let path = format!("schema/{entity}");
        self.authorized_json_request(config, Method::POST, &path, &revive_query(), None)
            .await
    }

    async fn current_user(
        &self,
        config: &ConnectionSettings,
        user_type: &str,
        params: &[(String, String)],
    ) -> Result<Value> {
        let collection = match user_type {
            "ApiUser" | "api_user" | "api" => "api_users",
            _ => "human_users",
        };
        let path = format!("entity/{collection}/current");
        self.authorized_json_request(config, Method::GET, &path, params, None)
            .await
    }

    async fn note_reply_read(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        reply_id: u64,
        params: &[(String, String)],
    ) -> Result<Value> {
        let path = format!("entity/notes/{note_id}/thread_contents/{reply_id}");
        self.authorized_json_request(config, Method::GET, &path, params, None)
            .await
    }

    async fn filmstrip_thumbnail(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        let path = format!("{}/filmstrip_image", entity_instance_path(entity, id));
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn preferences_update(&self, config: &ConnectionSettings, body: &Value) -> Result<Value> {
        self.authorized_json_request(config, Method::PUT, "preferences", &[], Some(body))
            .await
    }

    async fn note_reply_update(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        reply_id: u64,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("entity/notes/{note_id}/thread_contents/{reply_id}");
        self.authorized_json_request(config, Method::PUT, &path, &[], Some(body))
            .await
    }

    async fn note_reply_delete(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        reply_id: u64,
    ) -> Result<Value> {
        let path = format!("entity/notes/{note_id}/thread_contents/{reply_id}");
        self.authorized_json_request(config, Method::DELETE, &path, &[], None)
            .await
    }

    async fn entity_relationship_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!(
            "{}/relationships/{}",
            entity_instance_path(entity, id),
            related_field
        );
        self.authorized_json_request(config, Method::POST, &path, &[], Some(body))
            .await
    }

    async fn entity_relationship_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!(
            "{}/relationships/{}",
            entity_instance_path(entity, id),
            related_field
        );
        self.authorized_json_request(config, Method::PUT, &path, &[], Some(body))
            .await
    }

    async fn entity_share(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("{}/_share", entity_instance_path(entity, id));
        self.authorized_json_request(config, Method::POST, &path, &[], Some(body))
            .await
    }

    async fn entity_relationship_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        related_field: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!(
            "{}/relationships/{}",
            entity_instance_path(entity, id),
            related_field
        );
        self.authorized_json_request(config, Method::DELETE, &path, &[], Some(body))
            .await
    }

    async fn hierarchy_expand(&self, config: &ConnectionSettings, body: &Value) -> Result<Value> {
        self.authorized_json_request(config, Method::POST, "hierarchy/expand", &[], Some(body))
            .await
    }

    async fn schedule_work_day_rules(
        &self,
        config: &ConnectionSettings,
        params: &[(String, String)],
    ) -> Result<Value> {
        self.authorized_json_request(config, Method::GET, "schedule/work_day_rules", params, None)
            .await
    }

    async fn schedule_work_day_rules_update(
        &self,
        config: &ConnectionSettings,
        rule_id: u64,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("schedule/work_day_rules/{rule_id}");
        self.authorized_json_request(config, Method::PUT, &path, &[], Some(body))
            .await
    }

    async fn schedule_work_day_rules_create(
        &self,
        config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::POST,
            "schedule/work_day_rules",
            &[],
            Some(body),
        )
        .await
    }

    async fn schedule_work_day_rules_delete(
        &self,
        config: &ConnectionSettings,
        rule_id: u64,
    ) -> Result<Value> {
        let path = format!("schedule/work_day_rules/{rule_id}");
        self.authorized_json_request(config, Method::DELETE, &path, &[], None)
            .await
    }

    async fn schedule_work_day_rules_read(
        &self,
        config: &ConnectionSettings,
        rule_id: u64,
    ) -> Result<Value> {
        let path = format!("schedule/work_day_rules/{rule_id}");
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn thumbnail_upload(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("{}/image", entity_instance_path(entity, id));
        self.authorized_json_request(config, Method::PUT, &path, &[], Some(body))
            .await
    }

    async fn license(&self, config: &ConnectionSettings) -> Result<Value> {
        self.authorized_json_request(config, Method::GET, "license", &[], None)
            .await
    }

    async fn preferences_custom_entity(
        &self,
        config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::POST,
            "preferences/custom_entity",
            &[],
            Some(body),
        )
        .await
    }

    async fn rest_api_version(&self, site: &str, api_version: &str) -> Result<Value> {
        let normalized_site = site.trim_end_matches('/');
        let url_str = format!("{normalized_site}/api/{api_version}/");
        let url = Url::parse(&url_str).map_err(|error| {
            AppError::invalid_input(format!("invalid ShotGrid site URL: {error}"))
                .with_operation("rest_api_version")
                .with_invalid_field("site")
                .with_received_value(site)
        })?;

        let response = self
            .client
            .request(Method::GET, url)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| {
                AppError::network(format!(
                    "could not fetch the REST API version endpoint: {error}"
                ))
                .with_operation("rest_api_version")
                .with_transport(TRANSPORT_REST)
                .with_resource("/")
                .retryable(true)
            })?;

        Self::parse_response(response, TRANSPORT_REST).await
    }

    async fn openapi_spec(&self, site: &str, api_version: &str, format: &str) -> Result<Value> {
        let normalized_site = site.trim_end_matches('/');
        let url_str = format!("{normalized_site}/api/{api_version}/spec.{format}");
        let url = Url::parse(&url_str).map_err(|error| {
            AppError::invalid_input(format!("invalid ShotGrid site URL: {error}"))
                .with_operation("openapi_spec")
                .with_invalid_field("site")
                .with_received_value(site)
        })?;

        let accept = match format {
            "yaml" | "yml" => "application/x-yaml",
            _ => "application/json",
        };

        let response = self
            .client
            .request(Method::GET, url)
            .header(ACCEPT, accept)
            .send()
            .await
            .map_err(|error| {
                AppError::network(format!(
                    "could not fetch the OpenAPI specification: {error}"
                ))
                .with_operation("openapi_spec")
                .with_transport(TRANSPORT_REST)
                .with_resource(format!("spec.{format}"))
                .retryable(true)
            })?;

        // For YAML responses we wrap the text in a JSON envelope since the
        // CLI output layer expects `Value`.
        if format == "yaml" || format == "yml" {
            let status = response.status();
            let text = response.text().await.map_err(|error| {
                AppError::network(format!("could not read OpenAPI spec response: {error}"))
                    .with_operation("openapi_spec")
            })?;
            if !status.is_success() {
                return Err(AppError::api(format!(
                    "ShotGrid returned HTTP {status} for OpenAPI spec"
                ))
                .with_operation("openapi_spec")
                .with_http_status(status.as_u16()));
            }
            return Ok(json!({
                "format": format,
                "content": text,
            }));
        }

        Self::parse_response(response, TRANSPORT_REST).await
    }

    async fn entity_summarize_rest(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("entity/{}/_summarize", entity_collection_path(entity));
        self.authorized_json_request(config, Method::POST, &path, &[], Some(body))
            .await
    }

    async fn entity_batch_server(
        &self,
        config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        self.authorized_json_request(config, Method::POST, "entity/_batch", &[], Some(body))
            .await
    }
}

/// Build the query parameters used by the schema revive endpoints.
fn revive_query() -> Vec<(String, String)> {
    vec![("revive".to_string(), "true".to_string())]
}

/// Convert a `ShotGrid` entity type name (e.g. `"HumanUser"`, `"CustomEntity01"`)
/// into the plural, snake_case REST collection path segment used by the API
/// (e.g. `"human_users"`, `"custom_entity_01s"`).
///
/// The conversion handles CamelCase word boundaries, digit transitions, hyphens
/// and spaces.  If the resulting string does not already end with `'s'`, one is
/// appended to form the plural.
pub fn entity_collection_path(entity: &str) -> String {
    let entity = entity.trim();
    let chars: Vec<char> = entity.chars().collect();
    let mut output = String::new();

    for (index, ch) in chars.iter().copied().enumerate() {
        let previous = index
            .checked_sub(1)
            .and_then(|value| chars.get(value))
            .copied();
        let next = chars.get(index + 1).copied();

        if ch.is_ascii_uppercase() {
            let should_split = index > 0
                && (previous
                    .is_some_and(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
                    || (previous.is_some_and(|value| value.is_ascii_uppercase())
                        && next.is_some_and(|value| value.is_ascii_lowercase())));

            if should_split {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            continue;
        }

        if ch.is_ascii_digit() {
            if previous.is_some_and(|value| value.is_ascii_alphabetic()) && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch);
            continue;
        }

        if matches!(ch, '-' | ' ') {
            if !output.ends_with('_') {
                output.push('_');
            }
            continue;
        }

        output.push(ch.to_ascii_lowercase());
    }

    if !output.ends_with('s') {
        output.push('s');
    }

    output
}

/// Build the REST path segment for a specific entity instance:
/// `entity/{collection}/{id}`.
///
/// This combines [`entity_collection_path`] with the numeric id, removing the
/// need to repeat the `format!("entity/{}/{}",
/// entity_collection_path(entity), id)` boilerplate across every CRUD method.
fn entity_instance_path(entity: &str, id: u64) -> String {
    format!("entity/{}/{}", entity_collection_path(entity), id)
}

pub(crate) fn plan_entity_create(api_version: &str, entity: &str, body: Value) -> RequestPlan {
    RequestPlan {
        transport: TRANSPORT_REST,
        method: "POST",
        path: format!(
            "/api/{api_version}/entity/{}",
            entity_collection_path(entity)
        ),
        risk: RiskLevel::Write,
        query: Vec::new(),
        body: Some(body),
        notes: vec![DRY_RUN_NOTE],
    }
}

pub(crate) fn plan_entity_update(
    api_version: &str,
    entity: &str,
    id: u64,
    body: Value,
) -> RequestPlan {
    RequestPlan {
        transport: TRANSPORT_REST,
        method: "PUT",
        path: format!(
            "/api/{api_version}/entity/{}/{}",
            entity_collection_path(entity),
            id
        ),
        risk: RiskLevel::Write,
        query: Vec::new(),
        body: Some(body),
        notes: vec![DRY_RUN_NOTE],
    }
}

pub(crate) fn plan_entity_delete(api_version: &str, entity: &str, id: u64) -> RequestPlan {
    RequestPlan {
        transport: TRANSPORT_REST,
        method: "DELETE",
        path: format!(
            "/api/{api_version}/entity/{}/{}",
            entity_collection_path(entity),
            id
        ),
        risk: RiskLevel::Destructive,
        query: Vec::new(),
        body: None,
        notes: vec![
            DRY_RUN_NOTE,
            "actual deletion requires explicit `--yes` flag",
        ],
    }
}

pub(crate) fn plan_entity_revive(api_version: &str, entity: &str, id: u64) -> RequestPlan {
    RequestPlan {
        transport: TRANSPORT_REST,
        method: "POST",
        path: format!(
            "/api/{api_version}/entity/{}/{}",
            entity_collection_path(entity),
            id
        ),
        risk: RiskLevel::Write,
        query: vec![("revive".to_string(), "true".to_string())],
        body: None,
        notes: vec![DRY_RUN_NOTE],
    }
}
