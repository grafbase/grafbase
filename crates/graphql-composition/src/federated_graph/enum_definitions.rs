use super::{Directive, EnumDefinitionId, FederatedGraph, StringId};

pub type EnumDefinition<'a> = super::view::ViewNested<'a, EnumDefinitionId, EnumDefinitionRecord>;

impl std::fmt::Debug for EnumDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumDefinition")
            .field("name", &self.then(|ty| ty.name).as_str())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct EnumDefinitionRecord {
    pub(crate) namespace: Option<StringId>,
    pub(crate) name: StringId,
    pub(crate) directives: Vec<Directive>,
    pub(crate) description: Option<StringId>,
}

impl FederatedGraph {
    pub fn push_enum_definition(&mut self, enum_definition: EnumDefinitionRecord) -> EnumDefinitionId {
        let id = self.enum_definitions.len().into();
        self.enum_definitions.push(enum_definition);
        id
    }
}
