use std::ops::Index;

use super::*;

#[derive(Debug, Clone, Copy, Ord, PartialEq, Eq, PartialOrd)]
pub(crate) struct FieldIrId(usize);

impl Index<FieldIrId> for CompositionIr {
    type Output = FieldIr;

    fn index(&self, index: FieldIrId) -> &Self::Output {
        &self.fields[index.0]
    }
}

impl From<usize> for FieldIrId {
    fn from(value: usize) -> Self {
        FieldIrId(value)
    }
}

impl From<FieldIrId> for usize {
    fn from(value: FieldIrId) -> Self {
        value.0
    }
}

#[derive(Clone)]
pub(crate) struct FieldIr {
    pub(crate) parent_definition: federated::Definition,
    pub(crate) field_name: federated::StringId,
    pub(crate) field_type: subgraphs::FieldTypeId,
    pub(crate) arguments: federated::InputValueDefinitions,

    pub(crate) resolvable_in: Vec<federated::SubgraphId>,

    /// Subgraph fields corresponding to this federated graph field that have an `@provides`.
    pub(crate) provides: Vec<subgraphs::FieldId>,

    /// Subgraph fields corresponding to this federated graph field that have an `@requires`.
    pub(crate) requires: Vec<subgraphs::FieldId>,

    /// Subgraph fields corresponding to this federated graph field that have an `@authorized`.
    pub(crate) authorized_directives: Vec<subgraphs::FieldId>,

    // @join__field(graph: ..., override: ...)
    pub(crate) overrides: Vec<federated::Override>,

    pub(crate) composed_directives: federated::Directives,

    pub(crate) description: Option<federated::StringId>,
}
