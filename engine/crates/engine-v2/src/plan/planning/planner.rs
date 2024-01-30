use engine_parser::types::OperationType;
use fnv::{FnvHashMap, FnvHashSet};
use schema::{FieldId, FieldResolverWalker, ResolverId, Schema};
use std::{borrow::Cow, collections::hash_map::Entry, num::NonZeroU16};

use super::{
    attribution::AttributionLogic,
    boundary::{BoundaryParent, BoundaryPlanner},
    collect::Collector,
    walker_ext::GroupedByFieldId,
    PlanningError, PlanningResult,
};
use crate::{
    plan::{
        ExecutionPlanId, LogicalPlan, OperationPlan, ParentToChildEdge, PlanBoundaryId, PlanId, PlanInput, PlanOutput,
    },
    request::{
        BoundField, BoundFieldId, BoundSelection, BoundSelectionSet, BoundSelectionSetId, EntityType, FlatSelectionSet,
        FlatTypeCondition, Operation, OperationWalker, QueryPath,
    },
    response::{ReadSelectionSet, ResponseEdge, ResponseKeys, UnpackedResponseEdge},
    sources::ExecutionPlan,
    utils::IdRange,
};

pub(in crate::plan) struct Planner<'schema> {
    pub(super) schema: &'schema Schema,
    pub(super) operation: Operation,
    extra_field_edges: FnvHashMap<FieldId, ResponseEdge>,

    // -- Operation --
    field_attribution: Vec<Option<PlanId>>,
    selection_set_attribution: Vec<Option<PlanId>>,

    // -- Plans --
    plans: Vec<LogicalPlan>,
    plan_input_selection_sets: Vec<Option<ReadSelectionSet>>,
    // PlanId -> PlanRootSelectionSet
    plan_root_selection_sets: Vec<PlanRootSelectionSet>,
    // Child -> Parent(s)
    plan_to_dependencies: FnvHashMap<PlanId, FnvHashSet<PlanId>>,
    plan_boundaries_count: usize,
    // PlanId -> Vec<PlanBoundaryId>
    plan_to_children_tmp_boundary_ids: Vec<Vec<TemporaryPlanBoundaryId>>,
    // PlanId -> Option<PlanBoundaryId>
    plan_to_parent_tmp_boundary_id: Vec<Option<TemporaryPlanBoundaryId>>,
}

pub(super) struct PlanRootSelectionSet {
    pub ids: Vec<BoundSelectionSetId>,
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

impl<'schema> Planner<'schema> {
    pub fn new(schema: &'schema Schema, operation: Operation) -> Self {
        Self {
            schema,
            extra_field_edges: FnvHashMap::default(),
            field_attribution: vec![None; operation.fields.len()],
            selection_set_attribution: vec![None; operation.selection_sets.len()],
            operation,
            plans: Vec::new(),
            plan_input_selection_sets: Vec::new(),
            plan_root_selection_sets: Vec::new(),
            plan_boundaries_count: 0,
            plan_to_children_tmp_boundary_ids: Vec::new(),
            plan_to_parent_tmp_boundary_id: Vec::new(),
            plan_to_dependencies: FnvHashMap::default(),
        }
    }

