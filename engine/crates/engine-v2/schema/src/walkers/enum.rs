use super::SchemaWalker;
use crate::{EnumDefinitionId, EnumValueId, TypeSystemDirectivesWalker};

pub type EnumDefinitionWalker<'a> = SchemaWalker<'a, EnumDefinitionId>;
pub type EnumValueWalker<'a> = SchemaWalker<'a, EnumValueId>;

impl<'a> EnumDefinitionWalker<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name]
    }

    pub fn values(self) -> impl ExactSizeIterator<Item = EnumValueWalker<'a>> + 'a {
        self.as_ref().value_ids.into_iter().map(move |id| self.walk(id))
    }

    pub fn find_value_by_name(&self, name: &str) -> Option<EnumValueId> {
        let ids = self.as_ref().value_ids;
        self.schema[ids]
            .iter()
            .position(|enum_value| self.schema[enum_value.name].as_str() == name)
            .and_then(|idx| ids.get(idx))
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directives)
    }
}

impl<'a> EnumValueWalker<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name]
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directives)
    }
}

impl<'a> std::fmt::Debug for EnumDefinitionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enum")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("values", &self.values().map(|value| value.name()).collect::<Vec<_>>())
            .finish()
    }
}
