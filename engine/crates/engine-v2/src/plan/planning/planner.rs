use engine_parser::types::OperationType;
use im::HashSet;
use itertools::Itertools;
use schema::{ResolverId, Schema};
use std::num::NonZeroU16;

use super::{
    boundary::BoundarySelectionSetPlanner, collect::OperationPlanBuilder, logic::PlanningLogic, PlanningError,
    PlanningResult,
};
use crate::{
    operation::{
        Field, FieldId, Operation, OperationWalker, ParentToChildEdge, Plan, PlanBoundaryId, PlanId, QueryPath,
        Selection, SelectionSet, SelectionSetId, Variables,
    },
    plan::{flatten_selection_sets, EntityId, FlatField, FlatSelectionSet, OperationPlan},
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
pub(super) struct Planner<'a> {
    pub(super) schema: &'a Schema,
    pub(super) variables: &'a Variables,
    pub(super) operation: Operation,
    plan_edges: HashSet<ParentToChildEdge>,
    next_plan_boundary_id: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct TemporaryPlanBoundaryId(NonZeroU16);

impl From<usize> for TemporaryPlanBoundaryId {
    fn from(value: usize) -> Self {
        Self(NonZeroU16::new(value as u16 + 1).unwrap())
    }
}

impl From<TemporaryPlanBoundaryId> for usize {
    fn from(value: TemporaryPlanBoundaryId) -> Self {
        value.0.get() as usize - 1
    }
}

impl<'a> Planner<'a> {
    pub(super) fn new(schema: &'a Schema, variables: &'a Variables, operation: Operation) -> Self {
        Self {
            schema,
            variables,
            operation,
            plan_edges: HashSet::new(),
            next_plan_boundary_id: 0,
        }
    }

    pub(super) fn plan(mut self) -> PlanningResult<OperationPlan> {
        self.plan_all_fields()?;
        self.operation.field_dependencies.sort_unstable();
        self.operation.plan_edges = self.plan_edges.into_iter().collect();
        self.operation.plan_edges.sort_unstable();
        OperationPlanBuilder::new(self.schema, self.variables, self.operation).build()
    }

    /// Step 1 of the planning, attributed all fields to a plan and satisfying their requirements.
    fn plan_all_fields(&mut self) -> PlanningResult<()> {
        // The root plan is always introspection which also lets us handle operations like:
        // query { __typename }
        let introspection = self.schema.walker().introspection_metadata();
        let (introspection_selection_set, selection_set) =
            flatten_selection_sets(self.schema, &self.operation, vec![self.operation.root_selection_set_id])
                .partition_fields(|flat_field| {
                    let field = &self.operation[flat_field.id];
                    if let Some(definition_id) = field.definition_id() {
                        self.schema
                            .walker()
                            .walk(definition_id)
                            .is_resolvable_in(introspection.subgraph_id)
                    } else {
                        true
                    }
                });

        if !introspection_selection_set.is_empty() {
            self.push_plan(
                QueryPath::default(),
                introspection.resolver_id,
                EntityId::Object(self.operation.root_object_id),
                &introspection_selection_set,
            )?;
        }

        if matches!(self.operation.ty, OperationType::Mutation) {
            self.plan_mutation(selection_set)?;
        } else {
            // Subscription are considered to be Queries for planning, they just happen to have
            // only one root field.
            self.plan_query(selection_set)?;
        }

        Ok(())
    }

    /// A query is simply treated as a plan boundary with no parent.
    fn plan_query(&mut self, selection_set: FlatSelectionSet) -> PlanningResult<()> {
        BoundarySelectionSetPlanner::plan(
            self,
            &QueryPath::default(),
            None,
            FlatSelectionSet::empty(selection_set.ty),
            selection_set,
        )
    }

    /// Mutation is a special case because root fields need to execute in order. So planning each
    /// field individually and setting up plan dependencies between them to ensures proper
    /// execution order.
    fn plan_mutation(&mut self, mut selection_set: FlatSelectionSet) -> PlanningResult<()> {
        let entity_type = EntityId::Object(self.operation.root_object_id);

        let fields = std::mem::take(&mut selection_set.fields);
        let mut groups = self
            .walker()
            .group_by_response_key_sorted_by_query_position(fields.into_iter().map(|field| field.id))
            .into_values()
            .collect::<Vec<_>>();
        // Ordering groups by their position in the query, ensuring proper ordering of plans.
        groups.sort_unstable_by_key(|field_ids| self.operation[field_ids[0]].query_position());

        let mut maybe_previous_plan_id: Option<PlanId> = None;
        let boundary_id = self.next_plan_boundary_id();

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

            let plan_id = self.push_plan(
                QueryPath::default(),
                resolver.id(),
                entity_type,
                &selection_set.clone_with_fields(
                    field_ids
                        .into_iter()
                        .map(|id| FlatField {
                            id,
                            type_condition: None,
                            selection_set_path: vec![selection_set.root_selection_set_ids[0]],
                        })
                        .collect(),
                ),
            )?;

            if let Some(parent) = maybe_previous_plan_id {
                self.push_plan_dependency(ParentToChildEdge {
                    parent,
                    child: plan_id,
                    boundary: boundary_id,
                });
            }
            maybe_previous_plan_id = Some(plan_id);
        }
        Ok(())
    }

    /// After planning the individual fields, we plan their selection sets if any.
    fn plan_providable_subselections(
        &mut self,
        path: &QueryPath,
        logic: &PlanningLogic<'a>,
        providable: &FlatSelectionSet,
    ) -> PlanningResult<()> {
        let plan_id = logic.plan_id();
        self.attribute_selection_set(providable, plan_id);
        let grouped = self
            .walker()
            .group_by_response_key_sorted_by_query_position(providable.fields.iter().map(|field| field.id));
        for (key, field_ids) in grouped {
            let subselection_set_ids = field_ids
                .iter()
                .filter_map(|id| self.operation[*id].selection_set_id())
                .collect::<Vec<_>>();
            if !subselection_set_ids.is_empty() {
                let definition_id = self.operation[field_ids[0]]
                    .definition_id()
                    .expect("wouldn't have a subselection");
                let flat_selection_set = flatten_selection_sets(self.schema, &self.operation, subselection_set_ids);
                self.attribute_selection_sets(&flat_selection_set.root_selection_set_ids, plan_id);
                self.plan_selection_set(&path.child(key), &logic.child(definition_id), flat_selection_set)?;
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
        selection_set: FlatSelectionSet,
    ) -> PlanningResult<()> {
        let walker = self.walker();
        let (obviously_providable, missing) = {
            selection_set.partition_fields(|field| {
                if let Some(definition) = walker.walk(field.id).definition() {
                    logic.is_providable(definition.id())
                        && definition.requires(logic.resolver().subgraph_id()).is_empty()
                } else {
                    // __typename is always providable if the selection_set could be
                    true
                }
            })
        };

        self.plan_providable_subselections(path, logic, &obviously_providable)?;

        if !missing.is_empty() {
            self.plan_boundary_selection_set(path, logic, obviously_providable, missing)?;
        }
        Ok(())
    }

    fn plan_boundary_selection_set(
        &mut self,
        query_path: &QueryPath,
        logic: &PlanningLogic<'a>,
        providable: FlatSelectionSet,
        missing: FlatSelectionSet,
    ) -> PlanningResult<()> {
        BoundarySelectionSetPlanner::plan(self, query_path, Some(logic), providable, missing)
    }

    pub fn walker(&self) -> OperationWalker<'_, (), ()> {
        self.operation.walker_with(self.schema.walker(), self.variables)
    }

    pub fn push_extra_field(
        &mut self,
        plan_id: PlanId,
        parent_selection_set_id: Option<SelectionSetId>,
        field: Field,
    ) -> FieldId {
        let id = FieldId::from(self.operation.fields.len());
        self.operation.field_to_plan_id.push(Some(plan_id));
        self.operation.fields.push(field);
        if let Some(selection_set_id) = parent_selection_set_id {
            self.operation.selection_set_to_plan_id[usize::from(selection_set_id)] = Some(plan_id);
            self.operation[selection_set_id].items.push(Selection::Field(id));
            self.operation.field_to_parent.push(selection_set_id);
        }
        id
    }

    pub fn push_extra_selection_set(&mut self, plan_id: PlanId, selection_set: SelectionSet) -> SelectionSetId {
        let id = SelectionSetId::from(self.operation.selection_sets.len());
        for item in &selection_set.items {
            if let Selection::Field(field_id) = item {
                self.operation.field_to_parent[usize::from(*field_id)] = id;
            }
        }
        self.operation.selection_sets.push(selection_set);
        self.operation.selection_set_to_plan_id.push(Some(plan_id));
        id
    }

    pub fn push_plan(
        &mut self,
        path: QueryPath,
        resolver_id: ResolverId,
        entity_type: EntityId,
        providable: &FlatSelectionSet,
    ) -> PlanningResult<PlanId> {
        let plan_id = PlanId::from(self.operation.plans.len());
        tracing::trace!(
            "Creating {plan_id} ({}) for entity '{}': {}",
            self.schema.walk(resolver_id).name(),
            self.schema.walk(schema::Definition::from(entity_type)).name(),
            providable.fields.iter().format_with(", ", |field, f| f(&format_args!(
                "{}",
                self.walker().walk(field.id).response_key_str()
            )))
        );
        self.operation.plans.push(Plan { resolver_id });
        let logic = PlanningLogic::new(plan_id, self.schema.walk(resolver_id));
        self.plan_providable_subselections(&path, &logic, providable)?;
        Ok(plan_id)
    }

    pub fn next_plan_boundary_id(&mut self) -> PlanBoundaryId {
        let id = self.next_plan_boundary_id;
        self.next_plan_boundary_id += 1;
        PlanBoundaryId::from(id)
    }

    pub fn push_plan_dependency(&mut self, edge: ParentToChildEdge) {
        self.plan_edges.insert(edge);
    }

    pub fn get_field_plan(&self, id: FieldId) -> Option<PlanId> {
        self.operation.field_to_plan_id[usize::from(id)]
    }

    pub fn attribute_selection_set(&mut self, selection_set: &FlatSelectionSet, plan_id: PlanId) {
        for field in selection_set {
            self.operation.field_to_plan_id[usize::from(field.id)] = Some(plan_id);
            self.attribute_selection_sets(&field.selection_set_path, plan_id)
        }
    }

    pub fn attribute_selection_sets(&mut self, selection_set_ids: &[SelectionSetId], plan_id: PlanId) {
        for id in selection_set_ids {
            self.operation.selection_set_to_plan_id[usize::from(*id)].get_or_insert(plan_id);
        }
    }
}
