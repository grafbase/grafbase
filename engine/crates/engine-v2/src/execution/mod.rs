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
    /// Finalizes the operation by planning it for execution.
    ///
    /// This function takes a prepared operation and associated variables,
    /// performs the necessary planning, and returns a result containing an
    /// executable operation.
    ///
    /// # Arguments
    ///
    /// * `operation` - A reference-counted pointer to the prepared operation to be finalized.
    /// * `variables` - The variables associated with the operation.
    ///
    /// # Returns
    ///
    /// This function returns a `PlanningResult` containing an `ExecutableOperation`.
    pub(crate) async fn finalize_operation(
        &self,
        operation: Arc<PreparedOperation>,
        variables: Variables,
    ) -> PlanningResult<ExecutableOperation> {
        tracing::trace!("Execution Planning");
        planner::plan(self, operation, variables).await
    }
}

/// Represents all the necessary information required for the operation to be executed,
/// which can be prepared and cached for efficient reuse.
#[derive(id_derives::IndexedFields)]
pub(crate) struct ExecutableOperation {
    /// The prepared operation that is to be executed.
    pub(crate) prepared: Arc<PreparedOperation>,
    /// The variables that will be passed along with the operation.
    pub(crate) variables: Variables,
    /// Default headers for the subgraph, applicable to the request.
    pub(crate) subgraph_default_headers: http::HeaderMap,
    /// Modifications to the query that will be applied before execution.
    pub(crate) query_modifications: QueryModifications,
    /// A vector of execution plans indexed by their unique identifiers.
    #[indexed_by(ExecutionPlanId)]
    pub(crate) execution_plans: Vec<ExecutionPlan>,
    /// The response views that will be used for this operation.
    pub(crate) response_views: ResponseViews,
    /// A vector of response modifier executors indexed by their unique identifiers.
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
    /// The unique identifier for the logical plan associated with this execution plan.
    pub logical_plan_id: LogicalPlanId,
    /// The number of parent execution plans that this execution plan depends on.
    pub parent_count: usize,
    /// A list of child execution plans that are dependent on this execution plan.
    pub children: Vec<ExecutionPlanId>,
    /// A list of response modifier executors that depend on this execution plan.
    pub dependent_response_modifiers: Vec<ResponseModifierExecutorId>,
    /// The required response views for executing this plan.
    pub requires: ResponseViewSelectionSet,
    /// The resolver used to resolve the fields in this execution plan.
    pub resolver: Resolver,
}

/// Modifies the response based on a given rule
pub(crate) struct ResponseModifierExecutor {
    /// The rule that defines how the response should be modified.
    pub rule: ResponseModifierRule,
    /// A vector indicating which objects and fields are impacted by the modification,
    /// sorted in natural order.
    pub on: Vec<(ResponseObjectSetId, Option<EntityDefinitionId>, ResponseKey)>,
    /// The fields required by the hook to process the response.
    pub requires: ResponseViewSelectionSet,
    /// The count of parent execution plans that this executor depends on.
    pub parent_count: usize,
    /// A list of child execution plans that depend on this executor.
    pub children: Vec<ExecutionPlanId>,
}
