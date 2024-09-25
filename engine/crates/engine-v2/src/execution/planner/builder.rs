use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
};

use itertools::Itertools;
use schema::{RequiredFieldSetRecord, ResolverDefinition};
use walker::Walk;

use crate::{
    execution::{ExecutableOperation, ExecutionPlan, ExecutionPlanId, PreExecutionContext, ResponseModifierExecutor},
    operation::{
        FieldId, LogicalPlanId, OperationWalker, PlanWalker, ResponseModifierRule, SelectionSetId, SelectionSetType,
    },
    response::{ResponseObjectSetId, ResponseViewSelection, ResponseViewSelectionSet},
    sources::Resolver,
    Runtime,
};

use super::{BuildContext, PlanningResult};

pub(super) struct ExecutionBuilder<'ctx, 'op, R: Runtime> {
    pub(super) ctx: &'op PreExecutionContext<'ctx, R>,
    pub(super) operation: &'op ExecutableOperation,
    pub(super) build_context: &'op mut BuildContext,
}

impl<'ctx, 'op, R: Runtime> std::ops::Deref for ExecutionBuilder<'ctx, 'op, R> {
    type Target = BuildContext;
    fn deref(&self) -> &Self::Target {
        self.build_context
    }
}

impl<'ctx, 'op, R: Runtime> std::ops::DerefMut for ExecutionBuilder<'ctx, 'op, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.build_context
    }
}

