use engine_parser::types::OperationType;
use id_newtypes::IdRange;
use itertools::Itertools;
use schema::{ResolverId, Schema};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU16,
};

use super::{
    boundary::BoundarySelectionSetPlanner, collect::Collector, logic::PlanningLogic, PlanningError, PlanningResult,
};
use crate::{
    operation::{
        Field, FieldId, Operation, OperationWalker, QueryPath, Selection, SelectionSet, SelectionSetId, Variables,
    },
    plan::{
        flatten_selection_sets, EntityType, FlatField, FlatSelectionSet, FlatTypeCondition, OperationPlan,
        ParentToChildEdge, PlanBoundaryId, PlanId, PlanInput, PlanOutput, PlannedResolver,
    },
    response::ReadSelectionSet,
    sources::Plan,
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
pub(super) struct Planner<'ctx> {
    pub(super) schema: &'ctx Schema,
    pub(super) variables: &'ctx Variables,
    pub(super) operation: Operation,

    // -- Operation --
    // Associates for each field/selection a plan. Attributions is added incrementally
    // and used to determine dependencies between plans. It's later used in OperationPlan
    // to filter the selection that Executors see, only for their plan.
    // BoundFieldId -> Option<PlanId>
    field_to_plan_id: Vec<Option<PlanId>>,
    // BoundSelectionSetId -> Option<PlanId>
    selection_set_to_plan_id: Vec<Option<PlanId>>,

    // -- Plans --
    planned_resolvers: Vec<PlannedResolver>,
    plan_input_selection_sets: Vec<Option<ReadSelectionSet>>,
    // PlanId -> PlanRootSelectionSet
    plan_root_selection_sets: Vec<PlanRootSelectionSet>,
    // Child -> Parent(s)
    plan_to_dependencies: HashMap<PlanId, HashSet<PlanId>>,
    plan_boundaries_count: usize,
    // PlanId -> Vec<PlanBoundaryId>
    plan_to_children_tmp_boundary_ids: Vec<Vec<TemporaryPlanBoundaryId>>,
    // PlanId -> Option<PlanBoundaryId>
    plan_to_parent_tmp_boundary_id: Vec<Option<TemporaryPlanBoundaryId>>,
}

pub(super) struct PlanRootSelectionSet {
    pub ids: Vec<SelectionSetId>,
    pub entity_type: EntityType,
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

impl<'ctx> Planner<'ctx> {
    pub(super) fn new(schema: &'ctx Schema, variables: &'ctx Variables, operation: Operation) -> Self {
        Self {
            schema,
            variables,
            field_to_plan_id: vec![None; operation.fields.len()],
            selection_set_to_plan_id: vec![None; operation.selection_sets.len()],
            operation,
            planned_resolvers: Vec::new(),
            plan_input_selection_sets: Vec::new(),
            plan_root_selection_sets: Vec::new(),
            plan_boundaries_count: 0,
            plan_to_children_tmp_boundary_ids: Vec::new(),
            plan_to_parent_tmp_boundary_id: Vec::new(),
            plan_to_dependencies: HashMap::default(),
        }
    }
}

impl<'schema> Planner<'schema> {
    /// Step 1 of the planning, attributed all fields to a plan and satisfying their requirements.
    pub(super) fn plan_all_fields(&mut self) -> PlanningResult<()> {
        // The root plan is always introspection which also lets us handle operations like:
        // query { __typename }
        let (introspection_subgraph_id, introspection_resolver_id) = self
            .schema
            .data_sources
            .introspection
            .metadata
            .as_ref()
            .map(|m| (m.subgraph_id, m.resolver_id))
            .unwrap();
        let (introspection_selection_set, selection_set) =
            flatten_selection_sets(self.schema, &self.operation, vec![self.operation.root_selection_set_id])
                .partition_fields(|flat_field| {
                    let field = &self.operation[flat_field.id];
                    if let Some(definition_id) = field.definition_id() {
                        self.schema
                            .walker()
                            .walk(definition_id)
                            .is_resolvable_in(introspection_subgraph_id)
                    } else {
                        true
                    }
                });

        if !introspection_selection_set.is_empty() {
            self.push_plan(
                QueryPath::default(),
                introspection_resolver_id,
                EntityType::Object(self.operation.root_object_id),
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
        )?;
        Ok(())
    }

    /// Mutation is a special case because root fields need to execute in order. So planning each
    /// field individually and setting up plan dependencies between them to ensures proper
    /// execution order.
    fn plan_mutation(&mut self, mut selection_set: FlatSelectionSet) -> PlanningResult<()> {
        let entity_type = EntityType::Object(self.operation.root_object_id);

        let fields = std::mem::take(&mut selection_set.fields);
        let mut groups = self
            .walker()
            .group_by_response_key_sorted_by_query_position(fields.into_iter().map(|field| field.id))
            .into_values()
            .collect::<Vec<_>>();
        // Ordering groups by their position in the query, ensuring proper ordering of plans.
        groups.sort_unstable_by_key(|field_ids| self.operation[field_ids[0]].query_position());

        let mut maybe_previous_plan_id: Option<PlanId> = None;

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
                self.insert_plan_dependency(ParentToChildEdge { parent, child: plan_id });
            }
            maybe_previous_plan_id = Some(plan_id);
        }
        Ok(())
    }

