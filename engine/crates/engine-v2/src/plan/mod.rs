use std::collections::{HashMap, HashSet};

use schema::ResolverId;

use crate::{
    request::{
        BoundFieldId, BoundSelectionSetId, EntityType, FlatSelectionSet, FlatTypeCondition, QueryPath, SelectionSetType,
    },
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
    pub sibling_dependencies: HashSet<PlanId>,
    pub input: Option<PlanInput>,
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
    pub root_selection_set_id: BoundSelectionSetId,
    pub entity_type: EntityType,
    /// Part of the selection set the plan is responsible for.
    pub root_fields: Vec<BoundFieldId>,
    /// Attribution is necessary to filter the nested selection sets.
    pub attribution: Attribution,
    /// Expectation of the actual output data.
    pub expectations: Expectations,
}

#[derive(Debug, Clone)]
pub struct PlanBoundary {
    pub selection_set_type: SelectionSetType,
    pub query_path: QueryPath,
    /// A child plan isn't entirely planned yet. We only ensure that any `@requires` of children
    /// will be provided by the parent. Its actual output is only planned once we have the
    /// ResponseObjectRoots.
    pub children: Vec<ChildPlan>,
}

#[derive(Debug, Clone)]
pub struct ChildPlan {
    pub id: PlanId,
    pub resolver_id: ResolverId,
    pub input_selection_set: ReadSelectionSet,
    pub root_selection_set: FlatSelectionSet<EntityType>,
    pub sibling_dependencies: HashSet<PlanId>,
    // Only includes extra fields necessary for other child plans within the same
    // plan boundary.
    extra_selection_sets: HashMap<BoundSelectionSetId, planner::ExtraBoundarySelectionSet>,
}
