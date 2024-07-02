use super::SchemaWalker;
use crate::{EntityId, TypeSystemDirectivesWalker};

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

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        match self.item {
            EntityId::Object(id) => self.walk(id).directives(),
            EntityId::Interface(id) => self.walk(id).directives(),
        }
    }
}
