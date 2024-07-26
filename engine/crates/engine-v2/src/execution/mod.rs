mod context;
mod coordinator;
mod error;
mod header_rule;
pub(crate) mod hooks;
mod ids;
mod planner;
mod response_modifier;
mod state;
mod walkers;

use std::sync::Arc;

use crate::{
    operation::{FieldId, LogicalPlanId, PreparedOperation, ResponseModifierRule, Variables},
    response::{
        ConcreteObjectShapeId, FieldShapeId, GraphqlError, ResponseKey, ResponseObjectSetId, ResponseViewSelectionSet,
        ResponseViews,
    },
    sources::PreparedExecutor,
    Runtime,
};
pub(crate) use context::*;
pub(crate) use coordinator::*;
pub(crate) use error::*;
pub(crate) use hooks::RequestHooks;
use id_newtypes::{BitSet, IdToMany};
pub(crate) use ids::*;
use schema::EntityId;
use tracing::instrument;
pub(crate) use walkers::*;

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    #[instrument(skip_all)]
    pub(crate) async fn finalize_operation(
        &self,
        operation: Arc<PreparedOperation>,
        variables: Variables,
    ) -> PlanningResult<ExecutableOperation> {
        tracing::trace!("Execution Planning");
        planner::plan(self, operation, variables).await
    }
}

/// All the necessary information for the operation to be executed that can be prepared & cached.
pub(crate) struct ExecutableOperation {
    pub(crate) prepared: Arc<PreparedOperation>,
    pub(crate) variables: Variables,
    pub(crate) subgraph_default_headers: http::HeaderMap,
    pub(crate) query_modifications: QueryModifications,
    pub(crate) execution_plans: Vec<ExecutionPlan>,
    pub(crate) response_views: ResponseViews,
    pub(crate) response_modifier_executors: Vec<ResponseModifierExecutor>,
}

impl std::ops::Deref for ExecutableOperation {
    type Target = PreparedOperation;

    fn deref(&self) -> &Self::Target {
        &self.prepared
    }
}

impl<I> std::ops::Index<I> for ExecutableOperation
where
    PreparedOperation: std::ops::Index<I>,
{
    type Output = <PreparedOperation as std::ops::Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.prepared[index]
    }
}

pub(crate) struct ExecutionPlan {
    pub logical_plan_id: LogicalPlanId,
    pub parent_count: usize,
    pub children: Vec<ExecutionPlanId>,
    pub dependent_response_modifiers: Vec<ResponseModifierExecutorId>,
    pub requires: ResponseViewSelectionSet,
    pub prepared_executor: PreparedExecutor,
}

#[derive(Default)]
pub(crate) struct QueryModifications {
    pub skipped_fields: BitSet<FieldId>,
    pub errors: Vec<GraphqlError>,
    pub concrete_shape_has_error: BitSet<ConcreteObjectShapeId>,
    pub field_shape_id_to_error_ids: IdToMany<FieldShapeId, ErrorId>,
    pub root_error_ids: Vec<ErrorId>,
}

// Modifies the response based on a given rule
pub(crate) struct ResponseModifierExecutor {
    pub rule: ResponseModifierRule,
    /// Which object & fields are impacted
    /// sorted by natural order
    pub on: Vec<(ResponseObjectSetId, Option<EntityId>, ResponseKey)>,
    /// What fields the hook requires
    pub requires: ResponseViewSelectionSet,
    /// Dependency count
    pub parent_count: usize,
    /// Dependents
    pub children: Vec<ExecutionPlanId>,
}
