use super::{Directives, FederatedGraph, StringId, TypeDefinitionId};

#[derive(Clone, Debug)]
pub struct TypeDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringId>,
    pub directives: Directives,
}

impl FederatedGraph {
    pub fn push_type_definition(&mut self, type_def: TypeDefinitionRecord) -> TypeDefinitionId {
        let id = self.type_definitions.len().into();
        self.type_definitions.push(type_def);
        id
    }
}
