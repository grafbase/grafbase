use id_newtypes::IdRange;
use schema::{ResolverId, Schema};

use crate::{
    operation::{Operation, QueryPath, Variables},
    response::ReadSelectionSet,
    sources::Plan,
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
    //
    // BoundFieldId -> PlanId
    field_to_plan_id: Vec<PlanId>,
    // BoundSelectionSetId -> PlanId
    selection_to_plan_id: Vec<PlanId>,
    /// BoundSelectionSetId -> Option<CollectedSelectionSetId>
    selection_set_to_collected: Vec<Option<AnyCollectedSelectionSetId>>,

    // -- Plans --
    // Actual plans for the operation. A plan defines what do for a given selection set at a
    // certain query path.
    //
    // Its information is split into multiple Vecs as it's built over several steps.
    //
    // PlanId -> PlannedResolver
    planned_resolvers: Vec<PlannedResolver>,
    // PlanId -> Plan
    plans: Vec<Plan>,
    // PlanId -> PlanInput
    plan_inputs: Vec<Option<PlanInput>>,
    // PlanId -> PlanOutput
    plan_outputs: Vec<PlanOutput>,
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

    pub fn walker_with<'s>(&'s self, schema: &'s Schema, variables: &'s Variables, plan_id: PlanId) -> PlanWalker<'s> {
        let plan_id = PlanId::from(usize::from(plan_id));
        let schema_walker = schema
            .walk(self.planned_resolvers[usize::from(plan_id)].resolver_id)
            .with_own_names()
            .walk(());
        PlanWalker {
            schema_walker,
            operation_plan: self,
            variables,
            plan_id,
            item: (),
        }
    }
}

#[derive(Debug)]
pub struct PlannedResolver {
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
    pub collected_selection_set_id: CollectedSelectionSetId,
    pub boundary_ids: IdRange<PlanBoundaryId>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct ParentToChildEdge {
    pub parent: PlanId,
    pub child: PlanId,
}
