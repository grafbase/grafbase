use id_newtypes::IdRange;
use schema::{EntityId, ResolverId, Schema};

use crate::{
    operation::{Operation, PlanId, Variables},
    response::ReadSelectionSet,
    sources::PreparedExecutor,
};

mod collected;
mod flat;
mod ids;
mod planning;
mod state;
mod walkers;
pub(crate) use collected::*;
pub(crate) use flat::*;
pub(crate) use ids::*;
pub(crate) use planning::*;
pub(crate) use state::*;
pub(crate) use walkers::*;

/// All the necessary information for the operation to be executed that can be prepared & cached.
pub(crate) struct OperationPlan {
    operation: Operation,

    // Association between fields & selection sets and plans. Used when traversing the operation
    // for a plan filtering out other plans fields and to build the collected selection set.
    /// BoundSelectionSetId -> Option<CollectedSelectionSetId>
    selection_set_to_collected: Vec<Option<AnyCollectedSelectionSetId>>,

    // -- Plans --
    // Actual plans for the operation. A plan defines what do for a given selection set at a
    // certain query path.
    //
    // Its information is split into multiple Vecs as it's built over several steps.
    // PlanId -> Plan
    execution_plans: Vec<ExecutionPlan>,
    // sorted by parent plan id
    plan_parent_to_child_edges: Vec<ParentToChildEdge>,
    // PlanId -> u8
    plan_dependencies_count: Vec<u8>,
    // PlanBoundaryId -> u8
    plan_boundary_consummers_count: Vec<u8>,

    // -- Collected fields & selection sets --
    // Once all fields have been planned, we collect fields to know what to expect from the
    // response. It can be used in two different ways:
    // - to deserialize a JSON and ingest it directly into the response
    // - by introspection plan, and maybe others later, to know what to add to the response.
    //   As fields are already collected, it doesn't need to deal with GraphQL logic anymore.
    //
    // ConditionalSelectionSetId -> ConditionalSelectionSet
    conditional_selection_sets: Vec<ConditionalSelectionSet>,
    // ConditionalFieldId -> ConditionalField
    conditional_fields: Vec<ConditionalField>,
    // CollectedSelectionSetId -> CollectedSelectionSet
    collected_selection_sets: Vec<CollectedSelectionSet>,
    // CollectedFieldId -> CollectedField
    collected_fields: Vec<CollectedField>,
}

pub(crate) struct ExecutionPlan {
    pub plan_id: PlanId,
    pub resolver_id: ResolverId,
    pub input: Option<PlanInput>,
    pub output: PlanOutput,
    pub prepared_executor: PreparedExecutor,
}

impl std::ops::Deref for OperationPlan {
    type Target = Operation;

    fn deref(&self) -> &Self::Target {
        &self.operation
    }
}

impl std::ops::DerefMut for OperationPlan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.operation
    }
}

impl<I> std::ops::Index<I> for OperationPlan
where
    Operation: std::ops::Index<I>,
{
    type Output = <Operation as std::ops::Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.operation[index]
    }
}

impl OperationPlan {
    pub fn prepare(schema: &Schema, variables: &Variables, operation: Operation) -> PlanningResult<Self> {
        planning::plan_operation(schema, variables, operation)
    }

    pub fn new_execution_state(&self) -> OperationExecutionState {
        OperationExecutionState::new(self)
    }

    pub fn walker_with<'s>(
        &'s self,
        schema: &'s Schema,
        variables: &'s Variables,
        execution_plan_id: ExecutionPlanId,
    ) -> PlanWalker<'s, (), ()> {
        let schema_walker = schema
            .walk(self[execution_plan_id].resolver_id)
            .with_own_names()
            .walk(());
        PlanWalker {
            schema_walker,
            operation_plan: self,
            variables,
            execution_plan_id,
            item: (),
        }
    }
}

#[derive(Debug)]
pub struct PlanInput {
    pub boundary_id: ExecutionPlanBoundaryId,
    /// if the plan `@requires` any data it will be included in the ReadSelectionSet.
    pub selection_set: ReadSelectionSet,
}

#[derive(Debug)]
pub struct PlanOutput {
    pub type_condition: Option<FlatTypeCondition>,
    pub entity_type: EntityId,
    pub collected_selection_set_id: CollectedSelectionSetId,
    pub boundary_ids: IdRange<ExecutionPlanBoundaryId>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct ParentToChildEdge {
    pub parent: ExecutionPlanId,
    pub child: ExecutionPlanId,
}
