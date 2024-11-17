use super::*;

#[derive(Clone)]
pub(crate) struct FieldIr {
    pub(crate) parent_definition: federated::StringId,
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
