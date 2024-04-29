use common_types::auth::Operations;
use gateway_v2_auth_config::v1::AuthConfig;
use indexmap::IndexMap;
use registry_v2::{resolvers::Resolver, CacheControl, Deprecation, FederationProperties, FieldSet};

use crate::{field_types::MetaFieldType, MetaInputValue};

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, Deprecation)]
#[derive(Clone, Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct MetaField {
    pub name: String,
    pub mapped_name: Option<String>,
    pub description: Option<String>,
    pub args: IndexMap<String, MetaInputValue>,
    pub ty: MetaFieldType,
    pub deprecation: Deprecation,
    pub cache_control: Option<Box<CacheControl>>,
    pub requires: Option<FieldSet>,
    pub federation: Option<Box<FederationProperties>>,
    #[serde(skip_serializing_if = "Resolver::is_parent", default)]
    pub resolver: Resolver,
    pub required_operation: Option<Operations>,
    pub auth: Option<Box<AuthConfig>>,
}

impl MetaField {
    pub fn new(name: impl Into<String>, ty: impl Into<MetaFieldType>) -> MetaField {
        MetaField {
            name: name.into(),
            ty: ty.into(),
            ..Default::default()
        }
    }

    pub fn with_cache_control(self, cache_control: Option<Box<CacheControl>>) -> Self {
        Self { cache_control, ..self }
    }

    pub fn target_field_name(&self) -> &str {
        self.mapped_name.as_deref().unwrap_or(&self.name)
    }
}
