use std::collections::HashMap;

use aws_region_nearby::AwsRegion;
use dynaql::AuthConfig;
use grafbase::auth::ExecutionAuth;
use worker::js_sys::Uint8Array;
use worker::{Headers, Method, RequestInit};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct VersionedRegistry<'a> {
    pub registry: std::borrow::Cow<'a, dynaql::registry::Registry>,
    pub deployment_id: std::borrow::Cow<'a, str>,
}

#[derive(serde::Serialize)]
pub struct ParserResult<'a> {
    pub versioned_registry: VersionedRegistry<'a>,
    pub required_resolvers: Vec<String>,
}

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
    pub request: dynaql::Request,
    /// Customer specific configuration needed to execute the request
    pub config: CustomerDeploymentConfig,
    /// Authorization details
    pub auth: ExecutionAuth,

    #[serde(skip)]
    /// AWS Region closest to the worker Colocation
    pub closest_aws_region: rusoto_core::Region,
    /// Request headers
    #[serde(skip)]
    pub execution_headers: HashMap<String, String>,
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
pub struct ExecutionHealthResponse {
    pub deployment_id: String,
    pub ready: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ResolverHealthResponse {
    pub ready: bool,
}

// FIXME: Drop `Default` and instantiate this explicitly for local.
/// Encapsulates customer specific configuration
/// Required for executing requests that target a customer deployment
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
    /// Resolver service names
    #[serde(default)]
    pub resolver_bindings: std::collections::HashMap<String, String>,
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
    pub caching_enabled: bool,
    #[serde(default)]
    pub auth_config: AuthConfig,
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

    use dynaql::registry::Registry;
    use sha2::{Digest, Sha256};

    use super::*;

    const EXPECTED_SHA: &str = "7d3e71061a604ae16f13eb0a71c7f6d983992d7bca8525cdeb3613a0608e4910";

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
}
