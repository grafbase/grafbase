use super::prelude::ids::ScalarTypeId;
use super::prelude::*;
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct ScalarTypeRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none", default)]
    pub description: Option<StringId>,
    #[serde(rename = "2", skip_serializing_if = "Option::is_none", default)]
    pub specified_by_url: Option<StringId>,
    #[serde(rename = "3")]
    pub parser: ScalarParser,
}

#[derive(Clone, Copy)]
pub struct ScalarType<'a>(pub(crate) ReadContext<'a, ScalarTypeId>);

impl<'a> ScalarType<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).description.map(|id| registry.lookup(id))
    }
    pub fn specified_by_url(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry
            .lookup(self.0.id)
            .specified_by_url
            .map(|id| registry.lookup(id))
    }
    pub fn parser(&self) -> ScalarParser {
        let registry = self.0.registry;
        registry.lookup(self.0.id).parser
    }
}

impl fmt::Debug for ScalarType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScalarType")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("specified_by_url", &self.specified_by_url())
            .field("parser", &self.parser())
            .finish()
    }
}

impl std::cmp::PartialEq for ScalarType<'_> {
    fn eq(&self, other: &ScalarType<'_>) -> bool {
        std::ptr::eq(self.0.registry, other.0.registry) && self.0.id == other.0.id
    }
}
impl std::cmp::Eq for ScalarType<'_> {}

impl RegistryId for ScalarTypeId {
    type Reader<'a> = ScalarType<'a>;
}

impl IdReader for ScalarType<'_> {
    type Id = ScalarTypeId;
}

impl<'a> From<ReadContext<'a, ScalarTypeId>> for ScalarType<'a> {
    fn from(value: ReadContext<'a, ScalarTypeId>) -> Self {
        Self(value)
    }
}
