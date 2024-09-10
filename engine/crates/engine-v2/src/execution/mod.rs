mod context;
mod coordinator;
mod error;
mod header_rule;
pub(crate) mod hooks;
mod ids;
mod planner;
mod response_modifier;
mod state;

use std::sync::Arc;

use crate::{
    operation::{LogicalPlanId, PreparedOperation, QueryModifications, ResponseModifierRule, Variables},
    response::{ResponseKey, ResponseObjectSetId, ResponseViewSelectionSet, ResponseViews},
    sources::Resolver,
    Runtime,
};
pub(crate) use context::*;
pub(crate) use coordinator::*;
pub(crate) use error::*;
pub(crate) use hooks::RequestHooks;
pub(crate) use ids::*;
use schema::EntityDefinitionId;

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
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
#[derive(id_derives::IndexedFields)]
pub(crate) struct ExecutableOperation {
    pub(crate) prepared: Arc<PreparedOperation>,
    pub(crate) variables: Variables,
    pub(crate) subgraph_default_headers: http::HeaderMap,
    pub(crate) query_modifications: QueryModifications,
    #[indexed_by(ExecutionPlanId)]
    pub(crate) execution_plans: Vec<ExecutionPlan>,
    pub(crate) response_views: ResponseViews,
    #[indexed_by(ResponseModifierExecutorId)]
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
    pub resolver: Resolver,
}

// Modifies the response based on a given rule
pub(crate) struct ResponseModifierExecutor {
    pub rule: ResponseModifierRule,
    /// Which object & fields are impacted
    /// sorted by natural order
    pub on: Vec<(ResponseObjectSetId, Option<EntityDefinitionId>, ResponseKey)>,
    /// What fields the hook requires
    pub requires: ResponseViewSelectionSet,
    /// Dependency count
    pub parent_count: usize,
    /// Dependents
    pub children: Vec<ExecutionPlanId>,
}
