use engine_parser::types::OperationType;
use im::HashSet;
use itertools::Itertools;
use schema::{ResolverId, Schema};

use super::{boundary::SelectionSetSolver, logic::PlanningLogic, PlanningError, PlanningResult};
use crate::operation::{
    FieldId, Operation, OperationWalker, ParentToChildEdge, Plan, PlanId, QueryPath, SelectionSetId,
    SolvedRequiredFieldSet, Variables,
};

/// The planner is responsible to attribute a plan id for every field & selection set in the
/// operation and ensuring that all requirements from resolvers and fields are satisfied.
///
/// The planning works in three steps:
///
/// 1. Attribute the fields and adding any extra ones:
///     - We have unplanned (missing) fields
///     - Flatten the selection set, removing fragments & inline fragments, for easier
///       manipulation.
///     - Detect which part is providable by the current plan if any and which aren't.
///     - Attribute relevant fields & selection sets to the current plan.
///     - If there are missing fields, create a new plan boundary. This allows us to know that we
///       should keep a reference to response objects for that selection sets so that children plan
///       don't need to search for for them. We plan any missing fields with the same logic.
/// 2. Collect attributed fields to know what to expect from the response. This follows the field
///    collection logic from GraphQL. If the selection set is simple enough (no type conditions
///    typically), we can do it in advance and store it. Otherwise we generate what we can for
///    later.
/// 3. Generate the actual plans for each resolver, allowing them to cache what they can for later.
///    During execution, those Plans create Executors with the actual response objects that do the
///    real work.
///
pub(super) struct OperationSolver<'a> {
    pub(super) schema: &'a Schema,
    pub(super) variables: &'a Variables,
    pub(super) operation: &'a mut Operation,
    plan_edges: HashSet<ParentToChildEdge>,
    plans: Vec<Plan>,
    pub(super) field_to_plan_id: Vec<Option<PlanId>>,
    pub(super) solved_requirements: Vec<(SelectionSetId, SolvedRequiredFieldSet)>,
}

id_newtypes::index! {
    OperationSolver<'a>.plans[PlanId] => Plan,
    OperationSolver<'a>.field_to_plan_id[FieldId] => Option<PlanId>,
}

impl<'a> OperationSolver<'a> {
    pub(super) fn new(schema: &'a Schema, variables: &'a Variables, operation: &'a mut Operation) -> Self {
        Self {
            schema,
            variables,
            field_to_plan_id: vec![None; operation.fields.len()],
            operation,
            plans: Vec::new(),
            plan_edges: HashSet::new(),
            solved_requirements: Vec::new(),
        }
    }

    pub(super) fn solve(mut self) -> PlanningResult<()> {
        self.plan_all_fields()?;
        let Self {
            operation,
            plan_edges,
            plans,
            field_to_plan_id,
            solved_requirements,
            ..
        } = self;

        operation.plans = plans;
        operation.plan_edges = plan_edges.into_iter().collect();
        operation.solved_requirements = solved_requirements;
        operation.field_to_plan_id = field_to_plan_id
            .into_iter()
            .enumerate()
            .map(|(i, maybe_plan_id)| match maybe_plan_id {
                Some(plan_id) => plan_id,
                None => {
                    let name = &operation.response_keys[operation.fields[i].response_key()];
                    unreachable!("No plan was associated with field:\n{name}");
                }
            })
            .collect();

        operation.solved_requirements.sort_unstable_by_key(|(id, _)| *id);
        operation.plan_edges.sort_unstable();
        Ok(())
    }

    /// Step 1 of the planning, attributed all fields to a plan and satisfying their requirements.
    fn plan_all_fields(&mut self) -> PlanningResult<()> {
        // The root plan is always introspection which also lets us handle operations like:
        // query { __typename }
        let introspection = self.schema.walker().introspection_metadata();

        let walker = self.walker();
        let (introspection_field_ids, field_ids): (Vec<_>, Vec<_>) =
            walker.selection_set().as_ref().field_ids.iter().partition(|field_id| {
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
    fn plan_query(&mut self, field_ids: Vec<FieldId>) -> PlanningResult<()> {
        let id = self.operation.root_selection_set_id;
        SelectionSetSolver::new(self, &QueryPath::default(), None).solve(id, Vec::new(), field_ids)
    }

    /// Mutation is a special case because root fields need to execute in order. So planning each
    /// field individually and setting up plan dependencies between them to ensures proper
    /// execution order.
    fn plan_mutation(&mut self, field_ids: Vec<FieldId>) -> PlanningResult<()> {
        let mut groups = self
            .walker()
            .group_by_response_key_sorted_by_query_position(field_ids)
            .into_values()
            .collect::<Vec<_>>();
        // Ordering groups by their position in the query, ensuring proper ordering of plans.
        groups.sort_unstable_by_key(|field_ids| self.operation[field_ids[0]].query_position());

        let mut maybe_previous_plan_id: Option<PlanId> = None;

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
                .ok_or_else(|| PlanningError::CouldNotPlanAnyField {
                    missing: vec![self.operation.response_keys[field.response_key()].to_string()],
                    query_path: vec![],
                })?;

            let plan_id = self.push_plan(QueryPath::default(), resolver.id(), &field_ids)?;

            if let Some(parent) = maybe_previous_plan_id {
                self.push_plan_dependency(ParentToChildEdge { parent, child: plan_id });
            }
            maybe_previous_plan_id = Some(plan_id);
        }
        Ok(())
    }

    /// Obviously providable fields have no requirements and can be provided by the current
    /// resolver.
    pub(super) fn plan_obviously_providable_subselections(
        &mut self,
        path: &QueryPath,
        logic: &PlanningLogic<'a>,
        field_ids: &[FieldId],
    ) -> PlanningResult<()> {
        let plan_id = logic.plan_id();
        self.attribute_fields(field_ids, plan_id);
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
    ) -> PlanningResult<()> {
        let walker = self.walker();
        let (obviously_plannable_field_ids, unplanned_field_ids): (Vec<_>, Vec<_>) =
            self.operation[id].field_ids.iter().copied().partition(|field_id| {
                if let Some(definition) = walker.walk(*field_id).definition() {
                    logic.is_providable(definition.id())
                        && definition.requires(logic.resolver().subgraph_id()).is_empty()
                } else {
                    true
                }
            });

        self.plan_obviously_providable_subselections(path, logic, &obviously_plannable_field_ids)?;

        if !unplanned_field_ids.is_empty() {
            SelectionSetSolver::new(self, path, Some(logic)).solve(
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
    ) -> PlanningResult<PlanId> {
        let plan_id = PlanId::from(self.plans.len());
        tracing::trace!(
            "Creating {plan_id} ({}): {}",
            self.schema.walk(resolver_id).name(),
            field_ids.iter().format_with(", ", |id, f| f(&format_args!(
                "{}",
                self.walker().walk(*id).response_key_str()
            )))
        );
        self.plans.push(Plan { resolver_id });
        let logic = PlanningLogic::new(plan_id, self.schema.walk(resolver_id));
        self.plan_obviously_providable_subselections(&query_path, &logic, field_ids)?;
        Ok(plan_id)
    }

    pub fn push_plan_dependency(&mut self, edge: ParentToChildEdge) {
        if edge.parent != edge.child {
            self.plan_edges.insert(edge);
        }
    }

    pub fn attribute_fields(&mut self, fields: &[FieldId], plan_id: PlanId) {
        for id in fields {
            self[*id] = Some(plan_id);
        }
    }
}
