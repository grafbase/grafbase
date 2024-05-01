use std::collections::{BTreeMap, HashMap, HashSet};

mod cache_pruning;
mod constraint;
mod directives;
mod enums;
mod field_types;
mod fields;
mod input_value;
mod registry_impl;
mod serde_preserve_enum;
mod types;

use gateway_v2_auth_config::v1::AuthConfig;
use postgres_connector_types::database_definition::DatabaseDefinition;
use registry_v2::{
    CodegenConfig, ConnectorHeaders, CorsConfig, FederationEntity, MongoDBConfiguration, OperationLimits,
    TrustedDocuments,
};

pub use registry_v2::resolvers;

pub use constraint::*;
pub use directives::*;
pub use enums::*;
pub use field_types::*;
pub use fields::*;
pub use input_value::*;
pub use types::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Registry {
    pub types: BTreeMap<String, MetaType>,
    pub directives: HashMap<String, MetaDirective>,
    pub implements: HashMap<String, HashSet<String>>,
    pub query_type: String,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
    pub disable_introspection: bool,
    pub enable_federation: bool,
    pub federation_subscription: bool,
    pub auth: AuthConfig,
    #[serde(default)]
    pub mongodb_configurations: HashMap<String, MongoDBConfiguration>,
    #[serde(default)]
    pub http_headers: BTreeMap<String, ConnectorHeaders>,
    #[serde(default)]
    pub postgres_databases: HashMap<String, DatabaseDefinition>,
    #[serde(default)]
    pub enable_caching: bool,
    #[serde(default)]
    pub enable_kv: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub federation_entities: BTreeMap<String, FederationEntity>,
    #[serde(default)]
    pub enable_ai: bool,
    #[serde(default)]
    pub enable_codegen: bool,
    // FIXME: Make an enum.
    pub is_federated: bool,
    #[serde(default)]
    pub operation_limits: OperationLimits,
    #[serde(default)]
    pub trusted_documents: Option<TrustedDocuments>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codegen: Option<CodegenConfig>,
    #[serde(default)]
    pub cors_config: Option<CorsConfig>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            types: Default::default(),
            directives: Default::default(),
            implements: Default::default(),
            query_type: "Query".to_string(),
            mutation_type: None,
            subscription_type: None,
            disable_introspection: false,
            enable_federation: false,
            federation_subscription: false,
            auth: Default::default(),
            mongodb_configurations: Default::default(),
            http_headers: Default::default(),
            postgres_databases: Default::default(),
            enable_caching: false,
            enable_kv: false,
            federation_entities: Default::default(),
            enable_ai: false,
            enable_codegen: false,
            is_federated: false,
            operation_limits: Default::default(),
            trusted_documents: Default::default(),
            cors_config: Default::default(),
            codegen: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use registry_v2::{resolvers::Resolver, CacheControl};

    pub use super::*;

    #[test]
    fn types_should_have_reasonable_sizes() {
        // We do some testing on the exact size of these.
        // If the size goes up think very carefully about it.
        // If it goes down - yay, just update the test so we can keep the new low water mark.

        assert_eq!(std::mem::size_of::<ObjectType>(), 184);
        assert_eq!(std::mem::size_of::<InterfaceType>(), 232);
        assert_eq!(std::mem::size_of::<MetaType>(), 232);

        assert_eq!(std::mem::size_of::<MetaField>(), 304);

        assert_eq!(std::mem::size_of::<CacheControl>(), 80);

        assert_eq!(std::mem::size_of::<MetaInputValue>(), 200);

        assert_eq!(std::mem::size_of::<Resolver>(), 56);
    }
}
