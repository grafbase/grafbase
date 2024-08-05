mod builder;

use std::{mem::take, sync::Arc};

use builder::ExecutionBuilder;
use id_derives::{Id, IndexImpls};
use id_newtypes::IdRange;
use itertools::Itertools;

use crate::{
    execution::{
        ExecutionPlan, ExecutionPlanId, PlanningResult, PreExecutionContext, QueryModifications,
        ResponseModifierExecutorId,
    },
    operation::{FieldId, LogicalPlanId, PreparedOperation, Variables},
    response::{ResponseViewSelection, ResponseViews},
    utils::BufferPool,
    Runtime,
};

use super::{header_rule::create_subgraph_headers_with_rules, ExecutableOperation, ResponseModifierExecutor};

struct ExecutionPlanner<'ctx, 'op, R: Runtime> {
    ctx: &'op PreExecutionContext<'ctx, R>,
    operation: ExecutableOperation,
    build_context: BuildContext,
}

impl<'ctx, 'op, R: Runtime> std::ops::Deref for ExecutionPlanner<'ctx, 'op, R> {
    type Target = BuildContext;
    fn deref(&self) -> &Self::Target {
        &self.build_context
    }
}

impl<'ctx, 'op, R: Runtime> std::ops::DerefMut for ExecutionPlanner<'ctx, 'op, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.build_context
    }
}

