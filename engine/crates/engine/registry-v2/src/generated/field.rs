use super::prelude::*;
use super::{
    inputs::MetaInputValue,
    prelude::ids::{MetaFieldId, MetaInputValueId},
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct MetaFieldRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none", default)]
    pub mapped_name: Option<StringId>,
    #[serde(rename = "2", skip_serializing_if = "Option::is_none", default)]
    pub description: Option<StringId>,
    #[serde(rename = "3", skip_serializing_if = "crate::Container::is_empty", default)]
    pub args: IdRange<MetaInputValueId>,
    #[serde(rename = "4")]
    pub ty: MetaFieldTypeRecord,
    #[serde(rename = "5", skip_serializing_if = "Option::is_none", default)]
    pub deprecation: Option<Box<Deprecation>>,
    #[serde(rename = "6", skip_serializing_if = "Option::is_none", default)]
    pub cache_control: Option<Box<CacheControl>>,
    #[serde(rename = "7", skip_serializing_if = "Option::is_none", default)]
    pub requires: Option<Box<FieldSet>>,
    #[serde(rename = "8", skip_serializing_if = "Option::is_none", default)]
    pub federation: Option<Box<FederationProperties>>,
    #[serde(rename = "9")]
    pub resolver: Resolver,
    #[serde(rename = "10", skip_serializing_if = "Option::is_none", default)]
    pub required_operation: Option<Box<Operations>>,
    #[serde(rename = "11", skip_serializing_if = "Option::is_none", default)]
    pub auth: Option<Box<AuthConfig>>,
}

#[derive(Clone, Copy)]
pub struct MetaField<'a>(pub(crate) ReadContext<'a, MetaFieldId>);

impl<'a> MetaField<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn mapped_name(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).mapped_name.map(|id| registry.lookup(id))
    }
    pub fn description(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).description.map(|id| registry.lookup(id))
    }
    pub fn args(&self) -> Iter<'a, MetaInputValue<'a>> {
        let registry = self.0.registry;
        Iter::new(registry.lookup(self.0.id).args, registry)
    }
    pub fn deprecation(&self) -> Option<&'a Deprecation> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).deprecation.as_deref()
    }
    pub fn cache_control(&self) -> Option<&'a CacheControl> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).cache_control.as_deref()
    }
    pub fn requires(&self) -> Option<&'a FieldSet> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).requires.as_deref()
    }
    pub fn federation(&self) -> Option<&'a FederationProperties> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).federation.as_deref()
    }
    pub fn resolver(&self) -> &'a Resolver {
        let registry = self.0.registry;
        &registry.lookup(self.0.id).resolver
    }
    pub fn required_operation(&self) -> Option<&'a Operations> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).required_operation.as_deref()
    }
    pub fn auth(&self) -> Option<&'a AuthConfig> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).auth.as_deref()
    }
}

impl fmt::Debug for MetaField<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetaField")
            .field("name", &self.name())
            .field("mapped_name", &self.mapped_name())
            .field("description", &self.description())
            .field("args", &self.args().collect::<Vec<_>>())
            .field("ty", &self.ty())
            .field("deprecation", &self.deprecation())
            .field("cache_control", &self.cache_control())
            .field("requires", &self.requires())
            .field("federation", &self.federation())
            .field("resolver", &self.resolver())
            .field("required_operation", &self.required_operation())
            .field("auth", &self.auth())
            .finish()
    }
}

impl RegistryId for MetaFieldId {
    type Reader<'a> = MetaField<'a>;
}

impl IdReader for MetaField<'_> {
    type Id = MetaFieldId;
}

impl<'a> From<ReadContext<'a, MetaFieldId>> for MetaField<'a> {
    fn from(value: ReadContext<'a, MetaFieldId>) -> Self {
        Self(value)
    }
}