    /// After planning the individual fields, we plan their selection sets if any.
    fn plan_providable_subselections(
        &mut self,
        path: &QueryPath,
        logic: &PlanningLogic<'schema>,
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
        logic: &PlanningLogic<'schema>,
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
        logic: &PlanningLogic<'schema>,
        providable: FlatSelectionSet,
        missing: FlatSelectionSet,
    ) -> PlanningResult<()> {
        let children = BoundarySelectionSetPlanner::plan(self, query_path, Some(logic), providable, missing)?;

        let parent = logic.plan_id();
        let plan_boundary_id = self.new_boundary(parent)?;
        for child in children {
            self.insert_parent_plan(plan_boundary_id, ParentToChildEdge { parent, child });
        }

        Ok(())
    }
}

impl<'schema> Planner<'schema> {
    /// This function is a bit... heavy. it generates the OperationPlan and several parts need some
    /// post-processing to return something that makes sense. It does the step 2 & 3 of the
    /// planning.
    pub(super) fn finalize_operation(mut self) -> PlanningResult<OperationPlan> {
        //
        // -- Ensuring we attributed all fields & selection set --
        //
        let field_attribution = self
            .field_to_plan_id
            .iter()
            .enumerate()
            .map(|(i, maybe_plan_id)| match maybe_plan_id {
                Some(plan_id) => *plan_id,
                None => {
                    let field = &self.walker().walk(FieldId::from(i));
                    unreachable!("No plan was associated with field:\n{field:#?}");
                }
            })
            .collect();

        self.selection_set_to_plan_id[usize::from(self.operation.root_selection_set_id)] = Some(PlanId::from(0));
        let selection_set_attribution = self
            .selection_set_to_plan_id
            .iter()
            .enumerate()
            .map(|(i, maybe_plan_id)| match maybe_plan_id {
                Some(plan_id) => *plan_id,
                None => {
                    let selection_set_id = SelectionSetId::from(i);
                    let selection_set = self.walker().walk(selection_set_id);
                    unreachable!("No plan was associated with selection set:\n{selection_set:#?})");
                }
            })
            .collect();

        let Self {
            schema,
            operation,
            variables,
            planned_resolvers,
            plan_input_selection_sets,
            plan_root_selection_sets,
            plan_to_dependencies,
            plan_boundaries_count,
            plan_to_children_tmp_boundary_ids,
            plan_to_parent_tmp_boundary_id,
            ..
        } = self;

        //
        // -- Generating the plan boundaries, dependencies & inputs --
        //
        // Before we used TemporaryPlanBoundaryId to keep track of the boundaries. But we need them
        // to be sequential for a given plan. This allows us to access store boundaries in Vec and
        // access them with an offset when ingesting data into the response.
        // We also need plan boundary ids to be unique for the OperationExecutionState which needs
        // the number of consummers (children plan) for a given boundary.
        let mut plan_to_output_boundary_ids = Vec::with_capacity(planned_resolvers.len());
        let tmp_boundary_id_to_boundary_id = {
            let mut mapping = vec![PlanBoundaryId::from(0); plan_boundaries_count];
            let mut n: usize = 0;
            for tmp_boundary_ids in &plan_to_children_tmp_boundary_ids {
                let start = PlanBoundaryId::from(n);
                for tmp_boundary_id in tmp_boundary_ids {
                    let id = PlanBoundaryId::from(n);
                    n += 1;
                    mapping[usize::from(*tmp_boundary_id)] = id;
                }
                let end = PlanBoundaryId::from(n);
                plan_to_output_boundary_ids.push(IdRange { start, end });
            }
            mapping
        };

        let mut plan_boundary_consummers_count = vec![0; plan_boundaries_count];
        let mut plan_inputs = Vec::with_capacity(planned_resolvers.len());
        for (maybe_tmp_id, maybe_selection_set) in plan_to_parent_tmp_boundary_id
            .into_iter()
            .zip(plan_input_selection_sets)
        {
            if let Some(tmp_id) = maybe_tmp_id {
                let boundary_id = tmp_boundary_id_to_boundary_id[usize::from(tmp_id)];
                plan_boundary_consummers_count[usize::from(boundary_id)] += 1;
                plan_inputs.push(Some(PlanInput {
                    selection_set: maybe_selection_set.expect("Missing input selection set"),
                    boundary_id,
                }));
            } else {
                plan_inputs.push(None);
            }
        }

        let mut plan_dependencies_count = vec![0; planned_resolvers.len()];
        let mut plans_parent_to_child_edges = Vec::with_capacity(planned_resolvers.len());
        for (&child, dependencies) in &plan_to_dependencies {
            for &parent in dependencies {
                plan_dependencies_count[usize::from(child)] += 1;
                plans_parent_to_child_edges.push(ParentToChildEdge { parent, child });
            }
        }

        //
        // -- Collecting fields for the plan output --
        //
        let mut operation_plan = OperationPlan {
            field_to_plan_id: field_attribution,
            selection_to_plan_id: selection_set_attribution,
            plan_inputs,
            plan_outputs: Vec::with_capacity(planned_resolvers.len()),
            collected_selection_sets: Vec::with_capacity(planned_resolvers.len()),
            collected_fields: Vec::with_capacity(planned_resolvers.len()),
            selection_set_to_collected: vec![None; operation.selection_sets.len()],
            operation,
            conditional_selection_sets: Vec::new(),
            conditional_fields: Vec::new(),
            plans: Vec::with_capacity(planned_resolvers.len()),
            planned_resolvers,
            plan_parent_to_child_edges: plans_parent_to_child_edges,
            plan_dependencies_count,
            plan_boundary_consummers_count,
        };
        operation_plan.plan_parent_to_child_edges.sort_unstable();

        for (i, PlanRootSelectionSet { ids, entity_type }) in plan_root_selection_sets.into_iter().enumerate() {
            let plan_id = PlanId::from(i);
            let ty = operation_plan[ids[0]].ty;
            let collected_selection_set_id =
                Collector::new(schema, variables, &mut operation_plan, plan_id).collect(ids)?;
            operation_plan.plan_outputs.push(PlanOutput {
                type_condition: FlatTypeCondition::flatten(schema, ty, vec![entity_type.into()]),
                entity_type,
                collected_selection_set_id,
                boundary_ids: plan_to_output_boundary_ids[i],
            });
        }

        //
        // -- Generating the actual plans --
        //
        let mut execution_plans = Vec::with_capacity(operation_plan.plans.len());
        for (i, PlannedResolver { resolver_id, .. }) in operation_plan.planned_resolvers.iter().enumerate() {
            let resolver = schema.walker().walk(*resolver_id).with_own_names();
            let plan_id = PlanId::from(i);
            execution_plans.push(Plan::build(
                resolver,
                operation_plan.ty,
                operation_plan.walker_with(schema, variables, plan_id),
            )?);
        }
        operation_plan.plans = execution_plans;

        Ok(operation_plan)
    }
}