// Ideally this BuilderContext would just be inside the ExecutionPlanner. But we do need to modify
// the ExecutableOperation at some moments (here in the ExecutionPlanner) and at others we rely on it be immutable for walkers (in the builder::ExecutionBuilder)
#[derive(Default, IndexImpls)]
struct BuildContext {
    // Either an input or output field of a plan or response modifier
    #[indexed_by(IOFieldId)]
    io_fields: Vec<FieldId>,
    io_fields_buffer_pool: BufferPool<FieldId>,
    response_modifier_executors: Vec<ResponseModifierExecutor>,
    response_modifier_executors_input_fields: Vec<IdRange<IOFieldId>>,
    response_modifier_executors_output_fields: Vec<IdRange<IOFieldId>>,
    execution_plans: Vec<ExecutionPlan>,
    execution_plans_input_fields: Vec<IdRange<IOFieldId>>,
    logical_plan_to_execution_plan_id: Vec<Option<ExecutionPlanId>>,
    response_views: ResponseViews,
    response_view_selection_buffer_pool: BufferPool<ResponseViewSelection>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub struct IOFieldId(std::num::NonZero<u16>);

impl BuildContext {
    fn push_io_fields(&mut self, mut fields: Vec<FieldId>) -> IdRange<IOFieldId> {
        let start = self.io_fields.len();
        self.io_fields.extend(&mut fields.drain(..));
        self.io_fields_buffer_pool.push(fields);
        IdRange::from(start..self.io_fields.len())
    }
}

pub(super) async fn plan<'ctx, R: Runtime>(
    ctx: &PreExecutionContext<'ctx, R>,
    prepared: Arc<PreparedOperation>,
    variables: Variables,
) -> PlanningResult<ExecutableOperation> {
    let operation = ExecutableOperation {
        query_modifications: QueryModifications::build(ctx, &prepared, &variables).await?,
        prepared,
        variables,
        subgraph_default_headers: create_subgraph_headers_with_rules(
            ctx.request_context,
            ctx.schema.walker().default_header_rules(),
            http::HeaderMap::new(),
        ),
        execution_plans: Default::default(),
        response_views: Default::default(),
        response_modifier_executors: Default::default(),
    };

    let operation = ExecutionPlanner {
        ctx,
        build_context: BuildContext {
            logical_plan_to_execution_plan_id: vec![None; operation.plan.logical_plans.len()],
            io_fields: Vec::with_capacity(operation.fields.len()),
            ..Default::default()
        },
        operation,
    }
    .plan()?;

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

impl<'ctx, 'op, R: Runtime> ExecutionPlanner<'ctx, 'op, R>
where
    'ctx: 'op,
{
    fn plan(mut self) -> PlanningResult<ExecutableOperation> {
        // We start by the end so that we avoid retrieving extra fields that are never read.
        for plan_id in self.operation.plan.in_topological_order.clone().into_iter().rev() {
            self.insert_execution_plan_for(plan_id)?;
        }
        // Build all response modifiers that are still relevant
        self.builder().insert_all_response_modifier_executors();

        // finalize operation with all dependency relations
        self.operation.response_modifier_executors = take(&mut self.build_context.response_modifier_executors);
        self.operation.execution_plans = take(&mut self.build_context.execution_plans);
        self.operation.response_views = take(&mut self.build_context.response_views);

        // All parents of a response modifiers
        for (i, input_fields) in take(&mut self.response_modifier_executors_input_fields)
            .into_iter()
            .enumerate()
        {
            let child_id = ResponseModifierExecutorId::from(i);
            for &field_id in &self.build_context[input_fields] {
                let parent_logical_plan_id = self.operation.plan.field_to_logical_plan_id[usize::from(field_id)];
                let parent_id = self.logical_plan_to_execution_plan_id[usize::from(parent_logical_plan_id)]
                    .expect("Depend on unfinished plan");
                if !self.operation[parent_id]
                    .dependent_response_modifiers
                    .contains(&child_id)
                {
                    self.operation[parent_id].dependent_response_modifiers.push(child_id);
                    self.operation[child_id].parent_count += 1;
                }
            }
        }

        // All execution plans that must be executed *after* a response modifier
        for (i, output_fields) in take(&mut self.response_modifier_executors_output_fields)
            .into_iter()
            .enumerate()
        {
            let parent_id = ResponseModifierExecutorId::from(i);
            for &field_id in &self.build_context[output_fields] {
                for (i, input_fields) in self.build_context.execution_plans_input_fields.iter().enumerate() {
                    let child_id = ExecutionPlanId::from(i);
                    if self.build_context[*input_fields].contains(&field_id)
                        && !self.operation[parent_id].children.contains(&child_id)
                    {
                        self.operation[parent_id].children.push(child_id);
                        self.operation[child_id].parent_count += 1;
                    }
                }
            }
        }

        // child/parent relations between execution plans.
        for (i, input_fields) in take(&mut self.execution_plans_input_fields).into_iter().enumerate() {
            let child_id = ExecutionPlanId::from(i);
            for &field_id in &self.build_context[input_fields] {
                let parent_logical_plan_id = self.operation.plan.field_to_logical_plan_id[usize::from(field_id)];
                let parent_id = self.logical_plan_to_execution_plan_id[usize::from(parent_logical_plan_id)]
                    .expect("Depend on unfinished plan");
                if !self.operation[parent_id].children.contains(&child_id) {
                    self.operation[parent_id].children.push(child_id);
                    self.operation[child_id].parent_count += 1;
                }
            }
        }

        // To enforce the proper ordering of mutation fields, we create parent/child relations
        // between them.
        let mutation_order = self
            .operation
            .plan
            .mutation_fields_plan_order
            .iter()
            .filter_map(|id| self.logical_plan_to_execution_plan_id[usize::from(*id)])
            .collect::<Vec<_>>();

        for (prev, next) in mutation_order.into_iter().tuple_windows() {
            if !self.operation[prev].children.contains(&next) {
                self.operation[prev].children.push(next);
                self.operation[next].parent_count += 1;
            }
        }

        Ok(self.operation)
    }

    fn insert_execution_plan_for(&mut self, logical_plan_id: LogicalPlanId) -> PlanningResult<()> {
        tracing::trace!("Generating execution plan for {logical_plan_id}");
        // TODO: Skip plan with only skipped fields.
        self.builder().insert_execution_plan(logical_plan_id)?;

        Ok(())
    }

    fn builder<'a>(&'a mut self) -> ExecutionBuilder<'ctx, 'a, R>
    where
        'op: 'a,
    {
        ExecutionBuilder {
            ctx: self.ctx,
            operation: &self.operation,
            build_context: &mut self.build_context,
        }
    }
}