    pub fn finalize_operation(mut self) -> PlanningResult<OperationPlan> {
        let field_attribution = self
            .field_attribution
            .iter()
            .enumerate()
            .map(|(i, maybe_plan_id)| match maybe_plan_id {
                Some(plan_id) => *plan_id,
                None => {
                    let bound_field_id = BoundFieldId::from(i);
                    let bound_field = &self.walker().walk(bound_field_id);
                    unreachable!("No plan was associated with field:\n{bound_field:#?}");
                }
            })
            .collect();

        self.selection_set_attribution[usize::from(self.operation.root_selection_set_id)] = Some(PlanId::from(0));
        let selection_set_attribution = self
            .selection_set_attribution
            .iter()
            .enumerate()
            .map(|(i, maybe_plan_id)| match maybe_plan_id {
                Some(plan_id) => *plan_id,
                None => {
                    let bound_selection_set_id = BoundSelectionSetId::from(i);
                    let bound_selection_set = self.walker().walk(bound_selection_set_id);
                    unreachable!("No plan was associated with selection set:\n{bound_selection_set:#?})");
                }
            })
            .collect();

        let Self {
            schema,
            operation: bound_operation,
            plans,
            plan_input_selection_sets,
            plan_root_selection_sets,
            plan_to_dependencies,
            plan_boundaries_count,
            plan_to_children_tmp_boundary_ids,
            plan_to_parent_tmp_boundary_id,
            ..
        } = self;

        let mut plan_to_output_boundary_ids = Vec::with_capacity(plans.len());
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
        let mut plan_inputs = Vec::with_capacity(plans.len());
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

        let mut execution_plan_dependencies_count = vec![0; plans.len()];
        let mut execution_plans_parent_to_child_edges = Vec::with_capacity(plans.len());
        for (&child, dependencies) in &plan_to_dependencies {
            for &parent in dependencies {
                // For now there is a 1to1 mapping between logical plans and execution plans.
                let parent = ExecutionPlanId::from(usize::from(parent));
                let child = ExecutionPlanId::from(usize::from(child));
                execution_plan_dependencies_count[usize::from(child)] += 1;
                execution_plans_parent_to_child_edges.push(ParentToChildEdge { parent, child });
            }
        }

        let mut operation = OperationPlan {
            bound_operation,
            field_attribution,
            selection_set_attribution,
            plan_inputs,
            plan_outputs: Vec::with_capacity(plans.len()),
            collected_concrete_selection_sets: Vec::with_capacity(plans.len()),
            collected_concrete_fields: Vec::with_capacity(plans.len()),
            plans,
            execution_plans: Vec::new(),
            execution_plans_parent_to_child_edges,
            execution_plan_dependencies_count,
            plan_boundary_consummers_count,
            collected_conditional_selection_sets: Vec::new(),
            collected_conditional_fields: Vec::new(),
        };
        operation.execution_plans_parent_to_child_edges.sort_unstable();

        for (i, PlanRootSelectionSet { ids, entity_type }) in plan_root_selection_sets.into_iter().enumerate() {
            let plan_id = PlanId::from(i);
            let ty = operation[ids[0]].ty;
            let collected_selection_set_id = Collector::new(schema, &mut operation, plan_id).collect(ids)?;
            operation.plan_outputs.push(PlanOutput {
                type_condition: FlatTypeCondition::flatten(self.schema, ty, vec![entity_type.into()]),
                entity_type,
                collected_selection_set_id,
                boundary_ids: plan_to_output_boundary_ids[i],
            });
        }

        // For now there is a 1to1 mapping between logical plans and execution plans.
        let mut execution_plans = Vec::with_capacity(operation.plans.len());
        for (i, plan) in operation.plans.iter().enumerate() {
            let resolver = self.schema.walker().walk(plan.resolver_id).with_own_names();
            let plan_id = ExecutionPlanId::from(i);
            execution_plans.push(ExecutionPlan::build(
                resolver,
                operation.plan_walker(self.schema, plan_id, None),
            )?);
        }
        operation.execution_plans = execution_plans;

        Ok(operation)
    }
}

impl<'schema> Planner<'schema> {
    pub(super) fn plan_all(&mut self) -> PlanningResult<()> {
        let (introspection_selection_set, selection_set) = self
            .walker()
            .flatten_selection_sets(vec![self.operation.root_selection_set_id])
            .partition_fields(|flat_field| {
                let bound_field = &self.operation[flat_field.bound_field_id];
                if let Some(schema_field_id) = bound_field.schema_field_id() {
                    self.schema.walker().walk(schema_field_id).resolvers().len() == 0
                } else {
                    true
                }
            });

        // Planning introspection fields first.
        self.plan_introspection(introspection_selection_set)?;

        if matches!(self.operation.ty, OperationType::Mutation) {
            self.plan_mutation(selection_set)?;
        } else {
            self.plan_query(selection_set)?;
        }

        Ok(())
    }

    fn plan_query(&mut self, selection_set: FlatSelectionSet) -> PlanningResult<()> {
        BoundaryPlanner::plan(self, &QueryPath::default(), None, selection_set)?;
        Ok(())
    }

    fn plan_introspection(&mut self, selection_set: FlatSelectionSet) -> PlanningResult<()> {
        if !selection_set.is_empty() {
            self.push_plan(
                QueryPath::default(),
                self.schema.introspection_resolver_id(),
                EntityType::Object(self.operation.root_object_id),
                selection_set,
            )?;
        }
        Ok(())
    }

