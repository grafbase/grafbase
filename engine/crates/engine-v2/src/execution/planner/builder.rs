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

/// A builder that helps in constructing execution plans for GraphQL operations.
///
/// This struct holds references to the execution context, the current operation,
/// and a mutable build context, which is used to record the state of the building
/// process.
pub(super) struct ExecutionBuilder<'ctx, 'op, R: Runtime> {
    /// The pre-execution context containing relevant runtime information.
    pub(super) ctx: &'op PreExecutionContext<'ctx, R>,
    /// The executable operation that is being processed.
    pub(super) operation: &'op ExecutableOperation,
    /// A mutable context used for building execution plans.
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
    /// Inserts an execution plan for the specified logical plan ID into the builder.
    ///
    /// This function creates an execution plan from the provided logical plan ID
    /// and registers it within the execution builder. It collects required fields
    /// and prepares necessary resolvers, and also manages the input fields for
    /// the execution plan.
    ///
    /// # Parameters
    ///
    /// - `logical_plan_id`: The ID of the logical plan for which the execution
    ///   plan is being inserted.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure of the operation, encapsulated in
    /// `PlanningResult<()>`.
    pub(super) fn insert_execution_plan(mut self, logical_plan_id: LogicalPlanId) -> PlanningResult<()> {
        let logical_plan = &self.operation[logical_plan_id];
        let resolver = self.ctx.schema().walk(self.operation[logical_plan_id].resolver_id);

        let (requires, input_fields) =
            self.create_plan_view_and_list_dependencies(resolver, &logical_plan.root_field_ids);

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

    /// Inserts all response modifier executors into the execution builder.
    ///
    /// This function iterates through all defined response modifiers in the
    /// current operation and collects impacted fields for each modifier.
    /// It organizes the impacted fields by their respective rules and the
    /// logical plan that produces them, ensuring each rule is applied only
    /// once per plan while preserving the execution order.
    ///
    /// Additionally, it prepares the necessary input and output fields for
    /// each response modifier executor, maintaining the dependencies required
    /// for successful execution.
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

    /// Creates a plan view and lists dependencies for the provided resolver and field IDs.
    ///
    /// This function analyzes the specified resolver and its associated fields to determine
    /// the required fields and their dependencies. It builds the required field set needed for
    /// correct execution of the resolver while tracking input fields for the associated logical plans.
    ///
    /// # Parameters
    ///
    /// - `resolver`: The resolver definition that indicates the requirements for the logical plan.
    /// - `field_ids`: A vector of field IDs for which the dependencies are being calculated.
    ///
    /// # Returns
    ///
    /// A tuple containing the constructed `ResponseViewSelectionSet` and a vector of input field IDs
    /// that are required for executing the logical plan.
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

    /// Builds a response view selection set based on the required fields.
    ///
    /// This method constructs a `ResponseViewSelectionSet` for the given set
    /// of required fields. It maps each required field to a `ResponseViewSelection`
    /// containing the field's name, ID, and any subselections.
    ///
    /// # Parameters
    ///
    /// - `required`: The set of required fields defined by `RequiredFieldSetRecord`.
    ///
    /// # Returns
    ///
    /// A `ResponseViewSelectionSet` that includes all relevant selections based on
    /// the provided required fields.
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

    /// Collects dependencies for the required fields in the specified selection set.
    ///
    /// This function traverses the required fields and resolves their dependencies
    /// within the logical plan identified by `id`. It pushes the field IDs of the
    /// required fields into the `dependencies` vector. If any required fields have
    /// subselections, the function will recursively collect their dependencies as well.
    ///
    /// # Parameters
    ///
    /// - `id`: The ID of the selection set from which to collect dependencies.
    /// - `required_fields`: A reference to the required fields that dictate the dependencies.
    /// - `dependencies`: A mutable vector that will be populated with the resolved field IDs.
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

    /// Creates an `OperationWalker` for the current operation with the associated schema.
    ///
    /// # Returns
    ///
    /// An `OperationWalker` instance that allows for walking through the operation's
    /// elements.
    fn walker(&self) -> OperationWalker<'op, ()> {
        self.operation.walker_with(self.ctx.schema())
    }

    /// Pushes the given view selection buffer into the response view's selection set.
    ///
    /// This method appends the provided vector of `ResponseViewSelection` to the
    /// existing selections in the response view and returns a new `ResponseViewSelectionSet`
    /// representing the range of the newly added selections.
    ///
    /// # Parameters
    ///
    /// - `buffer`: A vector containing the `ResponseViewSelection` items to be added.
    ///
    /// # Returns
    ///
    /// A `ResponseViewSelectionSet` representing the indices of the newly added selections.
    fn push_view_selection_set(&mut self, mut buffer: Vec<ResponseViewSelection>) -> ResponseViewSelectionSet {
        let start = self.response_views.selections.len();

        self.response_views.selections.extend(&mut buffer.drain(..));
        self.response_view_selection_buffer_pool.push(buffer);

        ResponseViewSelectionSet::from(start..self.response_views.selections.len())
    }
}
