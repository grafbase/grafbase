use super::{FieldWalker, RangeWalker, SchemaWalker};
use crate::{InterfaceField, InterfaceId, ObjectWalker};

pub type InterfaceWalker<'a> = SchemaWalker<'a, InterfaceId>;

impl<'a> InterfaceWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.interface(self.schema, self.item)
    }

    pub fn fields(&self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        let start = self
            .schema
            .interface_fields
            .partition_point(|item| item.interface_id < self.item);
        let id = self.item;
        RangeWalker {
            schema: self.schema,
            names: self.names,
            range: &self.schema.interface_fields,
            index: start,
            key: move |item: &InterfaceField| {
                if item.interface_id == id {
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

    pub fn possible_types(&self) -> impl ExactSizeIterator<Item = ObjectWalker<'a>> + 'a {
        let walker = *self;
        self.as_ref()
            .possible_types
            .clone()
            .into_iter()
            .map(move |id| walker.walk(id))
    }
}

impl<'a> std::fmt::Debug for InterfaceWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interface")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("fields", &self.fields().map(|f| f.name()).collect::<Vec<_>>())
            .field("interfaces", &self.interfaces().map(|i| i.name()).collect::<Vec<_>>())
            .field(
                "possible_types",
                &self.possible_types().map(|t| t.name()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
