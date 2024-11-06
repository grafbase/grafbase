use itertools::Itertools;
use schema::RequiredFieldSetRecord;
use walker::Walk;

use crate::{
    execution::PreExecutionContext,
    plan::{error::PlanError, OperationPlan, PlanContext, PlanId, PlanResult},
    resolver::Resolver,
    Runtime,
};

use super::{ExecutableId, ExecutionPlan, PlanResolver, PlanResolverId, QueryContext, QueryModifications};

impl ExecutionPlan {
    #[allow(unused)]
    pub(super) fn build(
        ctx: &PreExecutionContext<'_, impl Runtime>,
        operation_plan: &OperationPlan,
        query_modifications: QueryModifications,
    ) -> PlanResult<Self> {
        let mut plan = Builder {
            ctx,
            plan_ctx: PlanContext {
                schema: ctx.schema(),
                operation_plan,
            },
            query_ctx: QueryContext {
                query_modifications: &query_modifications,
                schema: ctx.schema(),
                operation_plan,
            },
            execution_plan: ExecutionPlan {
                // replaced later
                query_modifications: Default::default(),
                plan_resolvers: Vec::with_capacity(operation_plan.plans.len()),
                response_modifiers: Vec::with_capacity(operation_plan.response_modifier_definitions.len()),
            },
            dependencies: Vec::with_capacity(operation_plan.data_field_refs.len()),
            plan_to_executable: vec![None; operation_plan.plans.len()],
        }
        .build()?;

        plan.query_modifications = query_modifications;
        Ok(plan)
    }
}

struct Builder<'op, 'ctx, R: Runtime> {
    #[allow(unused)]
    ctx: &'op PreExecutionContext<'ctx, R>,
    plan_ctx: PlanContext<'op>,
    query_ctx: QueryContext<'op>,
    execution_plan: ExecutionPlan,
    dependencies: Vec<(ExecutableId, PlanId)>,
    plan_to_executable: Vec<Option<ExecutableId>>,
}

impl<'op, 'ctx, R: Runtime> Builder<'op, 'ctx, R> {
    fn build(mut self) -> PlanResult<ExecutionPlan> {
        for plan_id in (0..self.plan_ctx.operation_plan.plans.len()).map(PlanId::from) {
            self.generate_plan_resolver(plan_id);
        }
        for (executable_id, plan_id) in self.dependencies {
            let Some(dependency_id) = self.plan_to_executable[usize::from(plan_id)] else {
                tracing::error!("Executable depends on an unknown plan");
                return Err(PlanError::InternalError);
            };
            match dependency_id {
                ExecutableId::PlanResolver(id) => self.execution_plan[id].children.push(executable_id),
                ExecutableId::ResponseModifier(id) => self.execution_plan[id].children.push(executable_id),
            }
            match executable_id {
                ExecutableId::PlanResolver(id) => self.execution_plan[id].parent_count += 1,
                ExecutableId::ResponseModifier(id) => self.execution_plan[id].parent_count += 1,
            }
        }
        Ok(self.execution_plan)
    }

    fn generate_plan_resolver(&mut self, plan_id: PlanId) {
        let id = ExecutableId::PlanResolver(PlanResolverId::from(self.execution_plan.plan_resolvers.len()));
        self.register_dependencies(id, plan_id);
        let plan_resolver = PlanResolver {
            plan_id,
            requires: self.create_required_field_set(plan_id),
            resolver: self.prepare_resolver(plan_id),
            // Set later
            parent_count: 0,
            children: Vec::new(),
        };
        self.execution_plan.plan_resolvers.push(plan_resolver);
    }

    fn create_required_field_set(&mut self, plan_id: PlanId) -> RequiredFieldSetRecord {
        let resolver_definition = plan_id.walk(self.plan_ctx).resolver_definition();
        let subgraph_id = resolver_definition.subgraph_id();
        let mut requires = resolver_definition
            .required_field_set()
            .map(|field_set| field_set.as_ref().clone())
            .unwrap_or_default();

        for field in plan_id.walk(self.query_ctx).selection_set().data_fields() {
            if let Some(field_requires) = field.definition().requires_for_subgraph(subgraph_id) {
                requires = RequiredFieldSetRecord::union(&requires, field_requires.as_ref());
            }
        }

        requires
    }

    fn register_dependencies(&mut self, id: ExecutableId, plan_id: PlanId) {
        let mut plan_ids = Vec::new();
        for field in plan_id.walk(self.plan_ctx).required_scalar_fields() {
            plan_ids.push(field.plan_id);
        }
        let mut stack = vec![plan_id.walk(self.query_ctx).selection_set()];
        while let Some(selection_set) = stack.pop() {
            for field in selection_set.data_fields() {
                let plan_field = field.id().walk(self.plan_ctx);

                // If empty we're sure to not have any fields
                if !plan_field.selection_set_record.data_field_ids.is_empty() {
                    stack.push(field.selection_set());
                }

                for required_field in plan_field.required_scalar_fields() {
                    // a requirement can come for the same plan, for @authorized typically
                    if plan_id != required_field.plan_id {
                        plan_ids.push(required_field.plan_id);
                    }
                }
            }
        }
        plan_ids.sort_unstable();
        for plan_id in plan_ids.into_iter().dedup() {
            self.dependencies.push((id, plan_id));
        }
    }

    fn prepare_resolver(&mut self, _plan_id: PlanId) -> Resolver {
        todo!()
    }
}
