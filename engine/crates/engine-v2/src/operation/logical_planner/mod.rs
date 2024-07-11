mod logic;
mod selection_set;

use engine_parser::types::OperationType;
use itertools::Itertools;
use schema::{ResolverId, Schema};
use tracing::instrument;

use crate::{
    operation::{
        FieldId, LogicalPlan, LogicalPlanId, Operation, OperationWalker, QueryPath, SelectionSetId,
        SolvedRequiredFieldSet, Variables,
    },
    response::{ErrorCode, GraphqlError},
};
use logic::*;
use selection_set::*;

#[derive(Debug, thiserror::Error)]
pub(crate) enum LogicalPlanningError {
    #[error("Could not plan fields: {}", .missing.join(", "))]
    CouldNotPlanAnyField {
        missing: Vec<String>,
        query_path: Vec<String>,
    },
}

impl From<LogicalPlanningError> for GraphqlError {
    fn from(error: LogicalPlanningError) -> Self {
        let message = error.to_string();
        let query_path = match error {
            LogicalPlanningError::CouldNotPlanAnyField { query_path, .. } => query_path
                .into_iter()
                .map(serde_json::Value::String)
                .collect::<Vec<_>>(),
        };

        GraphqlError::new(message, ErrorCode::OperationPlanningError).with_extension("queryPath", query_path)
    }
}

pub(super) type LogicalPlanningResult<T> = Result<T, LogicalPlanningError>;

pub(super) struct LogicalPlanner<'a> {
    schema: &'a Schema,
    variables: &'a Variables,
    operation: &'a mut Operation,
    field_to_logical_plan_id: Vec<Option<LogicalPlanId>>,
    logical_plans: Vec<LogicalPlan>,
    solved_requirements: Vec<(SelectionSetId, SolvedRequiredFieldSet)>,
}

id_newtypes::index! {
    LogicalPlanner<'a>.logical_plans[LogicalPlanId] => LogicalPlan,
    LogicalPlanner<'a>.field_to_logical_plan_id[FieldId] => Option<LogicalPlanId>,
}

impl<'a> LogicalPlanner<'a> {
    pub(super) fn new(schema: &'a Schema, variables: &'a Variables, operation: &'a mut Operation) -> Self {
        Self {
            schema,
            variables,
            field_to_logical_plan_id: vec![None; operation.fields.len()],
            operation,
            logical_plans: Vec::new(),
            solved_requirements: Vec::new(),
        }
    }

    #[instrument(skip_all)]
    pub(super) fn plan(mut self) -> LogicalPlanningResult<()> {
        tracing::trace!("Logical Planning");
        self.plan_all_fields()?;
        let Self {
            operation,
            logical_plans,
            field_to_logical_plan_id,
            solved_requirements,
            ..
        } = self;

        operation.logical_plans = logical_plans;
        operation.solved_requirements = solved_requirements;
        operation.field_to_logical_plan_id = field_to_logical_plan_id
            .into_iter()
            .enumerate()
            .map(|(i, maybe_logical_plan_id)| match maybe_logical_plan_id {
                Some(logical_plan_id) => logical_plan_id,
                None => {
                    let name = &operation.response_keys[operation.fields[i].response_key()];
                    unreachable!("No plan was associated with field:\n{name}");
                }
            })
            .collect();

        operation.solved_requirements.sort_unstable_by_key(|(id, _)| *id);
        Ok(())
    }

    /// Step 1 of the planning, attributed all fields to a plan and satisfying their requirements.
    fn plan_all_fields(&mut self) -> LogicalPlanningResult<()> {
        // The root plan is always introspection which also lets us handle operations like:
        // query { __typename }
        let introspection = self.schema.walker().introspection_metadata();

        let walker = self.walker();
        let (introspection_field_ids, field_ids): (Vec<_>, Vec<_>) = walker
            .selection_set()
            .as_ref()
            .field_ids_ordered_by_parent_entity_id_then_position
            .iter()
            .partition(|field_id| {
                if let Some(definition) = walker.walk(**field_id).definition() {
                    definition.is_resolvable_in(introspection.subgraph_id)
                } else {
                    true
                }
            });

        if !introspection_field_ids.is_empty() {
            self.push_plan(
                QueryPath::default(),
                introspection.resolver_id,
                &introspection_field_ids,
            )?;
        }

        if matches!(self.operation.ty(), OperationType::Mutation) {
            self.plan_mutation(field_ids)?;
        } else {
            // Subscription are considered to be Queries for planning, they just happen to have
            // only one root field.
            self.plan_query(field_ids)?;
        }

        Ok(())
    }

    /// A query is simply treated as a plan boundary with no parent.
    fn plan_query(&mut self, field_ids: Vec<FieldId>) -> LogicalPlanningResult<()> {
        let id = self.operation.root_selection_set_id;
        SelectionSetLogicalPlanner::new(self, &QueryPath::default(), None).solve(id, Vec::new(), field_ids)
    }

