use super::SchemaWalker;
use crate::{EntityDefinitionId, StringId, TypeSystemDirectivesWalker};

pub type EntityWalker<'a> = SchemaWalker<'a, EntityDefinitionId>;

impl<'a> EntityWalker<'a> {
    pub fn id(&self) -> EntityDefinitionId {
        self.item
    }

    pub fn name(&self) -> &'a str {
        match self.item {
            EntityDefinitionId::Object(id) => self.walk(id).name(),
            EntityDefinitionId::Interface(id) => self.walk(id).name(),
        }
    }

    pub fn schema_name_id(&self) -> StringId {
        match self.item {
            EntityDefinitionId::Object(id) => self.walk(id).as_ref().name_id,
            EntityDefinitionId::Interface(id) => self.walk(id).as_ref().name_id,
        }
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        match self.item {
            EntityDefinitionId::Object(id) => self.walk(id).directives(),
            EntityDefinitionId::Interface(id) => self.walk(id).directives(),
        }
    }
}

impl std::fmt::Debug for EntityWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entity").field("name", &self.name()).finish()
    }
}
