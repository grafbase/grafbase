use aws_region_nearby::AwsRegion;
use grafbase::auth::Operations;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use worker::js_sys::Uint8Array;
use worker::{Headers, Method, RequestInit};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct VersionedRegistry<'a> {
    pub registry: std::borrow::Cow<'a, dynaql::registry::Registry>,
    pub deployment_id: std::borrow::Cow<'a, str>,
}

#[derive(serde::Serialize)]
pub struct VersionedRegistrySerializable<'a> {
    pub registry: serde_json::Value,
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

/// Self-contained execution request
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ExecutionRequest<'a> {
    /// The versioned registry to be used when executing the request
    pub versioned_registry: VersionedRegistry<'a>,
    /// The request to execute
    pub request: dynaql::Request,
    /// Customer specific configuration needed to execute the request
    pub config: CustomerDeploymentConfig,
    /// Authorization details
    pub auth: ExecutionAuth,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]

pub enum ExecutionAuth {
    ApiKey,
    Token(ExecutionAuthToken),
}

impl ExecutionAuth {
    pub fn new_from_api_keys() -> Self {
        Self::ApiKey
    }

    pub fn new_from_token(
        private_and_group_ops: Operations,
        groups_from_token: HashSet<String>,
        subject_and_owner_ops: Option<(String, Operations)>,
    ) -> Self {
        let allowed_owner_ops = subject_and_owner_ops.as_ref().map(|it| it.1).unwrap_or_default();
        let global_ops = private_and_group_ops.union(allowed_owner_ops);
        Self::Token(ExecutionAuthToken {
            global_ops,
            private_and_group_ops,
            groups_from_token,
            subject_and_owner_ops,
        })
    }

    pub fn global_ops(&self) -> Operations {
        match self {
            Self::ApiKey => dynaql::AuthConfig::api_key_ops(),
            Self::Token(token) => token.global_ops,
        }
    }

    pub fn hash<H: Hasher + Default>(&self) -> u64 {
        match self {
            Self::ApiKey => {
                let hasher = H::default();
                hasher.finish()
            }
            Self::Token(token) => token.hash::<H>(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ExecutionAuthToken {
    /// API key or group based operations that are enabled on the global level.
    global_ops: Operations,
    private_and_group_ops: Operations,
    groups_from_token: HashSet<String>,
    /// Owner's subject and enabled operations on the global level.
    subject_and_owner_ops: Option<(String, Operations)>,
}

impl ExecutionAuthToken {
    pub fn global_ops(&self) -> Operations {
        self.global_ops
    }

    pub fn groups_from_token(&self) -> &HashSet<String> {
        &self.groups_from_token
    }

    pub fn subject_and_owner_ops(&self) -> Option<&(String, Operations)> {
        self.subject_and_owner_ops.as_ref()
    }

    pub fn hash<H: Hasher + Default>(&self) -> u64 {
        let mut hasher = H::default();
        self.global_ops.hash(&mut hasher);
        self.subject_and_owner_ops.hash(&mut hasher);
        hasher.write_usize(self.groups_from_token.len());
        let mut h: u64 = 0;
        // opted for XORing the hashes of the elements instead of sorting
        for group in &self.groups_from_token {
            let mut hasher = H::default();
            group.hash(&mut hasher);
            h ^= hasher.finish();
        }
        hasher.write_u64(h);
        hasher.finish()
    }
}

/// Execution health request with the necessary data to perform a health check for a given deployment
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ExecutionHealthRequest {
    pub config: CustomerDeploymentConfig,
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
    // FIXME: 2023-04-05: Optional for now for legacy projects being deployed. Change me back to a
    // String later on.
    #[serde(default)]
    pub account_id: Option<String>,
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
