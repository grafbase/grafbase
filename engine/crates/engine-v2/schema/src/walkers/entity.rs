use super::SchemaWalker;
use crate::{EntityId, StringId, TypeSystemDirectivesWalker};

pub type EntityWalker<'a> = SchemaWalker<'a, EntityId>;

impl<'a> EntityWalker<'a> {
    pub fn id(&self) -> EntityId {
        self.item
    }

    pub fn name(&self) -> &'a str {
        match self.item {
            EntityId::Object(id) => self.walk(id).name(),
            EntityId::Interface(id) => self.walk(id).name(),
        }
    }

    pub fn schema_name_id(&self) -> StringId {
        match self.item {
            EntityId::Object(id) => self.walk(id).as_ref().name,
            EntityId::Interface(id) => self.walk(id).as_ref().name,
        }
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        match self.item {
            EntityId::Object(id) => self.walk(id).directives(),
            EntityId::Interface(id) => self.walk(id).directives(),
        }
    }
}
