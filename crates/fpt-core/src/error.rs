use serde::Serialize;
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, AppError>;

/// Categorized error codes for structured CLI error output.
///
/// Each variant maps to a stable `SCREAMING_SNAKE_CASE` string and a distinct
/// process exit code (spaced by 10 for future extensibility).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Caller-provided input failed validation (bad JSON, missing fields, etc.).
    InvalidInput,
    /// Authentication or authorization against `ShotGrid` failed.
    AuthFailed,
    /// A network-level error occurred (DNS, TLS, timeout, connection refused).
    NetworkError,
    /// The remote `ShotGrid` API returned a non-success HTTP status.
    ApiError,
    /// A safety policy (e.g. missing `--yes` on destructive ops) blocked execution.
    PolicyBlocked,
    /// The requested capability exists but is not available for this transport.
    UnsupportedCapability,
    /// The requested capability is recognized but not yet implemented.
    NotImplemented,
    /// An unexpected internal error (bug, serialization failure, etc.).
    InternalError,
}

impl ErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "INVALID_INPUT",
            Self::AuthFailed => "AUTH_FAILED",
            Self::NetworkError => "NETWORK_ERROR",
            Self::ApiError => "API_ERROR",
            Self::PolicyBlocked => "POLICY_BLOCKED",
            Self::UnsupportedCapability => "UNSUPPORTED_CAPABILITY",
            Self::NotImplemented => "NOT_IMPLEMENTED",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }

    /// Maps each error code to a distinct process exit code.
    ///
    /// Exit codes are spaced by 10 so that future codes can be inserted
    /// without breaking existing scripts that pattern-match on exit status.
    ///
    /// | Code                    | Exit |
    /// |-------------------------|------|
    /// | `InvalidInput`          |  10  |
    /// | `AuthFailed`            |  20  |
    /// | `NetworkError`          |  30  |
    /// | `ApiError`              |  40  |
    /// | `PolicyBlocked`         |  50  |
    /// | `UnsupportedCapability` |  60  |
    /// | `NotImplemented`        |  61  |
    /// | `InternalError`         |  70  |
    #[must_use]
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::InvalidInput => 10,
            Self::AuthFailed => 20,
            Self::NetworkError => 30,
            Self::ApiError => 40,
            Self::PolicyBlocked => 50,
            Self::UnsupportedCapability => 60,
            Self::NotImplemented => 61,
            Self::InternalError => 70,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorEnvelope {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppError {
    code: ErrorCode,
    message: String,
    details: Option<Value>,
    retryable: bool,
    transport: Option<String>,
}

impl AppError {
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: format_error_message(code, &message.into()),
            details: None,
            retryable: false,
            transport: None,
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::AuthFailed, message)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NetworkError, message)
    }

    pub fn api(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ApiError, message)
    }

    pub fn policy_blocked(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::PolicyBlocked, message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::UnsupportedCapability, message)
    }

    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotImplemented, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, message)
    }

    #[must_use]
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    #[must_use]
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let key = key.into();
        let value = serde_json::to_value(value).unwrap_or_else(|error| {
            Value::String(format!("failed to serialize detail value: {error}"))
        });

        let mut object = match self.details.take() {
            Some(Value::Object(existing)) => existing,
            Some(other) => {
                let mut map = Map::new();
                map.insert("context".to_string(), other);
                map
            }
            None => Map::new(),
        };
        object.insert(key, value);
        self.details = Some(Value::Object(object));

        self
    }

    #[must_use]
    pub fn with_hint(self, hint: impl Into<String>) -> Self {
        self.with_detail("hint", hint.into())
    }

    #[must_use]
    pub fn with_expected_shape(self, expected_shape: impl Into<String>) -> Self {
        self.with_detail("expected_shape", expected_shape.into())
    }

    #[must_use]
    pub fn with_invalid_field(self, field_name: impl Into<String>) -> Self {
        self.with_detail("invalid_field", field_name.into())
    }

    #[must_use]
    pub fn with_input_source(self, input_source: impl Into<String>) -> Self {
        self.with_detail("input_source", input_source.into())
    }

    #[must_use]
    pub fn with_missing_fields<I, S>(self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.with_detail(
            "missing_fields",
            fields.into_iter().map(Into::into).collect::<Vec<_>>(),
        )
    }

    #[must_use]
    pub fn with_conflicting_fields<I, S>(self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.with_detail(
            "conflicting_fields",
            fields.into_iter().map(Into::into).collect::<Vec<_>>(),
        )
    }

    #[must_use]
    pub fn with_allowed_values<I, S>(self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.with_detail(
            "allowed_values",
            values.into_iter().map(Into::into).collect::<Vec<_>>(),
        )
    }

    #[must_use]
    pub fn with_operation(self, operation: impl Into<String>) -> Self {
        self.with_detail("operation", operation.into())
    }

    #[must_use]
    pub fn with_resource(self, resource: impl Into<String>) -> Self {
        self.with_detail("resource", resource.into())
    }

    #[must_use]
    pub fn with_http_status(self, status: u16) -> Self {
        self.with_detail("http_status", status)
    }

    #[must_use]
    pub fn with_retryable_reason(self, reason: impl Into<String>) -> Self {
        self.with_detail("retryable_reason", reason.into())
    }

    #[must_use]
    pub fn with_received_value(self, value: impl Serialize) -> Self {
        self.with_detail("received", value)
    }

    #[must_use]
    pub fn with_transport(mut self, transport: impl Into<String>) -> Self {
        self.transport = Some(transport.into());
        self
    }

    #[must_use]
    pub const fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    #[must_use]
    pub fn envelope(&self) -> ErrorEnvelope {
        ErrorEnvelope {
            code: self.code.as_str(),
            message: self.message.clone(),
            details: self.details.clone(),
            retryable: self.retryable,
            transport: self.transport.clone(),
        }
    }

    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }
}

fn format_error_message(code: ErrorCode, message: &str) -> String {
    let prefix = match code {
        ErrorCode::InvalidInput => "Input validation failed",
        ErrorCode::AuthFailed => "Authentication failed",
        ErrorCode::NetworkError => "Network request failed",
        ErrorCode::ApiError => "Remote API request failed",
        ErrorCode::PolicyBlocked => "Operation blocked by safety policy",
        ErrorCode::UnsupportedCapability => "Unsupported capability",
        ErrorCode::NotImplemented => "Capability not implemented",
        ErrorCode::InternalError => "Internal execution error",
    };

    let trimmed = message.trim();
    if trimmed.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}: {trimmed}")
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}
