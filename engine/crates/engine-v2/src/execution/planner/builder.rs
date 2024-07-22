use std::{borrow::Cow, collections::HashMap};

use schema::{RequiredFieldSet, ResolverWalker};

use crate::{
    execution::{ExecutableOperation, ExecutionPlan, ExecutionPlanId, PlanWalker, PreExecutionContext},
    operation::{FieldId, LogicalPlanId, OperationWalker, SelectionSetId},
    response::{ResponseViewSelection, ResponseViewSelectionSet, ResponseViews},
    sources::PreparedExecutor,
    utils::BufferPool,
    Runtime,
};

use super::PlanningResult;

pub(super) struct RequirementsBuildContext<'ctx, 'op, 'parent, R: Runtime> {
    ctx: &'op PreExecutionContext<'ctx, R>,
    operation: &'op ExecutableOperation,
    response_views: &'parent mut ResponseViews,
    response_view_selection_buffer_pool: &'parent mut BufferPool<ResponseViewSelection>,
}

impl<'ctx, 'op, 'parent, R: Runtime> RequirementsBuildContext<'ctx, 'op, 'parent, R>
where
    'ctx: 'op,
    'op: 'parent,
{
    pub(super) fn new(
        ctx: &'op PreExecutionContext<'ctx, R>,
        operation: &'op ExecutableOperation,
        response_views: &'parent mut ResponseViews,
        response_view_selection_buffer_pool: &'parent mut BufferPool<ResponseViewSelection>,
    ) -> Self {
        Self {
            ctx,
            operation,
            response_views,
            response_view_selection_buffer_pool,
        }
    }

    pub(super) fn build_execution_plan(
        mut self,
        execution_plan_id: ExecutionPlanId,
        logical_plan_id: LogicalPlanId,
    ) -> PlanningResult<(ExecutionPlan, Vec<FieldId>)> {
        let logical_plan = &self.operation[logical_plan_id];
        let resolver = self
            .ctx
            .schema()
            .walker()
            .walk(self.operation[logical_plan_id].resolver_id)
            .with_own_names();
        let (requires, dependencies) = self.create_view_and_list_dependencies(
            resolver,
            &logical_plan.root_field_ids_ordered_by_parent_entity_id_then_position,
        );
        let prepared_executor = PreparedExecutor::prepare(
            resolver,
            self.operation.ty(),
            PlanWalker {
                schema_walker: resolver.walk(()),
                operation: self.operation,
                plan_id: execution_plan_id,
                item: (),
            },
        )?;

        let plan = ExecutionPlan {
            // Defined once all execution plans are created.
            parent_count: 0,
            children: Vec::new(),
            requires,
            prepared_executor,
            logical_plan_id,
        };
        Ok((plan, dependencies))
    }

    fn create_view_and_list_dependencies(
        &mut self,
        resolver: ResolverWalker<'op>,
        field_ids: &Vec<FieldId>,
    ) -> (ResponseViewSelectionSet, Vec<FieldId>) {
        let mut required_fields = Cow::Borrowed(resolver.requires());
        let mut required_fields_by_selection_set_id = HashMap::new();
        for field_id in field_ids {
            let field = self.walker().walk(*field_id);
            if let Some(definition) = field.definition() {
                let field_requirements = definition.required_fields(resolver.subgraph_id());
                required_fields = RequiredFieldSet::union_cow(required_fields, field_requirements.clone());
                let value = required_fields_by_selection_set_id
                    .entry(field.as_ref().parent_selection_set_id())
                    .or_insert_with(|| Cow::Borrowed(resolver.requires()));
                *value = RequiredFieldSet::union_cow(std::mem::take(value), field_requirements);
            }
        }
        let mut dependencies = Vec::new();
        for (selection_set_id, required_fields) in required_fields_by_selection_set_id {
            self.collect_dependencies(selection_set_id, &required_fields, &mut dependencies)
        }
        let view = self.build_view(&required_fields);
        (view, dependencies)
    }

    pub fn build_view(&mut self, required: &RequiredFieldSet) -> ResponseViewSelectionSet {
        let schema = self.ctx.schema();
        let mut buffer = self.response_view_selection_buffer_pool.pop();

        buffer.extend(required.iter().map(|item| {
            let name = schema[schema[item.id].definition_id].name;
            ResponseViewSelection {
                name,
                id: item.id,
                subselection: self.build_view(&item.subselection),
            }
        }));
        self.push_view_selection_set(buffer)
    }

    fn collect_dependencies(
        &mut self,
        id: SelectionSetId,
        required_fields: &RequiredFieldSet,
        dependencies: &mut Vec<FieldId>,
    ) {
        for required_field in required_fields {
            let solved_requirements = &self.operation.solved_requirements_for(id).expect("Should be planned");
            tracing::trace!(
                "requires {} in ({id}) {:#?}",
                self.ctx
                    .schema
                    .walk(self.ctx.schema[required_field.id].definition_id)
                    .name(),
                self.walker().walk(*solved_requirements)
            );
            let solved = solved_requirements
                .iter()
                .find(|solved| solved.id == required_field.id)
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

    fn walker(&self) -> OperationWalker<'op, (), ()> {
        // yes looks weird, will be improved
        self.operation
            .walker_with(self.ctx.schema.walker(), &self.operation.variables)
    }

    fn push_view_selection_set(&mut self, mut buffer: Vec<ResponseViewSelection>) -> ResponseViewSelectionSet {
        let start = self.response_views.selections.len();
        self.response_views.selections.extend(&mut buffer.drain(..));
        self.response_view_selection_buffer_pool.push(buffer);
        ResponseViewSelectionSet::from(start..self.response_views.selections.len())
    }
}
