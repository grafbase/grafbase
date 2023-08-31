use std::collections::HashMap;

use aws_region_nearby::AwsRegion;
use grafbase_engine::{
    registry::{CacheConfig, CacheControlError},
    AuthConfig, CacheControl,
};
use grafbase_types::{auth::ExecutionAuth, UdfKind};
use serde_with::serde_as;
use worker::{js_sys::Uint8Array, Headers, Method, RequestInit};

pub use grafbase_engine::registry::VersionedRegistry;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct GatewayRequest {
    pub body: Option<Vec<u8>>,
    pub customer_config: CustomerDeploymentConfig,
    pub headers: HashMap<String, String>,
    pub method: String,
    pub url: String,
}

impl TryFrom<GatewayRequest> for worker::Request {
    type Error = worker::Error;

    fn try_from(value: GatewayRequest) -> Result<Self, Self::Error> {
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
pub struct ExecutionRequest {
    /// The request to execute
    pub request: grafbase_engine::Request,
    /// Customer specific configuration needed to execute the request
    pub config: CustomerDeploymentConfig,
    /// Authorization details
    pub auth: ExecutionAuth,

    #[serde(skip)]
    /// AWS Region closest to the worker Colocation
    pub closest_aws_region: rusoto_core::Region,
    /// Request headers
    pub execution_headers: HashMap<String, String>,
}

impl ExecutionRequest {
    /// Parses and validates the request
    pub fn parse(&self) -> Result<CacheControl, CacheControlError> {
        self.config.cache_config.get_cache_control(&self.request)
    }
}

/// Execution health request with the necessary data to perform a health check for a given deployment
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ExecutionHealthRequest {
    /// Customer specific configuration needed to execute the request
    pub config: CustomerDeploymentConfig,
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

// FIXME: Remove after migrating all projects to at least 2023-06-28.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum VecOrMapResolverBindingMap {
    Vec(Vec<((UdfKind, String), String)>),
    HashMap(std::collections::HashMap<String, String>),
}

impl VecOrMapResolverBindingMap {
    fn is_empty(&self) -> bool {
        match self {
            Self::Vec(vec) => vec.is_empty(),
            Self::HashMap(hash_map) => hash_map.is_empty(),
        }
    }
    fn into_iter(self) -> impl Iterator<Item = ((UdfKind, String), String)> {
        let (vec, hash_map) = match self {
            Self::Vec(vec) => (vec, Default::default()),
            Self::HashMap(hash_map) => (Default::default(), hash_map),
        };
        vec.into_iter().chain(
            hash_map
                .into_iter()
                .map(|(key, value)| ((UdfKind::Resolver, key), value)),
        )
    }
}

impl Default for VecOrMapResolverBindingMap {
    fn default() -> Self {
        Self::HashMap(Default::default())
    }
}

// FIXME: Drop `Default` and instantiate this explicitly for local.
/// Encapsulates customer specific configuration
/// Required for executing requests that target a customer deployment
#[serde_as]
#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct CustomerDeploymentConfig {
    /// Grafbase Gateway Version
    #[serde(default)]
    pub gateway_version: String,
    /// Grafbase Deployment ID where this config was generated
    #[serde(default)]
    pub deployment_id: String,
    /// Branch of the project this deployment belongs to
    pub github_ref_name: Option<String>,
    /// Default dynamodb Access Key
    pub globaldb_aws_access_key_id: String,
    /// Default dynamodb Secret Access Key
    pub globaldb_aws_secret_access_key: String,
    /// Default dynamodb Replication Regions
    #[serde(
        deserialize_with = "deserialize_aws_regions",
        serialize_with = "serialize_aws_regions"
    )]
    pub globaldb_dynamodb_replication_regions: Vec<AwsRegion>,
    /// Default dynamodb Table Name
    pub globaldb_dynamodb_table_name: String,
    /// Environment – either `preview` or `production`
    pub grafbase_environment: grafbase::BranchEnvironment,
    /// Plain JWT secret used for JWT validations
    pub jwt_secret: String,
    /// Grafbase project ID this deployment belongs to
    pub project_id: String,
    // FIXME: Remove after migrating all projects to at least 2023-06-28.
    /// Resolver service names
    #[serde(default)]
    pub resolver_bindings: VecOrMapResolverBindingMap,
    /// UDF service names
    #[serde(default)]
    #[serde_as(as = "Vec<(_, _)>")]
    pub udf_bindings: std::collections::HashMap<(UdfKind, String), String>,
    #[serde(default)]
    /// Customer's dedicated subdomain
    pub subdomain: String,
    // FIXME: 2023-04-05: Optional for now until legacy projects are redeployed.
    #[serde(default)]
    pub account_id: Option<String>,
    // FIXME: 2023-04-15: Optional for now until legacy projects are redeployed.
    #[serde(default)]
    pub account_plan: Option<grafbase::Plan>,
    #[serde(default)]
    pub auth_config: AuthConfig,
    #[serde(default)]
    pub cache_config: CacheConfig,
}

