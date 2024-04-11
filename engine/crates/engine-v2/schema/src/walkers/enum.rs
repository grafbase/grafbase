use super::SchemaWalker;
use crate::{EnumId, EnumValueId, TypeSystemDirectivesWalker};

pub type EnumWalker<'a> = SchemaWalker<'a, EnumId>;
pub type EnumValueWalker<'a> = SchemaWalker<'a, EnumValueId>;

impl<'a> EnumWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.r#enum(self.schema, self.item)
    }

    pub fn values(self) -> impl ExactSizeIterator<Item = EnumValueWalker<'a>> + 'a {
        self.as_ref().value_ids.map(move |id| self.walk(id))
    }

    pub fn find_value_by_name(&self, name: &str) -> Option<EnumValueId> {
        let ids = self.as_ref().value_ids;
        self.schema[ids]
            .binary_search_by(|enum_value| self.schema[enum_value.name].as_str().cmp(name))
            .ok()
            .map(EnumValueId::from)
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directives)
    }
}

impl<'a> EnumValueWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.enum_value(self.schema, self.item)
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directives)
    }
}

impl<'a> std::fmt::Debug for EnumWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enum")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("values", &self.values().map(|value| value.name()).collect::<Vec<_>>())
            .finish()
    }
}
