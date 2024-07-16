mod builder;
mod modifier;

use std::sync::Arc;

use itertools::Itertools;

use builder::ExecutionPlanBuilder;

use crate::{
    execution::{ExecutionPlan, ExecutionPlanId, PlanningResult, PreExecutionContext},
    operation::{FieldId, LogicalPlanId, PreparedOperation, Variables},
    sources::PreparedExecutor,
    Runtime,
};

use super::ExecutableOperation;

pub(super) struct ExecutionPlanner<'ctx, 'op, R: Runtime> {
    ctx: &'op PreExecutionContext<'ctx, R>,
    operation: ExecutableOperation,
    logical_plan_to_execution_plan_id: Vec<Option<ExecutionPlanId>>,
    execution_plans_dependencies: Vec<Vec<FieldId>>,
}

impl<'ctx, 'op, R: Runtime> ExecutionPlanner<'ctx, 'op, R>
where
    'ctx: 'op,
{
    pub(super) async fn plan(
        ctx: &'op PreExecutionContext<'ctx, R>,
        operation: Arc<PreparedOperation>,
        variables: Variables,
    ) -> PlanningResult<ExecutableOperation> {
        let operation = ExecutableOperation {
            query_modifications: modifier::QueryModificationsBuilder::new(ctx, &operation, &variables)
                .build()
                .await?,
            prepared: operation,
            variables,
            execution_plans: Default::default(),
        };
        Self {
            ctx,
            logical_plan_to_execution_plan_id: vec![None; operation.plan.logical_plans.len()],
            operation,
            execution_plans_dependencies: Vec::new(),
        }
        .build()
    }

    fn build(mut self) -> PlanningResult<ExecutableOperation> {
        // We start by the end so that we avoid retrieving extra fields that are never read.
        for plan_id in self.operation.plan.in_topological_order.clone().into_iter().rev() {
            self.create_execution_plan(plan_id)?;
        }

        let Self {
            mut operation,
            logical_plan_to_execution_plan_id,
            execution_plans_dependencies,
            ..
        } = self;

        for (i, dependencies) in execution_plans_dependencies.into_iter().enumerate() {
            let child_id = ExecutionPlanId::from(i);
            for field_id in dependencies {
                let logical_plan_id = operation.plan.field_to_logical_plan_id[usize::from(field_id)];
                let parent_id =
                    logical_plan_to_execution_plan_id[usize::from(logical_plan_id)].expect("Depend on unfinished plan");
                if !operation[parent_id].children.contains(&child_id) {
                    operation[parent_id].children.push(child_id);
                    operation[child_id].parent_count += 1;
                }
            }
        }

        // To enforce the proper ordering of mutation fields, we create parent/child relations
        // between them.
        let mut mutation_fields_plan_order = operation
            .plan
            .mutation_fields_plan_order
            .clone()
            .into_iter()
            .filter_map(|id| logical_plan_to_execution_plan_id[usize::from(id)]);
        if let Some(mut prev) = mutation_fields_plan_order.next() {
            for next in mutation_fields_plan_order {
                if !operation[prev].children.contains(&next) {
                    operation[prev].children.push(next);
                    operation[next].parent_count += 1;
                }
                prev = next;
            }
        }

        tracing::trace!(
            "== Plan Summary ==\n{}",
            operation
                .execution_plans
                .iter()
                .enumerate()
                .rev() // roots first
                .format_with("\n", |(id, plan), f| f(&format_args!(
                    "**{id}**\n  input <- {}\n  ouput -> {}",
                    plan.parent_count,
                    plan.children.iter().join(",")
                ))),
        );

        Ok(operation)
    }

    fn create_execution_plan(&mut self, logical_plan_id: LogicalPlanId) -> PlanningResult<()> {
        tracing::trace!("Generating execution plan for {logical_plan_id}");
        // TODO: Skip plan with only skipped fields.
        // FIXME: HACK to build an Executor, holding the prepared GraphQL query, we rely on a
        // PlanWalker which needs an ExecutionPlanId. So we reserve the spot with the
        // LogicalPlanId.
        self.operation.execution_plans.push(ExecutionPlan {
            logical_plan_id,
            parent_count: 0,
            children: Default::default(),
            requires: Default::default(),
            prepared_executor: PreparedExecutor::introspection(),
        });
        let id = ExecutionPlanId::from(self.operation.execution_plans.len() - 1);
        let (execution_plan, dependencies) =
            ExecutionPlanBuilder::new(self.ctx, &self.operation, id, logical_plan_id).build()?;
        self.execution_plans_dependencies.push(dependencies);
        self.operation[id] = execution_plan;
        self.logical_plan_to_execution_plan_id[usize::from(logical_plan_id)] = Some(id);

        Ok(())
    }
}