    fn plan_mutation(&mut self, mut selection_set: FlatSelectionSet) -> PlanningResult<()> {
        let entity_type = EntityType::Object(self.operation.root_object_id);

        let mut groups = self
            .walker()
            .group_by_response_key(std::mem::replace(&mut selection_set.fields, Vec::with_capacity(0)))
            .into_values()
            .collect::<Vec<_>>();
        // Ordering groups by their position in the query, ensuring proper ordering of plans.
        groups.sort_unstable_by(|a, b| a.edge.cmp(&b.edge));

        let mut maybe_previous_plan_id: Option<PlanId> = None;

        for group in groups {
            let field_id = self.operation[group.final_bound_field_id]
                .schema_field_id()
                .expect("Introspection resolver should have taken metadata fields");

            let FieldResolverWalker {
                resolver,
                field_requires,
            } = self.schema.walker().walk(field_id).resolvers().next().ok_or_else(|| {
                PlanningError::CouldNotPlanAnyField {
                    missing: vec![self
                        .walker()
                        .walk(group.final_bound_field_id)
                        .response_key_str()
                        .to_string()],
                    query_path: vec![],
                }
            })?;

            if !field_requires.is_empty() {
                return Err(PlanningError::CouldNotSatisfyRequires {
                    resolver: resolver.name().to_string(),
                    field: field_requires
                        .into_iter()
                        .map(|item| self.schema.walker().walk(item.field_id).name())
                        .collect(),
                });
            }

            let plan_id = self.push_plan(
                QueryPath::default(),
                resolver.id(),
                entity_type,
                selection_set.clone_with_fields(group.items),
            )?;

            if let Some(parent) = maybe_previous_plan_id {
                self.insert_plan_dependency(ParentToChildEdge { parent, child: plan_id });
            }
            maybe_previous_plan_id = Some(plan_id);
        }
        Ok(())
    }

    fn plan_children(
        &mut self,
        path: &QueryPath,
        plan_id: PlanId,
        resolver_id: ResolverId,
        selection_set: FlatSelectionSet,
    ) -> PlanningResult<()> {
        if let Some(subselections) = self.walker().flatten_subselection_sets(&selection_set.fields) {
            let logic = AttributionLogic::CompatibleResolver {
                plan_id,
                resolver: self.schema.walk(resolver_id),
                providable: Default::default(),
            };
            self.recursive_plan_children(path, &logic, subselections)?;
        }
        Ok(())
    }

    fn recursive_plan_children(
        &mut self,
        path: &QueryPath,
        logic: &AttributionLogic<'schema>,
        selection_set: FlatSelectionSet,
    ) -> PlanningResult<()> {
        let (providable, missing) = {
            selection_set.partition_fields(|field| {
                self.operation[field.bound_field_id]
                    .schema_field_id()
                    .map(|id| logic.is_providable(id))
                    // __typename is always providable if the selection_set could be
                    .unwrap_or(true)
            })
        };

        let plan_id = logic.plan_id();
        self.attribute_selection_set(&providable, plan_id);
        let grouped = self.walker().group_by_schema_field_id(&providable);
        for (schema_field_id, group) in &grouped {
            let bound_field = &self.operation[group.final_bound_field_id];
            let key = bound_field.response_key();
            if let Some(flat_selection_set) = self.walker().flatten_subselection_sets(&group.bound_field_ids) {
                self.attribute_selection_sets(&flat_selection_set.root_selection_set_ids, plan_id);
                self.recursive_plan_children(&path.child(key), &logic.child(*schema_field_id), flat_selection_set)?;
            }
        }

        if !missing.is_empty() {
            self.plan_boundary(path, logic, grouped, missing)?;
        }
        Ok(())
    }

    fn plan_boundary(
        &mut self,
        query_path: &QueryPath,
        logic: &AttributionLogic<'schema>,
        providable: FnvHashMap<FieldId, GroupedByFieldId>,
        missing: FlatSelectionSet,
    ) -> PlanningResult<()> {
        let parent = BoundaryParent { logic, providable };
        let children = BoundaryPlanner::plan(self, query_path, Some(parent), missing)?;

        let parent = logic.plan_id();
        let plan_boundary_id = self.new_boundary(parent)?;
        for child in children {
            self.insert_parent_plan(plan_boundary_id, ParentToChildEdge { parent, child });
        }

        Ok(())
    }
}

// Utilities
impl<'schema> Planner<'schema> {
    pub fn walker(&self) -> OperationWalker<'_> {
        self.operation.walker_with(self.schema.walker())
    }

