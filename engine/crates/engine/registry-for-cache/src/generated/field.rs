use super::prelude::ids::MetaFieldId;
use super::prelude::*;
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct MetaFieldRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1")]
    pub ty: MetaFieldTypeRecord,
    #[serde(rename = "2", skip_serializing_if = "Option::is_none", default)]
    pub cache_control: Option<Box<CacheControl>>,
}

#[derive(Clone, Copy)]
pub struct MetaField<'a>(pub(crate) ReadContext<'a, MetaFieldId>);

impl<'a> MetaField<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn cache_control(&self) -> Option<&'a CacheControl> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).cache_control.as_deref()
    }
}

impl fmt::Debug for MetaField<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetaField")
            .field("name", &self.name())
            .field("ty", &self.ty())
            .field("cache_control", &self.cache_control())
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
