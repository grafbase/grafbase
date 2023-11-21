use super::{FieldWalker, RangeWalker, SchemaWalker};
use crate::{InterfaceField, InterfaceId, ObjectWalker};

pub type InterfaceWalker<'a> = SchemaWalker<'a, InterfaceId>;

impl<'a> InterfaceWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.interface(self.id)
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description.map(|id| self.schema[id].as_str())
    }

    pub fn fields(&self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        let start = self
            .schema
            .interface_fields
            .partition_point(|item| item.interface_id < self.id);
        let id = self.id;
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

    pub fn interfaces(&self) -> impl Iterator<Item = InterfaceWalker<'a>> + 'a {
        let walker = *self;
        self.interfaces.clone().into_iter().map(move |id| walker.walk(id))
    }

    pub fn possible_types(&self) -> impl Iterator<Item = ObjectWalker<'a>> + 'a {
        let walker = *self;
        self.possible_types.clone().into_iter().map(move |id| walker.walk(id))
    }
}

impl<'a> std::fmt::Debug for InterfaceWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<InterfaceWalker<'_>>())
            .field("id", &usize::from(self.id))
            .field("name", &self.name())
            .field("description", &self.description())
            .field("fields", &self.fields().map(|f| f.name()).collect::<Vec<_>>())
            .field("interfaces", &self.interfaces().map(|i| i.name()).collect::<Vec<_>>())
            .field(
                "possible_types",
                &self.possible_types().map(|t| t.name()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
