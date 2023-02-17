use aws_region_nearby::AwsRegion;
use std::collections::HashSet;

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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ExecutionAuth {
    pub allowed_ops: dynaql::Operations,
    pub groups_from_token: Option<HashSet<String>>,
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

/// Encapsulates customer specific configuration
/// Required for executing requests that target a customer deployment
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct CustomerDeploymentConfig {
    /// Grafbase API Version
    #[serde(default)]
    pub api_version: String,
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
    #[serde(default)]
    /// Customer's dedicated subdomain
    pub subdomain: String,
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
