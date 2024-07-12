use id_newtypes::IdRange;
use schema::EntityId;

use crate::{
    operation::{FieldId, LogicalPlanId, SelectionSetId},
    response::{ConcreteObjectShapeId, ReadSelectionSet, ResponseObjectSetId, Shapes},
    sources::PreparedExecutor,
};

use super::ExecutionPlanId;

pub(crate) struct ExecutionPlans {
    pub(crate) shapes: Shapes,
    pub(super) response_object_set_consummers_count: Vec<usize>,
    pub(super) execution_plans: Vec<ExecutionPlan>,
    // ExecutionPlanId -> PreparedExecutor
    pub(super) prepared_executors: Vec<PreparedExecutor>,
}

impl ExecutionPlans {
    pub(crate) fn prepared_executor(&self, id: ExecutionPlanId) -> &PreparedExecutor {
        &self.prepared_executors[usize::from(id)]
    }
}

pub(crate) struct ExecutionPlan {
    pub logical_plan_id: LogicalPlanId,
    pub input: PlanInput,
    pub output: PlanOutput,
}

pub struct PlanInput {
    pub id: ResponseObjectSetId,
    pub entity_id: EntityId,
    pub dependencies_count: usize,
    pub requires: ReadSelectionSet,
}

pub struct PlanOutput {
    /// Like SelectionSet.field_ids
    /// Ordered by query (parent EntityId, query position)
    pub root_field_ids_ordered_by_parent_entity_id_then_position: Vec<FieldId>,
    pub shape_id: ConcreteObjectShapeId,
    pub tracked_output_ids: IdRange<ResponseObjectSetId>,
    pub dependent: Vec<ExecutionPlanId>,
    // sorted
    pub requires_typename_for: Vec<SelectionSetId>,
}
