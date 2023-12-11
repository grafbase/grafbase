use schema::ResolverId;

use crate::{
    request::{BoundFieldId, EntityType, FlatSelectionSet, FlatTypeCondition, QueryPath, SelectionSetType},
    response::{ReadSelectionSet, ResponseBoundaryItem},
};

mod attribution;
mod expectation;
mod ids;
mod planner;

pub use attribution::*;
pub use expectation::*;
pub use ids::*;
pub use planner::Planner;

#[derive(Debug)]
pub struct Plan {
    pub id: PlanId,
    pub resolver_id: ResolverId,
    pub input: PlanInput,
    pub output: PlanOutput,
    /// Boundaries between this plan and its children. ResponseObjectRoots will be collected at
    /// those during execution.
    pub boundaries: Vec<PlanBoundary>,
}

#[derive(Debug)]
pub struct PlanInput {
    /// Response objects which the plan must update.
    pub response_boundary: Vec<ResponseBoundaryItem>,
    /// if the plan `@requires` any data it will be included in the ReadSelectionSet.
    pub selection_set: ReadSelectionSet,
}

#[derive(Debug)]
pub struct PlanOutput {
    pub entity_type: EntityType,
    /// Part of the selection set the plan is responsible for.
    pub root_fields: Vec<BoundFieldId>,
    /// Attribution is necessary to filter the nested selection sets.
    pub attribution: Attribution,
    /// Expectation of the actual output data.
    pub expectations: Expectations,
}

#[derive(Debug)]
pub struct PlanBoundary {
    pub selection_set_type: SelectionSetType,
    /// A child plan isn't entirely planned yet. We only ensure that any `@requires` of children
    /// will be provided by the parent. Its actual output is only planned once we have the
    /// ResponseObjectRoots.
    pub children: Vec<ChildPlan>,
}

#[derive(Debug)]
pub struct ChildPlan {
    pub id: PlanId,
    pub path: QueryPath,
    pub resolver_id: ResolverId,
    pub input_selection_set: ReadSelectionSet,
    pub providable: FlatSelectionSet<EntityType>,
}
