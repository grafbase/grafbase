use super::SchemaWalker;
use crate::EntityId;

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
}