impl CustomerDeploymentConfig {
    pub fn all_udf_bindings(&self) -> std::borrow::Cow<'_, std::collections::HashMap<(UdfKind, String), String>> {
        if self.resolver_bindings.is_empty() {
            std::borrow::Cow::Borrowed(&self.udf_bindings)
        } else {
            assert!(self.udf_bindings.is_empty());
            std::borrow::Cow::Owned(self.resolver_bindings.clone().into_iter().collect())
        }
    }
}

impl CustomerDeploymentConfig {
    pub fn account_id(&self) -> String {
        // Unknown account id for now to avoid having nulls as we won't have them for long
        // ACCOUNT#00000000000000000000000000
        self.account_id
            .clone()
            .unwrap_or_else(|| format!("ACCOUNT#{}", ulid::Ulid::from(0)))
    }
}

pub fn deserialize_aws_regions<'de, D>(deserializer: D) -> Result<Vec<AwsRegion>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let comma_separated_list: String = serde::Deserialize::deserialize(deserializer)?;

    comma_separated_list
        .split(',')
        .map(|s| {
            s.parse::<AwsRegion>()
                .map_err(|_| serde::de::Error::custom(format!("Unable to parse {s} to AwsRegion")))
        })
        .collect()
}

pub fn serialize_aws_regions<S>(aws_regions: &[AwsRegion], s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let comma_separated_list = aws_regions
        .iter()
        .map(|region| region.name())
        .collect::<Vec<&str>>()
        .join(",");

    s.serialize_str(&comma_separated_list)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use grafbase_engine::registry::Registry;
    use sha2::{Digest, Sha256};

    use super::*;

    const EXPECTED_SHA: &str = "c46d25099cbf7711bc765cb67b4121803631311a06c66a417546270459028997";

    #[test]
    fn test_serde_roundtrip() {
        let id = r#"
            This test ensures the default `VersionedRegistry` serialization output remains stable.

            When this test fails, it likely means the shape of the `Registry` type was updated,
            which can cause backward-incompatibility issues.

            Before updating this test to match the expected result, please ensure the changes to
            `Registry` are applied in a backward compatible way.

            One way to do so, is to have the `Default` trait return a value that keeps the existing
            expectation, and `#[serde(default)]` is applied to any newly added field.

            Once you are satisfied your changes are backward-compatible, update `EXPECTED_SHA` with
            the new output presented in the test result.
        "#;

        let registry = Cow::Owned(Registry::new().with_sample_data());
        let versioned_registry = VersionedRegistry {
            registry,
            deployment_id: Cow::Borrowed(id),
        };
        let serialized_versioned_registry = serde_json::to_string(&versioned_registry).unwrap();
        let serialized_sha = Sha256::digest(serialized_versioned_registry);

        assert_eq!(&format!("{serialized_sha:x}"), EXPECTED_SHA);
    }

    #[test]
    fn serialize_customer_deployment_config() {
        use std::collections::HashMap;

        use grafbase_types::UdfKind;
        let customer_gateway_config = CustomerDeploymentConfig {
            udf_bindings: HashMap::from([((UdfKind::Authorizer, "name".to_string()), "value".to_string())]),
            ..Default::default()
        };
        serde_json::to_string(&customer_gateway_config).unwrap();
    }
}
