use std::collections::HashMap;

pub use self::{
    customer_deployment_config::{
        local::LocalSpecificConfig, CommonCustomerDeploymentConfig, CustomerDeploymentConfig,
    },
    execution_engine::{ExecutionEngine, ExecutionError, ExecutionResult, StreamingFormat},
    registry::{RegistryError, RegistryProvider, RegistryResult},
};
pub use engine::registry::VersionedRegistry;

use common_types::{auth::ExecutionAuth, UdfKind};
use engine::{registry::CacheControlError, CacheControl};
use worker::{js_sys::Uint8Array, Headers, Method, RequestInit};

mod customer_deployment_config;
mod execution_engine;
mod registry;

#[cfg(test)]
mod tests;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct GatewayRequest<C> {
    pub body: Option<Vec<u8>>,
    pub customer_config: C,
    pub headers: HashMap<String, String>,
    pub method: String,
    pub url: String,
}

impl<C: serde::de::DeserializeOwned> TryFrom<GatewayRequest<C>> for worker::Request {
    type Error = worker::Error;

    fn try_from(value: GatewayRequest<C>) -> Result<Self, Self::Error> {
        let mut request_init = RequestInit::new();
        request_init
            .with_headers(Headers::from_iter(value.headers))
            .with_method(Method::from(value.method));

        if let Some(customer_body) = value.body {
            request_init.with_body(Some(Uint8Array::from(customer_body.as_slice()).into()));
        }

        worker::Request::new_with_init(value.url.as_str(), &request_init)
    }
}

/// Owned execution request
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ExecutionRequest<C> {
    /// The request to execute
    pub request: engine::Request,
    /// Customer specific configuration needed to execute the request
    pub config: CustomerDeploymentConfig<C>,
    /// Authorization details
    pub auth: ExecutionAuth,

    #[serde(skip)]
    /// AWS Region closest to the worker Colocation
    pub closest_aws_region: rusoto_core::Region,
    /// Request headers
    pub execution_headers: HashMap<String, String>,
}

impl<C> ExecutionRequest<C> {
    /// Parses and validates the request
    pub fn parse(&self) -> Result<CacheControl, CacheControlError> {
        self.config.common.cache_config.get_cache_control(&self.request)
    }
}

/// Execution health request with the necessary data to perform a health check for a given deployment
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ExecutionHealthRequest<C> {
    /// Customer specific configuration needed to execute the request
    pub config: CustomerDeploymentConfig<C>,
    /// Request headers
    #[serde(skip)]
    pub execution_headers: HashMap<String, String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UdfHealthResult {
    pub udf_kind: UdfKind,
    pub udf_name: String,
    pub worker_name: String,
    pub ready: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ExecutionHealthResponse {
    pub deployment_id: String,
    pub ready: bool,
    #[serde(default)]
    pub udf_results: Vec<UdfHealthResult>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ResolverHealthResponse {
    pub ready: bool,
}
