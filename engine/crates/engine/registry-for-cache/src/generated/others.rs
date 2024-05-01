use super::prelude::ids::OtherTypeId;
use super::prelude::*;
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct OtherTypeRecord {
    #[serde(rename = "0")]
    pub name: StringId,
}

#[derive(Clone, Copy)]
pub struct OtherType<'a>(pub(crate) ReadContext<'a, OtherTypeId>);

impl<'a> OtherType<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
}

impl fmt::Debug for OtherType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OtherType").field("name", &self.name()).finish()
    }
}

impl std::cmp::PartialEq for OtherType<'_> {
    fn eq(&self, other: &OtherType<'_>) -> bool {
        std::ptr::eq(self.0.registry, other.0.registry) && self.0.id == other.0.id
    }
}
impl std::cmp::Eq for OtherType<'_> {}

impl RegistryId for OtherTypeId {
    type Reader<'a> = OtherType<'a>;
}

impl IdReader for OtherType<'_> {
    type Id = OtherTypeId;
}

impl<'a> From<ReadContext<'a, OtherTypeId>> for OtherType<'a> {
    fn from(value: ReadContext<'a, OtherTypeId>) -> Self {
        Self(value)
    }
}