impl<'ctx, 'op, R: Runtime> ExecutionBuilder<'ctx, 'op, R>
where
    'ctx: 'op,
{
    pub(super) fn insert_execution_plan(mut self, logical_plan_id: LogicalPlanId) -> PlanningResult<()> {
        let logical_plan = &self.operation[logical_plan_id];
        let resolver = self.ctx.schema().walk(self.operation[logical_plan_id].resolver_id);
        let (requires, input_fields) = self.create_plan_view_and_list_dependencies(
            resolver,
            &logical_plan.root_field_ids_ordered_by_parent_entity_id_then_position,
        );
        let resolver = Resolver::prepare(
            resolver,
            self.operation.borrow().ty(),
            PlanWalker {
                schema: self.ctx.schema(),
                operation: self.operation,
                variables: &self.operation.variables,
                query_modifications: &self.operation.query_modifications,
                logical_plan_id,
                item: (),
            },
        )?;

        let plan = ExecutionPlan {
            // Defined once all execution plans are created.
            parent_count: 0,
            children: Vec::new(),
            requires,
            resolver,
            logical_plan_id,
            dependent_response_modifiers: Vec::new(),
        };
        let execution_plan_id = ExecutionPlanId::from(self.execution_plans.len());
        self.execution_plans.push(plan);
        self.logical_plan_to_execution_plan_id[usize::from(logical_plan_id)] = Some(execution_plan_id);
        let range = self.push_io_fields(input_fields);
        self.execution_plans_input_fields.push(range);

        Ok(())
    }

    pub(super) fn insert_all_response_modifier_executors(&mut self) {
        #[derive(Eq, PartialEq, Ord, PartialOrd)]
        struct ImpactedField {
            // Which rule is applied
            rule: ResponseModifierRule,
            // Where the field will be present in the response
            set_id: ResponseObjectSetId,
            // What field is impacted
            field_id: FieldId,
            field_logical_plan_id: LogicalPlanId,
        }
        let walker = self.walker();

        // First we collect all the impacted fields
        let mut impacted_fields = Vec::with_capacity(self.operation.response_modifier_impacted_fields.len());
        for modifier in &self.operation.response_modifiers {
            for id in modifier.impacted_fields {
                let field = walker.walk(self.operation[id]);
                if self.operation.query_modifications.skipped_fields[field.id()] {
                    continue;
                }
                let set_id = self
                    .operation
                    .response_blueprint
                    .response_modifier_impacted_field_to_response_object_set[usize::from(id)];
                impacted_fields.push(ImpactedField {
                    rule: modifier.rule,
                    field_logical_plan_id: self.operation.plan[field.id()],
                    set_id,
                    field_id: field.id(),
                });
            }
        }
        impacted_fields.sort_unstable();
        let schema = self.ctx.schema();

        // Now we aggregate all the impacted fields by their rule and the plan producing them. This
        // ensure we call a given a rule at most once after a plan has finished but as early as
        // possible.
        self.response_modifier_executors =
            Vec::<ResponseModifierExecutor>::with_capacity(self.operation.response_modifiers.len());
        for ((rule, _), chunk) in impacted_fields
            .into_iter()
            .chunk_by(|impacted_key| (impacted_key.rule, impacted_key.field_logical_plan_id))
            .into_iter()
        {
            let mut on = Vec::new();
            let mut input_fields = self.io_fields_buffer_pool.pop();
            let mut output_fields = self.io_fields_buffer_pool.pop();
            // FIXME: split me into different functions
            let required_fields = match rule {
                ResponseModifierRule::AuthorizedParentEdge { directive_id, .. } => {
                    let required_fields = directive_id.walk(schema).fields().unwrap().as_ref();
                    for ImpactedField { set_id, field_id, .. } in chunk {
                        let field = walker.walk(field_id);

                        let set_ty =
                            self.operation.response_blueprint.response_object_sets_to_type[usize::from(set_id)];
                        let entity_id = field.definition().unwrap().parent_entity().id();
                        let type_condition = (set_ty != SelectionSetType::from(entity_id)).then_some(entity_id);
                        on.push((set_id, type_condition, field.response_key()));

                        output_fields.push(field_id);
                        self.collect_dependencies(
                            field.as_ref().parent_selection_set_id(),
                            required_fields,
                            &mut input_fields,
                        );
                    }
                    required_fields
                }
                ResponseModifierRule::AuthorizedEdgeChild { directive_id, .. } => {
                    let required_fields = directive_id.walk(schema).node().unwrap().as_ref();
                    for ImpactedField { set_id, field_id, .. } in chunk {
                        let field = walker.walk(field_id);

                        let set_ty =
                            self.operation.response_blueprint.response_object_sets_to_type[usize::from(set_id)];
                        let entity_id = field.definition().unwrap().ty().definition().as_entity().unwrap().id();
                        let type_condition = (set_ty != SelectionSetType::from(entity_id)).then_some(entity_id);
                        on.push((set_id, type_condition, field.response_key()));

                        output_fields.push(field_id);
                        self.collect_dependencies(
                            field.as_ref().selection_set_id().unwrap(),
                            required_fields,
                            &mut input_fields,
                        );
                    }
                    required_fields
                }
            };
            let requires = self.build_view(required_fields);
            self.response_modifier_executors.push(ResponseModifierExecutor {
                rule,
                on,
                requires,
                // Defined at the end.
                parent_count: 0,
                children: Vec::new(),
            });
            let range = self.push_io_fields(input_fields);
            self.response_modifier_executors_input_fields.push(range);
            let range = self.push_io_fields(output_fields);
            self.response_modifier_executors_output_fields.push(range);
        }
    }

    fn create_plan_view_and_list_dependencies(
        &mut self,
        resolver: ResolverDefinition<'_>,
        field_ids: &Vec<FieldId>,
    ) -> (ResponseViewSelectionSet, Vec<FieldId>) {
        let mut required_fields = Cow::Borrowed(resolver.requires());
        let mut required_fields_by_selection_set_id = HashMap::new();
        for field_id in field_ids {
            let field = self.walker().walk(*field_id);
            if let Some(definition) = field.definition() {
                let field_requirements = definition.requires_for_subgraph(resolver.as_ref().subgraph_id());
                required_fields = RequiredFieldSetRecord::union_cow(required_fields, field_requirements.clone());
                let value = required_fields_by_selection_set_id
                    .entry(field.as_ref().parent_selection_set_id())
                    .or_insert_with(|| Cow::Borrowed(resolver.requires()));
                *value = RequiredFieldSetRecord::union_cow(std::mem::take(value), field_requirements);
            }
        }
        let mut dependencies = self.io_fields_buffer_pool.pop();
        for (selection_set_id, required_fields) in required_fields_by_selection_set_id {
            self.collect_dependencies(selection_set_id, &required_fields, &mut dependencies)
        }
        let view = self.build_view(&required_fields);
        (view, dependencies)
    }

    pub fn build_view(&mut self, required: &RequiredFieldSetRecord) -> ResponseViewSelectionSet {
        let schema = self.ctx.schema();
        let mut buffer = self.response_view_selection_buffer_pool.pop();

        buffer.extend(required.iter().map(|item| {
            let name = schema[schema[item.field_id].definition_id].name_id;
            ResponseViewSelection {
                name,
                id: item.field_id,
                subselection: self.build_view(&item.subselection),
            }
        }));
        self.push_view_selection_set(buffer)
    }

    fn collect_dependencies(
        &mut self,
        id: SelectionSetId,
        required_fields: &RequiredFieldSetRecord,
        dependencies: &mut Vec<FieldId>,
    ) {
        for required_field in required_fields {
            let solved_requirements = &self.operation.solved_requirements_for(id).expect("Should be planned");
            tracing::trace!(
                "requires {} in ({id}) {:#?}",
                self.ctx
                    .schema()
                    .walk(self.ctx.schema()[required_field.field_id].definition_id)
                    .name(),
                self.walker().walk(*solved_requirements)
            );
            let solved = solved_requirements
                .iter()
                .find(|solved| solved.id == required_field.field_id)
                .expect("Solver did its job");
            let field_id = solved.field_id;
            dependencies.push(field_id);

            if !required_field.subselection.is_empty() {
                self.collect_dependencies(
                    self.operation[field_id]
                        .selection_set_id()
                        .expect("Could not have requirements"),
                    &required_field.subselection,
                    dependencies,
                )
            }
        }
    }

    fn walker(&self) -> OperationWalker<'op, ()> {
        self.operation.walker_with(self.ctx.schema())
    }

    fn push_view_selection_set(&mut self, mut buffer: Vec<ResponseViewSelection>) -> ResponseViewSelectionSet {
        let start = self.response_views.selections.len();
        self.response_views.selections.extend(&mut buffer.drain(..));
        self.response_view_selection_buffer_pool.push(buffer);
        ResponseViewSelectionSet::from(start..self.response_views.selections.len())
    }
}
