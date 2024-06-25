mod bind;
mod build;
mod cache_control;
pub mod ids;
mod input_value;
mod location;
mod parse;
mod path;
mod selection_set;
mod validation;
mod variables;
mod walkers;

use std::num::NonZeroU16;

use crate::response::ResponseKeys;
pub use cache_control::OperationCacheControl;
pub(crate) use engine_parser::types::OperationType;
pub(crate) use ids::*;
pub(crate) use input_value::*;
pub(crate) use location::Location;
pub(crate) use path::QueryPath;
use schema::{ObjectId, RequiredFieldId, ResolverId, SchemaWalker};
pub(crate) use selection_set::*;
pub(crate) use variables::*;
pub(crate) use walkers::*;

pub(crate) struct Plan {
    pub resolver_id: ResolverId,
}

pub(crate) struct Operation {
    pub ty: OperationType,
    pub root_object_id: ObjectId,
    #[allow(dead_code)]
    pub name: Option<String>,
    pub response_keys: ResponseKeys,
    pub root_selection_set_id: SelectionSetId,
    pub selection_sets: Vec<SelectionSet>,
    pub fields: Vec<Field>,
    pub field_to_parent: Vec<SelectionSetId>,
    pub fragments: Vec<Fragment>,
    pub fragment_spreads: Vec<FragmentSpread>,
    pub inline_fragments: Vec<InlineFragment>,
    pub variable_definitions: Vec<VariableDefinition>,
    pub cache_control: Option<OperationCacheControl>,
    pub field_arguments: Vec<FieldArgument>,
    pub query_input_values: QueryInputValues,
    // -- Added during planning --
    pub plans: Vec<Plan>,
    /// Sorted
    pub plan_edges: Vec<ParentToChildEdge>,
    pub field_dependencies: Vec<FieldDependency>,
    pub field_to_plan_id: Vec<Option<PlanId>>,
    pub selection_set_to_plan_id: Vec<Option<PlanId>>,
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ParentToChildEdge {
    // Ordering of the fields matter and is relied upon to find the boundary_id between two plans.
    pub parent: PlanId,
    pub child: PlanId,
    pub boundary: PlanBoundaryId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FieldDependency {
    pub plan_boundary_id: PlanBoundaryId,
    pub required_field_id: RequiredFieldId,
    pub field_id: FieldId,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PlanBoundaryId(NonZeroU16);

impl From<usize> for PlanBoundaryId {
    fn from(value: usize) -> Self {
        Self(
            u16::try_from(value)
                .ok()
                .and_then(|value| NonZeroU16::new(value + 1))
                .expect("Too many plan boundaries"),
        )
    }
}

impl Operation {
    pub fn parent_selection_set_id(&self, id: FieldId) -> SelectionSetId {
        self.field_to_parent[usize::from(id)]
    }

    pub fn walker_with<'op, 'schema, SI>(
        &'op self,
        schema_walker: SchemaWalker<'schema, SI>,
        variables: &'op Variables,
    ) -> OperationWalker<'op, (), SI>
    where
        'schema: 'op,
    {
        OperationWalker {
            operation: self,
            variables,
            schema_walker,
            item: (),
        }
    }

    pub fn find_matching_field(
        &self,
        plan_boundary_id: PlanBoundaryId,
        required_field_id: RequiredFieldId,
    ) -> Option<FieldId> {
        self.field_dependencies
            .binary_search_by(|field_dependency| {
                field_dependency
                    .plan_boundary_id
                    .cmp(&plan_boundary_id)
                    .then(field_dependency.required_field_id.cmp(&required_field_id))
            })
            .ok()
            .map(|index| self.field_dependencies[index].field_id)
    }

    pub fn find_boundary_between(&self, parent: PlanId, child: PlanId) -> Option<PlanBoundaryId> {
        self.plan_edges
            .binary_search_by(|edge| edge.parent.cmp(&parent).then(edge.child.cmp(&child)))
            .ok()
            .map(|index| self.plan_edges[index].boundary)
    }
}
