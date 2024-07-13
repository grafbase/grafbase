use std::{borrow::Cow, collections::HashMap};

use schema::{RequiredFieldSet, ResolverWalker};

use crate::{
    execution::{ExecutableOperation, ExecutionPlan, ExecutionPlanId, PlanWalker, PreExecutionContext},
    operation::{FieldId, LogicalPlanId, OperationWalker, SelectionSetId},
    response::{ReadField, ReadSelectionSet},
    sources::PreparedExecutor,
    Runtime,
};

use super::PlanningResult;

pub(super) struct ExecutionPlanBuilder<'ctx, 'op, R: Runtime> {
    ctx: &'op PreExecutionContext<'ctx, R>,
    operation: &'op ExecutableOperation,
    execution_plan_id: ExecutionPlanId,
    logical_plan_id: LogicalPlanId,
    resolver: ResolverWalker<'ctx>,
    dependencies: Vec<FieldId>,
}

impl<'ctx, 'op, R: Runtime> ExecutionPlanBuilder<'ctx, 'op, R>
where
    'ctx: 'op,
{
    pub(super) fn new(
        ctx: &'op PreExecutionContext<'ctx, R>,
        operation: &'op ExecutableOperation,
        execution_plan_id: ExecutionPlanId,
        logical_plan_id: LogicalPlanId,
    ) -> Self {
        let resolver = ctx
            .schema()
            .walker()
            .walk(operation[logical_plan_id].resolver_id)
            .with_own_names();
        Self {
            ctx,
            operation,
            execution_plan_id,
            logical_plan_id,
            resolver,
            dependencies: Vec::new(),
        }
    }
    pub(super) fn build(mut self) -> PlanningResult<(ExecutionPlan, Vec<FieldId>)> {
        let logical_plan = &self.operation[self.logical_plan_id];
        let requires =
            self.create_read_selection_set(&logical_plan.root_field_ids_ordered_by_parent_entity_id_then_position);
        let prepared_executor = PreparedExecutor::prepare(
            self.resolver,
            self.operation.ty(),
            PlanWalker {
                schema_walker: self.resolver.walk(()),
                operation: self.operation,
                plan_id: self.execution_plan_id,
                item: (),
            },
        )?;

        let plan = ExecutionPlan {
            // Defined once all execution plans are created.
            parent_count: 0,
            children: Vec::new(),
            requires,
            prepared_executor,
            logical_plan_id: self.logical_plan_id,
        };
        Ok((plan, self.dependencies))
    }

    fn create_read_selection_set(&mut self, field_ids: &Vec<FieldId>) -> ReadSelectionSet {
        let mut field_ids_by_selection_set_id = HashMap::<_, Vec<_>>::new();
        for field_id in field_ids {
            field_ids_by_selection_set_id
                .entry(self.operation[*field_id].parent_selection_set_id())
                .or_default()
                .push(field_id);
        }

        let mut field_ids_by_selection_set_id = field_ids_by_selection_set_id.into_iter();

        let mut read_selection_set = {
            let (selection_set_id, field_ids) = field_ids_by_selection_set_id
                .next()
                .expect("At least one field is planned");
            let mut requires = Cow::Borrowed(self.resolver.requires());
            for field_id in field_ids {
                if let Some(definition) = self.walker().walk(*field_id).definition() {
                    let field_requires = definition.requires(self.resolver.subgraph_id());
                    if !field_requires.is_empty() {
                        requires = Cow::Owned(requires.union(field_requires));
                    }
                }
            }
            self.create_read_selection_set_for_requirements(selection_set_id, &requires)
        };

        for (selection_set_id, field_ids) in field_ids_by_selection_set_id {
            let mut requires = RequiredFieldSet::default();
            for field_id in field_ids {
                if let Some(definition) = self.walker().walk(*field_id).definition() {
                    let field_requires = definition.requires(self.resolver.subgraph_id());
                    if !field_requires.is_empty() {
                        requires = requires.union(field_requires);
                    }
                }
            }
            read_selection_set =
                read_selection_set.union(self.create_read_selection_set_for_requirements(selection_set_id, &requires));
        }

        read_selection_set
    }

    /// Create the input selection set of a Plan given its resolver and requirements.
    /// We iterate over the requirements and find the matching fields inside the boundary fields,
    /// which contains all providable & extra fields. During the iteration we track all the dependency
    /// plans.
    fn create_read_selection_set_for_requirements(
        &mut self,
        id: SelectionSetId,
        requires: &RequiredFieldSet,
    ) -> ReadSelectionSet {
        if requires.is_empty() {
            return ReadSelectionSet::default();
        }
        requires
            .iter()
            .map(|required_field| {
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
                self.dependencies.push(field_id);
                ReadField {
                    edge: self.operation[field_id].response_edge(),
                    name: self
                        .resolver
                        .walk(self.ctx.schema[required_field.id].definition_id)
                        .name()
                        .to_string(),
                    subselection: if !required_field.subselection.is_empty() {
                        self.create_read_selection_set_for_requirements(
                            self.operation[field_id]
                                .selection_set_id()
                                .expect("Could not have requirements"),
                            &required_field.subselection,
                        )
                    } else {
                        ReadSelectionSet::default()
                    },
                }
            })
            .collect()
    }

    fn walker(&self) -> OperationWalker<'op, (), ()> {
        // yes looks weird, will be improved
        self.operation
            .walker_with(self.ctx.schema.walker(), &self.operation.variables)
    }
}
