use super::prelude::*;
use super::{
    inputs::MetaInputValue,
    prelude::ids::{MetaDirectiveId, MetaInputValueId},
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct MetaDirectiveRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none", default)]
    pub description: Option<StringId>,
    #[serde(rename = "2", skip_serializing_if = "crate::Container::is_empty", default)]
    pub locations: Vec<DirectiveLocation>,
    #[serde(rename = "3", skip_serializing_if = "crate::Container::is_empty", default)]
    pub args: IdRange<MetaInputValueId>,
    #[serde(rename = "4", skip_serializing_if = "crate::is_false", default)]
    pub is_repeatable: bool,
}

#[derive(Clone, Copy)]
pub struct MetaDirective<'a>(pub(crate) ReadContext<'a, MetaDirectiveId>);

impl<'a> MetaDirective<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).description.map(|id| registry.lookup(id))
    }
    pub fn locations(&self) -> impl ExactSizeIterator<Item = DirectiveLocation> + 'a {
        let registry = self.0.registry;
        registry.lookup(self.0.id).locations.iter().copied()
    }
    pub fn args(&self) -> Iter<'a, MetaInputValue<'a>> {
        let registry = self.0.registry;
        Iter::new(registry.lookup(self.0.id).args, registry)
    }
    pub fn is_repeatable(&self) -> bool {
        let registry = self.0.registry;
        registry.lookup(self.0.id).is_repeatable
    }
}

impl fmt::Debug for MetaDirective<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetaDirective")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("locations", &self.locations().collect::<Vec<_>>())
            .field("args", &self.args().collect::<Vec<_>>())
            .field("is_repeatable", &self.is_repeatable())
            .finish()
    }
}

impl std::cmp::PartialEq for MetaDirective<'_> {
    fn eq(&self, other: &MetaDirective<'_>) -> bool {
        std::ptr::eq(self.0.registry, other.0.registry) && self.0.id == other.0.id
    }
}
impl std::cmp::Eq for MetaDirective<'_> {}

impl RegistryId for MetaDirectiveId {
    type Reader<'a> = MetaDirective<'a>;
}

impl IdReader for MetaDirective<'_> {
    type Id = MetaDirectiveId;
}

impl<'a> From<ReadContext<'a, MetaDirectiveId>> for MetaDirective<'a> {
    fn from(value: ReadContext<'a, MetaDirectiveId>) -> Self {
        Self(value)
    }
}