    /// Mutation is a special case because root fields need to execute in order. So planning each
    /// field individually and setting up plan dependencies between them to ensures proper
    /// execution order.
    fn plan_mutation(&mut self, field_ids: Vec<FieldId>) -> LogicalPlanningResult<()> {
        let mut groups = field_ids
            .into_iter()
            .into_group_map_by(|id| self.operation[*id].response_key())
            .into_values()
            .collect::<Vec<_>>();
        // Ordering groups by their position in the query, ensuring proper ordering of plans.
        groups.sort_unstable_by_key(|field_ids| field_ids.iter().map(|id| self.operation[*id].query_position()).min());

        // FIXME: generates one plan per field, should be aggregated if consecutive fields can be
        // planned by a single resolver.
        for field_ids in groups {
            let field = &self.operation[field_ids[0]];
            let definition_id = field
                .definition_id()
                .expect("Introspection resolver should have taken metadata fields");

            let resolver = self
                .schema
                .walker()
                .walk(definition_id)
                .resolvers()
                .next()
                .ok_or_else(|| LogicalPlanningError::CouldNotPlanAnyField {
                    missing: vec![self.operation.response_keys[field.response_key()].to_string()],
                    query_path: vec![],
                })?;

            self.push_plan(QueryPath::default(), resolver.id(), &field_ids)?;
        }
        Ok(())
    }

    /// Obviously providable fields have no requirements and can be provided by the current
    /// resolver.
    fn grow_with_obviously_providable_subselections(
        &mut self,
        path: &QueryPath,
        logic: &PlanningLogic<'a>,
        field_ids: &[FieldId],
    ) -> LogicalPlanningResult<()> {
        self.attribute_fields(field_ids, logic.id());
        for id in field_ids {
            if let Some(selection_set_id) = self.operation[*id].selection_set_id() {
                let field = self.walker().walk(*id);
                let path = path.child(field.response_key());
                let logic = logic.child(field.definition().expect("wouldn't have a subselection").id());
                self.plan_selection_set(&path, &logic, selection_set_id)?;
            }
        }

        Ok(())
    }

    /// Recursively traverse the operation to attribute all fields, planning a boundary if not all
    /// are providable by the current plan.
    ///
    /// The traversal order is important. We want the deepest selection sets to be planned first
    /// ensuring that when we plan a boundary (~selection set with missing fields) we have a
    /// complete picture of the providable fields. All of their fields and nested sub-selections
    /// will be already attributed to plan.
    fn plan_selection_set(
        &mut self,
        path: &QueryPath,
        logic: &PlanningLogic<'a>,
        id: SelectionSetId,
    ) -> LogicalPlanningResult<()> {
        let walker = self.walker();
        let (obviously_plannable_field_ids, unplanned_field_ids): (Vec<_>, Vec<_>) = self.operation[id]
            .field_ids_ordered_by_parent_entity_id_then_position
            .iter()
            .copied()
            .partition(|field_id| {
                if let Some(definition) = walker.walk(*field_id).definition() {
                    logic.is_providable(definition.id())
                        && definition.requires(logic.resolver().subgraph_id()).is_empty()
                } else {
                    true
                }
            });

        self.grow_with_obviously_providable_subselections(path, logic, &obviously_plannable_field_ids)?;

        if !unplanned_field_ids.is_empty() {
            SelectionSetLogicalPlanner::new(self, path, Some(logic)).solve(
                id,
                obviously_plannable_field_ids,
                unplanned_field_ids,
            )?;
        }
        Ok(())
    }

    pub fn walker(&self) -> OperationWalker<'_, (), ()> {
        self.operation.walker_with(self.schema.walker(), self.variables)
    }

    pub fn push_plan(
        &mut self,
        query_path: QueryPath,
        resolver_id: ResolverId,
        field_ids: &[FieldId],
    ) -> LogicalPlanningResult<LogicalPlanId> {
        let id = LogicalPlanId::from(self.logical_plans.len());
        tracing::trace!(
            "Creating {id} ({}): {}",
            self.schema.walk(resolver_id).name(),
            field_ids.iter().format_with(", ", |id, f| f(&format_args!(
                "{}",
                self.walker().walk(*id).response_key_str()
            )))
        );
        self.logical_plans.push(LogicalPlan { resolver_id });
        let logic = PlanningLogic::new(id, self.schema.walk(resolver_id));
        self.grow_with_obviously_providable_subselections(&query_path, &logic, field_ids)?;
        Ok(id)
    }

    pub fn attribute_fields(&mut self, fields: &[FieldId], id: LogicalPlanId) {
        for field_id in fields {
            self[*field_id] = Some(id);
        }
    }
}