    pub fn generate_unique_edge_for(&mut self, field_id: FieldId) -> ResponseEdge {
        // When the resolver supports aliases, we must ensure that extra fields
        // don't collide with existing response keys. And to avoid duplicates
        // during field collection, we have a single unique name per field id.
        match self.extra_field_edges.entry(field_id) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let name = Self::find_available_response_key(self.schema, &self.operation.response_keys, field_id);
                let key = self.operation.response_keys.get_or_intern(name.as_ref());
                let edge = UnpackedResponseEdge::ExtraField(key).pack();
                entry.insert(edge);
                edge
            }
        }
    }

    fn find_available_response_key<'b>(
        schema: &'b Schema,
        response_keys: &ResponseKeys,
        field_id: FieldId,
    ) -> Cow<'b, str> {
        let schema_name = schema.walker().walk(field_id).name();
        if !response_keys.contains(schema_name) {
            return Cow::Borrowed(schema_name);
        }
        let short_id = hex::encode(u32::from(field_id).to_be_bytes())
            .trim_start_matches('0')
            .to_uppercase();
        let name = format!("_{}{}", schema_name, short_id);
        // name is unique, but may collide with existing keys so
        // iterating over candidates until we find a valid one.
        // This is only a safeguard, it most likely won't ever run.
        if !response_keys.contains(&name) {
            return Cow::Owned(name);
        }
        let mut index = 0;
        loop {
            let candidate = format!("{name}{index}");
            if !response_keys.contains(&candidate) {
                return Cow::Owned(candidate);
            }
            index += 1;
        }
    }

    pub fn push_extra_field(
        &mut self,
        plan_id: PlanId,
        parent_selection_set_id: Option<BoundSelectionSetId>,
        field: BoundField,
    ) -> BoundFieldId {
        let id = BoundFieldId::from(self.operation.fields.len());
        self.field_attribution.push(Some(plan_id));
        self.operation.fields.push(field);
        if let Some(selection_set_id) = parent_selection_set_id {
            self.selection_set_attribution[usize::from(selection_set_id)] = Some(plan_id);
            self.operation[selection_set_id].items.push(BoundSelection::Field(id));
            self.operation.field_to_parent.push(selection_set_id);
        }
        id
    }

    pub fn push_extra_selection_set(
        &mut self,
        plan_id: PlanId,
        selection_set: BoundSelectionSet,
    ) -> BoundSelectionSetId {
        let id = BoundSelectionSetId::from(self.operation.selection_sets.len());
        for item in &selection_set.items {
            if let BoundSelection::Field(bound_field_id) = item {
                self.operation.field_to_parent[usize::from(*bound_field_id)] = id;
            }
        }
        self.operation.selection_sets.push(selection_set);
        self.selection_set_attribution.push(Some(plan_id));
        id
    }

    pub fn push_plan(
        &mut self,
        path: QueryPath,
        resolver_id: ResolverId,
        entity_type: EntityType,
        selection_set: FlatSelectionSet,
    ) -> PlanningResult<PlanId> {
        let id = PlanId::from(self.plans.len());
        tracing::debug!(
            "Creating new plan {id} at '{}' for entity '{}': {}",
            self.walker().walk(&path),
            self.schema.walk(schema::Definition::from(entity_type)).name(),
            selection_set
                .fields
                .iter()
                .map(|field| self.walker().walk(field.bound_field_id).response_key_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        self.plans.push(LogicalPlan {
            resolver_id,
            path: path.clone(),
        });
        self.plan_to_children_tmp_boundary_ids.push(Vec::new());
        self.plan_to_parent_tmp_boundary_id.push(None);
        self.plan_input_selection_sets.push(None);
        self.plan_root_selection_sets.push(PlanRootSelectionSet {
            ids: selection_set.root_selection_set_ids.clone(),
            entity_type,
        });
        self.attribute_selection_set(&selection_set, id);
        self.plan_children(&path, id, resolver_id, selection_set)?;
        Ok(id)
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

    pub fn get_plan(&self, plan_id: PlanId) -> &LogicalPlan {
        &self.plans[usize::from(plan_id)]
    }

    pub fn insert_plan_dependency(&mut self, edge: ParentToChildEdge<PlanId>) {
        self.plan_to_dependencies
            .entry(edge.child)
            .or_default()
            .insert(edge.parent);
    }

    pub fn insert_parent_plan(&mut self, plan_boundary_id: TemporaryPlanBoundaryId, edge: ParentToChildEdge<PlanId>) {
        self.insert_plan_dependency(edge);
        self.plan_to_parent_tmp_boundary_id[usize::from(edge.child)] = Some(plan_boundary_id);
    }

    pub fn get_field_plan(&self, id: BoundFieldId) -> Option<PlanId> {
        self.field_attribution[usize::from(id)]
    }

    pub fn attribute_selection_set(&mut self, selection_set: &FlatSelectionSet, plan_id: PlanId) {
        for field in selection_set {
            self.field_attribution[usize::from(field.bound_field_id)] = Some(plan_id);
            // Ignoring the first selection_set which comes from the parent plan.
            for id in &field.selection_set_path {
                self.selection_set_attribution[usize::from(*id)].get_or_insert(plan_id);
            }
        }
    }

    pub fn attribute_selection_sets(&mut self, selection_set_ids: &[BoundSelectionSetId], plan_id: PlanId) {
        for id in selection_set_ids {
            self.selection_set_attribution[usize::from(*id)].get_or_insert(plan_id);
        }
    }
}
