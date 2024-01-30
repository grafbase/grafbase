use schema::{ResolverId, Schema};

use crate::execution::Variables;
use crate::request::{EntityType, FlatTypeCondition, Operation, QueryPath};
use crate::response::ReadSelectionSet;
use crate::utils::IdRange;

mod collected;
mod ids;
mod planning;
mod state;
mod walkers;
pub(crate) use collected::*;
pub(crate) use ids::*;
pub(crate) use planning::*;
pub(crate) use state::*;
pub(crate) use walkers::*;

pub(crate) struct OperationPlan {
    // -- Operation --
    bound_operation: Operation,
    /// BoundFieldId -> PlanId
    field_attribution: Vec<PlanId>,
    /// BoundSelectionSetId -> PlanId
    selection_set_attribution: Vec<PlanId>,

    // -- Plans --
    /// PlanId -> LogicalPlan
    plans: Vec<LogicalPlan>,
    /// PlanBoundaryId -> u8
    plan_boundary_consummers_count: Vec<u8>,
    /// PlanId/ExecutionPlanId -> PlanInput
    plan_inputs: Vec<Option<PlanInput>>,
    /// PlanId/ExecutionPlanId -> PlanOutput
    plan_outputs: Vec<PlanOutput>,

    // -- Execution plans --
    /// ExecutionPlanId -> ExecutionPlan
    execution_plans: Vec<crate::sources::ExecutionPlan>,
    // sorted by parent plan id
    execution_plans_parent_to_child_edges: Vec<ParentToChildEdge<ExecutionPlanId>>,
    /// ExecutionPlanId -> u8
    execution_plan_dependencies_count: Vec<u8>,

    // -- Collected fields & selection sets --
    collected_conditional_selection_sets: Vec<ConditionalSelectionSet>,
    collected_conditional_fields: Vec<ConditionalField>,
    collected_concrete_selection_sets: Vec<ConcreteSelectionSet>,
    collected_concrete_fields: Vec<ConcreteField>,
}

impl std::ops::Deref for OperationPlan {
    type Target = Operation;

    fn deref(&self) -> &Self::Target {
        &self.bound_operation
    }
}

impl<I> std::ops::Index<I> for OperationPlan
where
    Operation: std::ops::Index<I>,
{
    type Output = <Operation as std::ops::Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.bound_operation[index]
    }
}

impl OperationPlan {
    pub fn prepare(schema: &Schema, operation: Operation) -> PlanningResult<Self> {
        planning::prepare(schema, operation)
    }

    pub fn new_execution_state(&self) -> OperationExecutionState {
        OperationExecutionState::new(self)
    }

    pub fn plan_walker<'s>(
        &'s self,
        schema: &'s Schema,
        plan_id: ExecutionPlanId,
        variables: Option<&'s Variables>,
    ) -> PlanWalker<'s> {
        let plan_id = PlanId::from(usize::from(plan_id));
        let schema_walker = schema.walk(self[plan_id].resolver_id).with_own_names().walk(());
        PlanWalker {
            schema_walker,
            operation: self,
            variables,
            plan_id,
            item: (),
        }
    }
}

#[derive(Debug)]
pub struct LogicalPlan {
    pub resolver_id: ResolverId,
    pub path: QueryPath,
}

#[derive(Debug)]
pub struct PlanInput {
    pub boundary_id: PlanBoundaryId,
    /// if the plan `@requires` any data it will be included in the ReadSelectionSet.
    pub selection_set: ReadSelectionSet,
}

#[derive(Debug)]
pub struct PlanOutput {
    pub type_condition: Option<FlatTypeCondition>,
    pub entity_type: EntityType,
    pub collected_selection_set_id: ConcreteSelectionSetId,
    pub boundary_ids: IdRange<PlanBoundaryId>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct ParentToChildEdge<Id> {
    pub parent: Id,
    pub child: Id,
}