// Utilities
impl<'schema> Planner<'schema> {
    pub fn walker(&self) -> OperationWalker<'_> {
        self.operation.walker_with(self.schema.walker(), self.variables)
    }

    pub fn push_extra_field(
        &mut self,
        plan_id: PlanId,
        parent_selection_set_id: Option<SelectionSetId>,
        field: Field,
    ) -> FieldId {
        let id = FieldId::from(self.operation.fields.len());
        self.field_to_plan_id.push(Some(plan_id));
        self.operation.fields.push(field);
        if let Some(selection_set_id) = parent_selection_set_id {
            self.selection_set_to_plan_id[usize::from(selection_set_id)] = Some(plan_id);
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
        self.selection_set_to_plan_id.push(Some(plan_id));
        id
    }

    pub fn push_plan(
        &mut self,
        path: QueryPath,
        resolver_id: ResolverId,
        entity_type: EntityType,
        providable: &FlatSelectionSet,
    ) -> PlanningResult<PlanId> {
        let plan_id = PlanId::from(self.planned_resolvers.len());
        tracing::trace!(
            "Creating {plan_id} ({}) for entity '{}': {}",
            self.schema.walk(resolver_id).name(),
            self.schema.walk(schema::Definition::from(entity_type)).name(),
            providable.fields.iter().format_with(", ", |field, f| f(&format_args!(
                "{}",
                self.walker().walk(field.id).response_key_str()
            )))
        );
        self.planned_resolvers.push(PlannedResolver {
            resolver_id,
            path: path.clone(),
        });
        self.plan_to_children_tmp_boundary_ids.push(Vec::new());
        self.plan_to_parent_tmp_boundary_id.push(None);
        self.plan_input_selection_sets.push(None);
        self.plan_root_selection_sets.push(PlanRootSelectionSet {
            ids: providable.root_selection_set_ids.clone(),
            entity_type,
        });
        let logic = PlanningLogic::new(plan_id, self.schema.walk(resolver_id));
        self.plan_providable_subselections(&path, &logic, providable)?;
        Ok(plan_id)
    }

    pub fn new_boundary(&mut self, plan_id: PlanId) -> PlanningResult<TemporaryPlanBoundaryId> {
        let id = TemporaryPlanBoundaryId::from(self.plan_boundaries_count);
        self.plan_boundaries_count += 1;
        self.plan_to_children_tmp_boundary_ids[usize::from(plan_id)].push(id);
        Ok(id)
    }

    pub fn insert_plan_input_selection_set(&mut self, plan_id: PlanId, selection_set: ReadSelectionSet) {
        self.plan_input_selection_sets[usize::from(plan_id)] = Some(selection_set);
    }

    pub fn insert_plan_dependency(&mut self, edge: ParentToChildEdge) {
        self.plan_to_dependencies
            .entry(edge.child)
            .or_default()
            .insert(edge.parent);
    }

    pub fn insert_parent_plan(&mut self, plan_boundary_id: TemporaryPlanBoundaryId, edge: ParentToChildEdge) {
        self.insert_plan_dependency(edge);
        self.plan_to_parent_tmp_boundary_id[usize::from(edge.child)] = Some(plan_boundary_id);
    }

    pub fn get_field_plan(&self, id: FieldId) -> Option<PlanId> {
        self.field_to_plan_id[usize::from(id)]
    }

    pub fn attribute_selection_set(&mut self, selection_set: &FlatSelectionSet, plan_id: PlanId) {
        for field in selection_set {
            self.field_to_plan_id[usize::from(field.id)] = Some(plan_id);
            // Ignoring the first selection_set which comes from the parent plan.
            for id in &field.selection_set_path {
                self.selection_set_to_plan_id[usize::from(*id)].get_or_insert(plan_id);
            }
        }
    }

    pub fn attribute_selection_sets(&mut self, selection_set_ids: &[SelectionSetId], plan_id: PlanId) {
        for id in selection_set_ids {
            self.selection_set_to_plan_id[usize::from(*id)].get_or_insert(plan_id);
        }
    }
}
