use super::{FieldWalker, SchemaWalker};
use crate::{CacheConfig, InterfaceWalker, ObjectField, ObjectId, RangeWalker};

pub type ObjectWalker<'a> = SchemaWalker<'a, ObjectId>;

impl<'a> ObjectWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.object(self.schema, self.item)
    }

    pub fn fields(&self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        let start = self
            .schema
            .object_fields
            .partition_point(|item| item.object_id < self.item);
        let id = self.item;
        RangeWalker {
            schema: self.schema,
            names: self.names,
            range: &self.schema.object_fields,
            index: start,
            key: move |item: &ObjectField| {
                if item.object_id == id {
                    Some(item.field_id)
                } else {
                    None
                }
            },
        }
    }

    pub fn interfaces(&self) -> impl ExactSizeIterator<Item = InterfaceWalker<'a>> + 'a {
        let walker = *self;
        self.as_ref()
            .interfaces
            .clone()
            .into_iter()
            .map(move |id| walker.walk(id))
    }

    pub fn cache_config(&self) -> Option<CacheConfig> {
        self.as_ref()
            .cache_config
            .map(|cache_config_id| self.schema[cache_config_id])
    }
}

impl<'a> std::fmt::Debug for ObjectWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("fields", &self.fields().map(|f| f.name()).collect::<Vec<_>>())
            .field("interfaces", &self.interfaces().map(|i| i.name()).collect::<Vec<_>>())
            .finish()
    }
}
