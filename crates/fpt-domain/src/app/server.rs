use fpt_core::Result;
use serde_json::Value;

use crate::config::{ConnectionOverrides, api_version_or_default, resolve_site};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn server_info(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let site = resolve_site(overrides)?;
        self.transport.server_info(&site).await
    }

    /// Fetch the REST API version information.
    ///
    /// Calls `GET /api/{version}/` which is an unauthenticated endpoint that
    /// returns the `ShotGrid` server version and REST API version metadata.
    pub async fn rest_api_version(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let site = resolve_site(overrides.clone())?;
        let api_version = api_version_or_default(overrides.api_version.as_deref());
        self.transport.rest_api_version(&site, &api_version).await
    }

    /// Download the OpenAPI specification for the `ShotGrid` REST API.
    ///
    /// Calls `GET /api/{version}/spec.{format}` which is an unauthenticated
    /// endpoint returning the OpenAPI v3 spec in JSON or YAML format.
    pub async fn openapi_spec(
        &self,
        overrides: ConnectionOverrides,
        format: &str,
    ) -> Result<Value> {
        let site = resolve_site(overrides.clone())?;
        let api_version = api_version_or_default(overrides.api_version.as_deref());
        self.transport
            .openapi_spec(&site, &api_version, format)
            .await
    }
}
