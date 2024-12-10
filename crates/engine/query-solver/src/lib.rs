use std::borrow::Cow;

#[cfg(test)]
mod tests;

pub(crate) mod dot_graph;
mod error;
mod operation;
mod solution;
pub(crate) mod solve;
pub use error::*;
pub(crate) use operation::*;
pub use petgraph;
use schema::{
    CompositeTypeId, FieldDefinition, FieldDefinitionId, ObjectDefinitionId, Schema, SchemaField, SubgraphId,
};
pub use solution::*;

pub(crate) type Cost = u16;

pub trait Operation {
    type FieldId: From<usize> + Into<usize> + Copy + std::fmt::Debug + Ord;

    fn root_object_id(&self) -> ObjectDefinitionId;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + 'static;
    fn field_query_position(&self, field_id: Self::FieldId) -> usize;
    fn field_definition(&self, field_id: Self::FieldId) -> Option<FieldDefinitionId>;
    fn field_is_equivalent_to(&self, field_id: Self::FieldId, field: SchemaField<'_>) -> bool;
    fn create_potential_extra_field_from_requirement(
        &mut self,
        petitioner_field_id: Self::FieldId,
        field: SchemaField<'_>,
    ) -> Self::FieldId;
    fn create_potential_alternative_with_different_definition(
        &mut self,
        original: Self::FieldId,
        definition: FieldDefinition<'_>,
        deep_clone: bool,
    ) -> Self::FieldId;
    fn finalize_selection_set(
        &mut self,
        parent_type: CompositeTypeId,
        extra: &mut [(SubgraphId, Self::FieldId)],
        existing: &mut [(SubgraphId, Self::FieldId)],
    );

    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;
    fn subselection(&self, field_id: Self::FieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;

    fn field_label(&self, field_id: Self::FieldId, schema: &Schema, short: bool) -> Cow<'_, str>;
}

pub fn solve<Op: Operation>(schema: &Schema, operation: Op) -> Result<Solution<'_, Op>> {
    let operation_graph = OperationGraph::new(schema, operation)?;
    let solution = solve::Solver::initialize(&operation_graph)?.solve()?;
    Ok(Solution::build_partial(operation_graph, solution)?.finalize())
}
