mod builder;
mod conditions;
mod partition;
mod pool;
mod shape;

use itertools::Itertools;
use pool::BufferPool;
use schema::Schema;
use std::collections::{HashMap, HashSet};

use builder::ExecutionPlanBuilder;

use crate::{
    execution::{ExecutionPlanId, ExecutionPlans, PlanWalker, PlanningError, PlanningResult, PreExecutionContext},
    operation::{ConditionResult, FieldId, LogicalPlanId, Operation, OperationWalker, SelectionSetType, Variables},
    response::{FieldError, FieldShape, ResponseObjectSetId, Shapes},
    sources::PreparedExecutor,
    Runtime,
};

pub(super) struct ExecutionPlanner<'ctx, 'op, R: Runtime> {
    ctx: &'op PreExecutionContext<'ctx, R>,
    operation: &'op Operation,
    variables: &'op Variables,
    to_be_planned: Vec<ToBePlanned>,
    plan_parent_to_child_edges: HashSet<UnfinalizedParentToChildEdge>,
    plan_id_to_execution_plan_id: Vec<Option<ExecutionPlanId>>,
    condition_results: Vec<ConditionResult>,
    field_shapes_buffer_pool: BufferPool<FieldShape>,
    field_errors_buffer_pool: BufferPool<FieldError>,
    plans: ExecutionPlans,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct UnfinalizedParentToChildEdge {
    parent: LogicalPlanId,
    child: LogicalPlanId,
}

struct ToBePlanned {
    logical_plan_id: LogicalPlanId,
    input_id: ResponseObjectSetId,
    selection_set_ty: SelectionSetType,
    root_fields: Vec<FieldId>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct ParentToChildEdge {
    pub parent: ExecutionPlanId,
    pub child: ExecutionPlanId,
}

impl<'ctx, 'op, R: Runtime> ExecutionPlanner<'ctx, 'op, R>
where
    'ctx: 'op,
{
    pub(super) fn new(
        ctx: &'op PreExecutionContext<'ctx, R>,
        operation: &'op Operation,
        variables: &'op Variables,
    ) -> Self {
        ExecutionPlanner {
            ctx,
            operation,
            variables,
            to_be_planned: Vec::new(),
            plan_parent_to_child_edges: HashSet::new(),
            plan_id_to_execution_plan_id: vec![None; operation.logical_plans.len()],
            condition_results: Vec::new(),
            field_shapes_buffer_pool: Default::default(),
            field_errors_buffer_pool: Default::default(),
            plans: ExecutionPlans {
                shapes: Shapes::default(),
                response_object_set_consummers_count: Vec::new(),
                execution_plans: Vec::new(),
                root_errors: Vec::new(),
                prepared_executors: Vec::new(),
            },
        }
    }

    pub(super) async fn plan(mut self) -> PlanningResult<ExecutionPlans> {
        self.condition_results = self.evaluate_all_conditions().await?;
        self.finalize()
    }

    fn finalize(mut self) -> PlanningResult<ExecutionPlans> {
        if let Some(id) = self.operation.root_condition_id {
            match &self.condition_results[usize::from(id)] {
                ConditionResult::Include => (),
                ConditionResult::Errors(errors) => {
                    self.plans.root_errors.extend_from_slice(errors);
                    return Ok(self.plans);
                }
            }
        }

        self.generate_root_execution_plans()?;
        let mut plans = self.plans;
        let mut plan_parent_to_child_edges = self
            .plan_parent_to_child_edges
            .into_iter()
            .map(|edge| {
                let parent = self.plan_id_to_execution_plan_id[usize::from(edge.parent)];
                let child = self.plan_id_to_execution_plan_id[usize::from(edge.child)];
                match (parent, child) {
                    (Some(parent), Some(child)) => Ok(ParentToChildEdge { parent, child }),
                    pc => Err(PlanningError::InternalError(format!(
                        "Unplanned depedency: {edge:?} -> {pc:?}"
                    ))),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        plan_parent_to_child_edges.sort_unstable();

        for ParentToChildEdge { parent, child } in plan_parent_to_child_edges {
            let parent = &mut plans.execution_plans[usize::from(parent)];
            parent.output.dependent.push(child);
            let child = &mut plans.execution_plans[usize::from(child)];
            child.input.dependencies_count += 1;
        }
        tracing::trace!(
            "== Plan Summary ==\n{}",
            plans
                .execution_plans
                .iter()
                .enumerate()
                .format_with("\n", |(id, plan), f| f(&format_args!(
                    "**{id}**\n  input <- {}\n  ouput -> {}",
                    plan.input.dependencies_count,
                    plan.output.dependent.iter().join(",")
                ))),
        );

        Ok(plans)
    }

    fn generate_root_execution_plans(&mut self) -> PlanningResult<()> {
        let walker = self.walker();
        let root_plans = walker.selection_set().fields().fold(
            HashMap::<LogicalPlanId, Vec<FieldId>>::default(),
            |mut acc, field| {
                let plan_id = self.operation.plan_id_for(field.id());
                acc.entry(plan_id).or_default().push(field.id());
                acc
            },
        );

        if walker.is_mutation() {
            let mut maybe_previous_plan_id: Option<LogicalPlanId> = None;
            let mut plan_ids = root_plans
                .iter()
                .map(|(plan_id, fields)| (walker.walk(fields[0]).as_ref().query_position(), plan_id))
                .collect::<Vec<_>>();
            plan_ids.sort_unstable();
            for (_, &plan_id) in plan_ids {
                if let Some(previous_plan_id) = maybe_previous_plan_id {
                    self.plan_parent_to_child_edges.insert(UnfinalizedParentToChildEdge {
                        parent: previous_plan_id,
                        child: plan_id,
                    });
                }
                maybe_previous_plan_id = Some(plan_id);
            }
        }

        let response_location = self.next_response_object_set_id();
        self.to_be_planned = root_plans
            .into_iter()
            .map(|(plan_id, root_fields)| ToBePlanned {
                input_id: response_location,
                selection_set_ty: SelectionSetType::Object(self.walker().as_ref().root_object_id),
                logical_plan_id: plan_id,
                root_fields,
            })
            .collect();

        while let Some(to_be_planned) = self.to_be_planned.pop() {
            self.generate_plan(to_be_planned)?;
        }

        Ok(())
    }

    fn generate_plan(
        &mut self,
        ToBePlanned {
            input_id,
            selection_set_ty,
            logical_plan_id,
            root_fields,
        }: ToBePlanned,
    ) -> PlanningResult<()> {
        tracing::trace!("Generating execution plan for {logical_plan_id}");
        let execution_plan =
            ExecutionPlanBuilder::new(self, input_id, logical_plan_id).build(selection_set_ty, root_fields)?;

        self.plans.execution_plans.push(execution_plan);
        let id = ExecutionPlanId::from(self.plans.execution_plans.len() - 1);
        self.plan_id_to_execution_plan_id[usize::from(logical_plan_id)] = Some(id);

        let resolver = self
            .ctx
            .schema()
            .walker()
            .walk(self.operation[logical_plan_id].resolver_id)
            .with_own_names();

        let prepared_executor = PreparedExecutor::prepare(
            resolver,
            self.operation.ty(),
            PlanWalker {
                schema_walker: resolver.walk(()),
                operation: self.operation,
                variables: self.variables,
                plans: &self.plans,
                execution_plan_id: id,
                item: (),
            },
        )?;
        self.plans.prepared_executors.push(prepared_executor);

        Ok(())
    }

    fn next_response_object_set_id(&mut self) -> ResponseObjectSetId {
        let id = ResponseObjectSetId::from(self.plans.response_object_set_consummers_count.len());
        self.plans.response_object_set_consummers_count.push(0);
        id
    }

    fn walker(&self) -> OperationWalker<'op, (), ()> {
        // yes looks weird, will be improved
        self.operation.walker_with(self.ctx.schema.walker(), self.variables)
    }

    fn schema(&self) -> &'ctx Schema {
        &self.ctx.engine.schema
    }
}
