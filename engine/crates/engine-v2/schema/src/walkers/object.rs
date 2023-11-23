use super::{FieldWalker, SchemaWalker};
use crate::{InterfaceWalker, ObjectField, ObjectId, RangeWalker};

pub type ObjectWalker<'a> = SchemaWalker<'a, ObjectId>;

impl<'a> ObjectWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.object(self.id)
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description.map(|id| self.schema[id].as_str())
    }

    pub fn fields(&self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        let start = self
            .schema
            .object_fields
            .partition_point(|item| item.object_id < self.id);
        let id = self.id;
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

    pub fn interfaces(&self) -> impl Iterator<Item = InterfaceWalker<'a>> + 'a {
        let walker = *self;
        self.interfaces.clone().into_iter().map(move |id| walker.walk(id))
    }
}

impl<'a> std::fmt::Debug for ObjectWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectWalker")
            .field("id", &usize::from(self.id))
            .field("name", &self.name())
            .field("description", &self.description())
            .field("fields", &self.fields().map(|f| f.name()).collect::<Vec<_>>())
            .field("interfaces", &self.interfaces().map(|i| i.name()).collect::<Vec<_>>())
            .finish()
    }
}
