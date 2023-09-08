use grafbase_engine::{registry::CacheConfig, AuthConfig};
use common_types::UdfKind;

pub mod local;

#[derive(
    serde_with::DeserializeFromStr,
    serde_with::SerializeDisplay,
    Clone,
    Copy,
    Eq,
    Hash,
    PartialEq,
    Debug,
    Default,
    strum::Display,
    strum::EnumString,
)]
#[strum(serialize_all = "lowercase")]
pub enum BranchEnvironment {
    #[default]
    Preview,
    Production,
}

/// Encapsulates customer specific configuration
/// Required for executing requests that target a customer deployment
#[serde_with::serde_as]
#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct CommonCustomerDeploymentConfig {
    /// Grafbase Gateway Version
    #[serde(default)]
    pub gateway_version: String,
    /// Grafbase Deployment ID where this config was generated
    #[serde(default)]
    pub deployment_id: String,
    /// Branch of the project this deployment belongs to
    pub github_ref_name: Option<String>,
    /// Environment – either `preview` or `production`
    pub grafbase_environment: BranchEnvironment,
    /// Plain JWT secret used for JWT validations
    pub jwt_secret: String,
    /// Grafbase project ID this deployment belongs to
    pub project_id: String,
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
    #[serde(default)]
    pub auth_config: AuthConfig,
    #[serde(default)]
    pub cache_config: CacheConfig,
}

/// Encapsulates customer specific configuration
/// Required for executing requests that target a customer deployment
#[serde_with::serde_as]
#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct CustomerDeploymentConfig<RuntimeSpecificConfig> {
    #[serde(flatten)]
    pub common: CommonCustomerDeploymentConfig,
    #[serde(flatten)]
    pub runtime_specific_config: RuntimeSpecificConfig,
}

impl CommonCustomerDeploymentConfig {
    pub fn account_id(&self) -> String {
        // Unknown account id for now to avoid having nulls as we won't have them for long
        // ACCOUNT#00000000000000000000000000
        self.account_id
            .clone()
            .unwrap_or_else(|| format!("ACCOUNT#{}", ulid::Ulid::from(0)))
    }
}
