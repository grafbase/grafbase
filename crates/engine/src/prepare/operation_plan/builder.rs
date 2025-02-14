use id_newtypes::IdToMany;
use itertools::Itertools;
use schema::CompositeTypeId;
use walker::Walk;

use crate::{
    prepare::{
        CachedOperation, CachedOperationContext, PlanError, PrepareContext, QueryPartition, QueryPartitionId,
        RequiredFieldSet, RequiredFieldSetRecord, ResponseModifierDefinition, ResponseModifierRule,
    },
    resolver::Resolver,
    Runtime,
};

use super::{
    ExecutableId, OperationPlan, OperationPlanContext, PlanId, PlanQueryPartition, PlanRecord, PlanResult,
    QueryModifications, ResponseModifierId, ResponseModifierRecord, ResponseModifierTargetRecord,
};

impl OperationPlan {
    pub(in crate::prepare) fn plan(
        ctx: &PrepareContext<'_, impl Runtime>,
        cached: &CachedOperation,
        query_modifications: QueryModifications,
    ) -> PlanResult<Self> {
        Builder {
            ctx,
            operation: cached,
            cached_ctx: CachedOperationContext {
                schema: ctx.schema(),
                cached,
            },
            operation_plan: OperationPlan {
                query_modifications,
                plans: Vec::with_capacity(cached.query_plan.partitions.len()),
                response_modifiers: Vec::with_capacity(cached.query_plan.response_modifier_definitions.len()),
            },
            dependencies: Vec::new(),
            partition_to_plan: vec![None; cached.query_plan.partitions.len()],
            partition_modifiers: Vec::with_capacity(cached.query_plan.response_modifier_definitions.len()),
        }
        .build()
    }
}

struct Builder<'op, 'ctx, R: Runtime> {
    ctx: &'op PrepareContext<'ctx, R>,
    operation: &'op CachedOperation,
    cached_ctx: CachedOperationContext<'op>,
    operation_plan: OperationPlan,
    dependencies: Vec<(ExecutableId, QueryPartitionId)>,
    partition_modifiers: Vec<(QueryPartitionId, ResponseModifierId)>,
    partition_to_plan: Vec<Option<PlanId>>,
}

impl<'op, R: Runtime> Builder<'op, '_, R> {
    fn build(mut self) -> PlanResult<OperationPlan> {
        for query_partition in self.cached_ctx.query_partitions() {
            self.generate_plan(query_partition)?;
        }

        for definition in self.cached_ctx.response_modifier_definitions() {
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
            .query_plan
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

    fn generate_response_modifier(&mut self, definition: ResponseModifierDefinition<'op>) -> PlanResult<()> {
        let mut impacted_fields = Vec::new();
        for field in definition.impacted_fields() {
            if !self.operation_plan.query_modifications.response_data_fields[field.id] {
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
            impacted_fields.push((field.query_partition_id, set_id, composite_type_id, field.id));
        }

        impacted_fields.sort_unstable();

        for (partition_id, targets) in impacted_fields
            .into_iter()
            .dedup()
            .chunk_by(|(partition_id, _, _, _)| *partition_id)
            .into_iter()
        {
            let modifier_id = ResponseModifierId::from(self.operation_plan.response_modifiers.len());
            let sorted_target_records = targets
                .into_iter()
                .map(|(_, set_id, ty_id, field_id)| {
                    self.register_dependencies(
                        modifier_id.into(),
                        field_id.walk(self.cached_ctx).required_fields_by_supergraph(),
                    );
                    ResponseModifierTargetRecord {
                        set_id,
                        ty_id,
                        field_id,
                    }
                })
                .collect();
            self.operation_plan.response_modifiers.push(ResponseModifierRecord {
                rule: definition.rule,
                sorted_target_records,
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
        let required_fields_record = self.create_required_field_set_for_query_partition(query_partition);

        self.register_dependencies(plan_id.into(), required_fields_record.walk(self.cached_ctx));
        let plan_resolver = PlanRecord {
            query_partition_id: query_partition.id,
            required_fields_record,
            resolver: self.prepare_resolver(query_partition)?,
            // Set later
            parent_count: 0,
            children_ids: Vec::new(),
        };
        self.operation_plan.plans.push(plan_resolver);
        Ok(())
    }

    fn create_required_field_set_for_query_partition(
        &mut self,
        query_partition: QueryPartition<'_>,
    ) -> RequiredFieldSetRecord {
        let mut required_fields = query_partition.required_fields_record.clone();

        for field in self
            .view_plan_query_partition(query_partition.id)
            .selection_set()
            .fields()
        {
            required_fields = required_fields.union(&field.id.walk(self.cached_ctx).required_fields_record);
        }

        required_fields
    }

    fn register_dependencies(&mut self, executable_id: ExecutableId, required_fields: RequiredFieldSet<'_>) {
        let mut partition_ids = Vec::new();
        let mut stack = Vec::new();
        stack.push(required_fields);
        while let Some(required_fields) = stack.pop() {
            for required_field in required_fields.iter() {
                partition_ids.push(required_field.data_field().query_partition_id);
                let subselection = required_field.subselection();
                if !subselection.is_empty() {
                    stack.push(subselection);
                }
            }
        }

        partition_ids.sort_unstable();
        for dependency_id in partition_ids.into_iter().dedup() {
            self.dependencies.push((executable_id, dependency_id));
        }
    }

    fn prepare_resolver(&mut self, query_partition: QueryPartition<'_>) -> PlanResult<Resolver> {
        Resolver::prepare(self.operation.ty(), self.view_plan_query_partition(query_partition.id))
    }

    pub(crate) fn view_plan_query_partition(&self, id: QueryPartitionId) -> PlanQueryPartition<'_> {
        OperationPlanContext {
            schema: self.ctx.schema(),
            cached: self.cached_ctx.cached,
            plan: &self.operation_plan,
        }
        .view(id)
    }
}
