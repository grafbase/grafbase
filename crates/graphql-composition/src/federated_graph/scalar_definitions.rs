use super::{Directive, FederatedGraph, ScalarDefinitionId, StringId};

pub type ScalarDefinition<'a> = super::view::ViewNested<'a, ScalarDefinitionId, ScalarDefinitionRecord>;

impl std::fmt::Debug for ScalarDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScalarDefinition")
            .field("name", &self.then(|ty| ty.name).as_str())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct ScalarDefinitionRecord {
    pub(crate) namespace: Option<StringId>,
    pub(crate) name: StringId,
    pub(crate) directives: Vec<Directive>,
    pub(crate) description: Option<StringId>,
}

impl FederatedGraph {
    pub fn push_scalar_definition(&mut self, scalar_definition: ScalarDefinitionRecord) -> ScalarDefinitionId {
        let id = self.scalar_definitions.len().into();
        self.scalar_definitions.push(scalar_definition);
        id
    }
}
