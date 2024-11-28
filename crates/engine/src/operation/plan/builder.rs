use id_newtypes::IdToMany;
use itertools::Itertools;
use schema::{CompositeTypeId, FieldSetRecord};
use walker::Walk;

use crate::{
    operation::{
        PlanError, QueryPartition, QueryPartitionId, ResponseModifierDefinition, ResponseModifierRule,
        SolvedOperationContext,
    },
    prepare::{CachedOperation, PrepareContext},
    resolver::Resolver,
    Runtime,
};

use super::{
    ExecutableId, OperationPlan, OperationPlanContext, PlanId, PlanQueryPartition, PlanRecord, PlanResult,
    QueryModifications, ResponseModifierId, ResponseModifierRecord, ResponseModifierTargetRecord,
};

impl OperationPlan {
    #[allow(unused)]
    pub(in crate::operation) fn plan(
        ctx: &PrepareContext<'_, impl Runtime>,
        operation: &CachedOperation,
        query_modifications: QueryModifications,
    ) -> PlanResult<Self> {
        let mut plan = Builder {
            ctx,
            operation,
            solve_ctx: SolvedOperationContext {
                schema: ctx.schema(),
                operation: &operation.solved,
            },
            operation_plan: OperationPlan {
                query_modifications,
                plans: Vec::with_capacity(operation.solved.query_partitions.len()),
                response_modifiers: Vec::with_capacity(operation.solved.response_modifier_definitions.len()),
            },
            dependencies: Vec::with_capacity(operation.solved.data_field_refs.len()),
            partition_to_plan: vec![None; operation.solved.query_partitions.len()],
            partition_modifiers: Vec::with_capacity(operation.solved.response_modifier_definitions.len()),
        }
        .build()?;

        Ok(plan)
    }
}

struct Builder<'op, 'ctx, R: Runtime> {
    #[allow(unused)]
    ctx: &'op PrepareContext<'ctx, R>,
    operation: &'op CachedOperation,
    solve_ctx: SolvedOperationContext<'op>,
    operation_plan: OperationPlan,
    dependencies: Vec<(ExecutableId, QueryPartitionId)>,
    partition_modifiers: Vec<(QueryPartitionId, ResponseModifierId)>,
    partition_to_plan: Vec<Option<PlanId>>,
}

impl<R: Runtime> Builder<'_, '_, R> {
    fn build(mut self) -> PlanResult<OperationPlan> {
        for query_partition in self.solve_ctx.query_partitions() {
            self.generate_plan(query_partition)?;
        }

        for definition in self.solve_ctx.response_modifier_definitions() {
            self.generate_response_modifier(definition)?;
        }

        self.finalize()
    }

    fn finalize(mut self) -> PlanResult<OperationPlan> {
        self.partition_modifiers.sort_unstable();
        for (query_partition_id, modifier_id) in self.partition_modifiers.iter().copied() {
            let Some(dependency_id) = self.partition_to_plan[usize::from(query_partition_id)] else {
                tracing::error!("Executable depends on an unknown plan");
                return Err(PlanError::InternalError);
            };
            self.operation_plan[dependency_id].children_ids.push(modifier_id.into());
            self.operation_plan[modifier_id].parent_count += 1;
        }

        let partition_id_to_modifier_ids = IdToMany::from_sorted_vec(std::mem::take(&mut self.partition_modifiers));

        for (id, query_partition_id) in std::mem::take(&mut self.dependencies) {
            let Some(dependency_id) = self.partition_to_plan[usize::from(query_partition_id)] else {
                tracing::error!("Executable depends on an unknown plan");
                return Err(PlanError::InternalError);
            };
            self.operation_plan[dependency_id].children_ids.push(id);
            match id {
                ExecutableId::Plan(plan_id) => {
                    self.operation_plan[plan_id].parent_count += 1;
                    for modifier_id in partition_id_to_modifier_ids.find_all(query_partition_id).copied() {
                        self.operation_plan[modifier_id].children_ids.push(plan_id.into());
                        self.operation_plan[plan_id].parent_count += 1;
                    }
                }
                ExecutableId::ResponseModifier(modifier_id) => {
                    self.operation_plan[modifier_id].parent_count += 1;
                }
            }
        }

        for (prev, next) in self
            .operation
            .solved
            .mutation_partition_order
            .iter()
            .copied()
            .tuple_windows()
        {
            let Some(prev_id) = self.partition_to_plan[usize::from(prev)] else {
                tracing::error!("Executable depends on an unknown plan");
                return Err(PlanError::InternalError);
            };
            let Some(next_id) = self.partition_to_plan[usize::from(next)] else {
                tracing::error!("Executable depends on an unknown plan");
                return Err(PlanError::InternalError);
            };
            self.operation_plan[prev_id].children_ids.push(next_id.into());
            self.operation_plan[next_id].parent_count += 1;
        }

        Ok(self.operation_plan)
    }

    fn generate_response_modifier(&mut self, definition: ResponseModifierDefinition<'_>) -> PlanResult<()> {
        let mut impacted_fields = Vec::new();
        for field in definition.impacted_fields() {
            if self.operation_plan.query_modifications.skipped_data_fields[field.id] {
                continue;
            }
            let (set_id, composite_type_id) = match definition.rule {
                ResponseModifierRule::AuthorizedParentEdge { .. } => (
                    field.parent_field_output_id.ok_or_else(|| {
                        tracing::error!("Missing response object set id.");
                        PlanError::InternalError
                    })?,
                    field.definition().parent_entity_id.into(),
                ),
                ResponseModifierRule::AuthorizedEdgeChild { .. } => (
                    field.output_id.ok_or_else(|| {
                        tracing::error!("Missing response object set id.");
                        PlanError::InternalError
                    })?,
                    CompositeTypeId::maybe_from(field.definition().ty().definition_id).unwrap(),
                ),
            };
            impacted_fields.push((
                field.query_partition_id,
                set_id,
                composite_type_id,
                field.key.response_key,
                field.id,
            ));
        }

        impacted_fields.sort_unstable();

        for (partition_id, targets) in impacted_fields
            .into_iter()
            .dedup()
            .chunk_by(|(partition_id, _, _, _, _)| *partition_id)
            .into_iter()
        {
            let modifier_id = ResponseModifierId::from(self.operation_plan.response_modifiers.len());

            self.operation_plan.response_modifiers.push(ResponseModifierRecord {
                definition_id: definition.id,
                sorted_target_records: targets
                    .into_iter()
                    .map(|(_, set_id, ty_id, key, id)| {
                        for required_field in id.walk(self.solve_ctx).required_fields_by_supergraph() {
                            self.dependencies
                                .push((modifier_id.into(), required_field.query_partition_id));
                        }
                        ResponseModifierTargetRecord { set_id, ty_id, key }
                    })
                    .collect(),
                // Set later
                parent_count: 0,
                children_ids: Vec::new(),
            });
            self.partition_modifiers.push((partition_id, modifier_id));
        }

        Ok(())
    }

    fn generate_plan(&mut self, query_partition: QueryPartition<'_>) -> PlanResult<()> {
        let plan_id = PlanId::from(self.operation_plan.plans.len());
        self.partition_to_plan[usize::from(query_partition.id)] = Some(plan_id);

        self.register_dependencies(plan_id, query_partition);
        let plan_resolver = PlanRecord {
            query_partition_id: query_partition.id,
            requires: self.create_required_field_set_for_query_partition(query_partition),
            resolver: self.prepare_resolver(query_partition)?,
            // Set later
            parent_count: 0,
            children_ids: Vec::new(),
        };
        self.operation_plan.plans.push(plan_resolver);
        Ok(())
    }

    fn create_required_field_set_for_query_partition(&mut self, query_partition: QueryPartition<'_>) -> FieldSetRecord {
        let resolver_definition = query_partition.resolver_definition();
        let subgraph_id = resolver_definition.subgraph_id();
        let mut requires = resolver_definition
            .required_field_set()
            .map(|field_set| field_set.as_ref().clone())
            .unwrap_or_default();

        for field in self
            .view_plan_query_partition(query_partition.id)
            .selection_set()
            .fields()
        {
            if let Some(field_requires) = field.definition().requires_for_subgraph(subgraph_id) {
                requires = FieldSetRecord::union(&requires, field_requires.as_ref());
            }
        }

        requires
    }

    fn register_dependencies(&mut self, plan_id: PlanId, query_partition: QueryPartition<'_>) {
        let mut partition_ids = Vec::new();
        for field in query_partition.required_fields() {
            partition_ids.push(field.query_partition_id);
        }
        for field in self
            .view_plan_query_partition(query_partition.id)
            .selection_set()
            .fields()
        {
            let plan_field = field.id().walk(self.solve_ctx);

            for required_field in plan_field.required_fields() {
                debug_assert!(required_field.query_partition_id != query_partition.id);
                partition_ids.push(required_field.query_partition_id);
            }
        }
        partition_ids.sort_unstable();
        for dependency_id in partition_ids.into_iter().dedup() {
            self.dependencies.push((plan_id.into(), dependency_id));
        }
    }

    fn prepare_resolver(&mut self, query_partition: QueryPartition<'_>) -> PlanResult<Resolver> {
        Resolver::prepare(self.operation.ty(), self.view_plan_query_partition(query_partition.id))
    }

    pub(crate) fn view_plan_query_partition(&self, id: QueryPartitionId) -> PlanQueryPartition<'_> {
        OperationPlanContext {
            schema: self.ctx.schema(),
            solved_operation: self.solve_ctx.operation,
            operation_plan: &self.operation_plan,
        }
        .view(id)
    }
}
