use std::{borrow::Cow, collections::hash_map::Entry};

use fnv::FnvHashMap;
use schema::{FieldId, FieldResolverWalker, FieldSet, FieldSetItem, ResolverId, ResolverWalker};

use super::{
    attribution::AttributionLogic, planner::Planner, walker_ext::GroupedByFieldId, PlanningError, PlanningResult,
};
use crate::{
    plan::{ParentToChildEdge, PlanId},
    request::{
        BoundField, BoundFieldId, BoundSelection, BoundSelectionSet, BoundSelectionSetId, EntityType, FlatField,
        FlatSelectionSet, QueryPath, SelectionSetType,
    },
    response::{ReadField, ReadSelectionSet},
};

impl<'schema> Planner<'schema> {}

pub(super) struct BoundaryPlanner<'schema, 'a> {
    planner: &'a mut Planner<'schema>,
    query_path: &'a QueryPath,
    maybe_parent: Option<&'a AttributionLogic<'schema>>,
    children: Vec<PlanId>,
}

impl<'schema, 'a> BoundaryPlanner<'schema, 'a> {
    pub(super) fn plan(
        planner: &'a mut Planner<'schema>,
        query_path: &'a QueryPath,
        maybe_parent: Option<BoundaryParent<'schema, 'a>>,
        missing: FlatSelectionSet,
    ) -> PlanningResult<Vec<PlanId>> {
        if let Some(BoundaryParent { logic, providable }) = maybe_parent {
            let mut boundary_fields = providable
                .into_iter()
                .map(|(field_id, group)| (field_id, BoundaryField::new(logic.plan_id(), group)))
                .collect();
            Self {
                planner,
                query_path,
                maybe_parent: Some(logic),
                children: Vec::new(),
            }
            .plan_selection_set(&mut boundary_fields, missing)
        } else {
            Self {
                planner,
                query_path,
                maybe_parent: None,
                children: Vec::new(),
            }
            .plan_selection_set(&mut BoundaryFields::default(), missing)
        }
    }
}

pub(super) struct BoundaryParent<'schema, 'a> {
    pub logic: &'a AttributionLogic<'schema>,
    pub providable: FnvHashMap<FieldId, GroupedByFieldId>,
}

impl<'schema, 'a> std::ops::Deref for BoundaryPlanner<'schema, 'a> {
    type Target = Planner<'schema>;
    fn deref(&self) -> &Self::Target {
        self.planner
    }
}

impl<'schema, 'a> std::ops::DerefMut for BoundaryPlanner<'schema, 'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.planner
    }
}

type BoundaryFields = FnvHashMap<FieldId, BoundaryField>;

#[derive(Debug)]
struct BoundaryField {
    plan_id: PlanId,
    group: GroupedByFieldId,
    lazy_subselection: Option<BoundaryFields>,
}

impl BoundaryField {
    pub(super) fn new(plan_id: PlanId, group: GroupedByFieldId) -> Self {
        Self {
            plan_id,
            group,
            lazy_subselection: None,
        }
    }
}

/// Potential child plan, but might not be the best one.
struct ChildPlanCandidate<'schema> {
    resolver_id: ResolverId,
    /// Entity type (object/interface id) of the fields
    entity_type: EntityType,
    /// Providable fields by the resolvers with their requirements
    providable_fields: Vec<(BoundFieldId, &'schema FieldSet)>,
}

struct MissingField<'schema> {
    entity_type: EntityType,
    flat_field: FlatField,
    field_resolvers: Vec<FieldResolverWalker<'schema>>,
}

impl<'schema> std::ops::Deref for MissingField<'schema> {
    type Target = FlatField;
    fn deref(&self) -> &Self::Target {
        &self.flat_field
    }
}

impl<'schema> From<MissingField<'schema>> for FlatField {
    fn from(missing: MissingField<'schema>) -> Self {
        missing.flat_field
    }
}

impl<'schema, 'a> BoundaryPlanner<'schema, 'a> {
    fn plan_selection_set(
        mut self,
        boundary_fields: &mut BoundaryFields,
        mut selection_set: FlatSelectionSet,
    ) -> PlanningResult<Vec<PlanId>> {
        // Fields that couldn't be provided by the parent and that have yet to be planned by one
        // child plan.
        let mut id_to_missing_fields: FnvHashMap<BoundFieldId, MissingField<'schema>> =
            self.build_missing_fields(std::mem::replace(&mut selection_set.fields, Vec::with_capacity(0)));

        // Actual planning, we plan one child plan at a time.
        let mut candidates: FnvHashMap<ResolverId, ChildPlanCandidate<'schema>> = FnvHashMap::default();
        while !id_to_missing_fields.is_empty() {
            candidates.clear();
            self.generate_all_candidates(id_to_missing_fields.values(), boundary_fields, &mut candidates);

            let Some(candidate) = select_best_child_plan(&mut candidates) else {
                let walker = self.walker();
                return Err(PlanningError::CouldNotPlanAnyField {
                    missing: id_to_missing_fields
                        .into_keys()
                        .map(|id| walker.walk(id).response_key_str().to_string())
                        .collect(),
                    query_path: walker.walk(self.query_path).iter().map(|s| s.to_string()).collect(),
                });
            };

            let mut requires = self.schema.walk(candidate.resolver_id).requires();
            let mut output = vec![];
            for (id, field_requires) in std::mem::take(&mut candidate.providable_fields) {
                let flat_field = FlatField::from(id_to_missing_fields.remove(&id).unwrap());
                if !field_requires.is_empty() {
                    requires = Cow::Owned(FieldSet::merge(&requires, field_requires));
                }
                output.push(flat_field);
            }
            let output = selection_set.clone_with_fields(output);
            self.push_child(candidate, requires, output, boundary_fields)?;
        }

        Ok(self.children)
    }

    fn push_child(
        &mut self,
        candidate: &mut ChildPlanCandidate<'schema>,
        requires: Cow<'_, FieldSet>,
        selection_set: FlatSelectionSet,
        boundary_fields: &mut BoundaryFields,
    ) -> PlanningResult<()> {
        let grouped_by_schema_field_id = self.walker().group_by_schema_field_id(&selection_set);
        let path = self.query_path.clone();
        let plan_id = self.push_plan(path, candidate.resolver_id, candidate.entity_type, selection_set)?;
        if !requires.is_empty() {
            let resolver = self.schema.walker().walk(candidate.resolver_id).with_own_names();
            let input_selection_set = self.create_input_selection_set(plan_id, &resolver, &requires, boundary_fields);
            self.insert_plan_input_selection_set(plan_id, input_selection_set);
        };
        for (field_id, group) in grouped_by_schema_field_id {
            boundary_fields
                .entry(field_id)
                .or_insert_with(|| BoundaryField::new(plan_id, group));
        }

        self.children.push(plan_id);
        Ok(())
    }

    /// Create the the input selection set of a Plan given its resolver and requirements.
    /// We iterate over the requirements and find the matching fields inside the boundary fields,
    /// which contains all providable & extra fields. During the iteration we track all the dependency
    /// plans.
    fn create_input_selection_set(
        &mut self,
        plan_id: PlanId,
        resolver: &ResolverWalker<'_>,
        requires: &FieldSet,
        boundary_fields: &BoundaryFields,
    ) -> ReadSelectionSet {
        if requires.is_empty() {
            return ReadSelectionSet::default();
        }
        requires
            .iter()
            .map(|item| {
                let boundary_field = boundary_fields
                    .get(&item.field_id)
                    .expect("field should be present, we could plan it");
                self.operation[boundary_field.group.final_bound_field_id].mark_as_read();
                let parent = self
                    .get_field_plan(boundary_field.group.final_bound_field_id)
                    .expect("Plan for a field we required should have been determined");
                self.insert_plan_dependency(ParentToChildEdge { parent, child: plan_id });
                ReadField {
                    edge: boundary_field.group.edge,
                    name: resolver.walk(item.field_id).name().to_string(),
                    subselection: if item.subselection.is_empty() {
                        ReadSelectionSet::default()
                    } else {
                        let subselection = boundary_field
                            .lazy_subselection
                            .as_ref()
                            .expect("subselection should be present, we could plan the subselection");
                        self.create_input_selection_set(plan_id, resolver, &item.subselection, subselection)
                    },
                }
            })
            .collect()
    }

    fn build_missing_fields(&self, fields: Vec<FlatField>) -> FnvHashMap<BoundFieldId, MissingField<'schema>> {
        let walker = self.schema.walker();
        let mut id_to_missing_fields = FnvHashMap::default();
        for field in fields {
            let entity_type = match self.operation[field.parent_selection_set_id()].ty {
                SelectionSetType::Object(id) => EntityType::Object(id),
                SelectionSetType::Interface(id) => EntityType::Interface(id),
                SelectionSetType::Union(_) => unreachable!("Unions have no fields."),
            };
            let field_id = self.operation[field.bound_field_id]
                .schema_field_id()
                .expect("Meta fields are always providable, it can't be missing.");
            let field_resolvers = walker.walk(field_id).resolvers().collect::<Vec<_>>();
            id_to_missing_fields.insert(
                field.bound_field_id,
                MissingField {
                    entity_type,
                    flat_field: field,
                    field_resolvers,
                },
            );
        }
        id_to_missing_fields
    }

    fn generate_all_candidates<'field>(
        &mut self,
        missing_fields: impl IntoIterator<Item = &'field MissingField<'schema>>,
        boundary_fields: &mut BoundaryFields,
        candidates: &mut FnvHashMap<ResolverId, ChildPlanCandidate<'schema>>,
    ) where
        'schema: 'field,
    {
        for field in missing_fields {
            for FieldResolverWalker {
                resolver,
                field_requires,
            } in &field.field_resolvers
            {
                match candidates.entry(resolver.id()) {
                    Entry::Occupied(mut entry) => {
                        let candidate = entry.get_mut();
                        if self.could_plan_requirements(boundary_fields, field.bound_field_id, field_requires) {
                            candidate.providable_fields.push((field.bound_field_id, field_requires));
                        }
                    }
                    Entry::Vacant(entry) => {
                        if self.could_plan_requirements(boundary_fields, field.bound_field_id, &resolver.requires())
                            && self.could_plan_requirements(boundary_fields, field.bound_field_id, field_requires)
                        {
                            entry.insert(ChildPlanCandidate {
                                entity_type: field.entity_type,
                                resolver_id: resolver.id(),
                                providable_fields: vec![(field.bound_field_id, field_requires)],
                            });
                        }
                    }
                }
            }
        }
    }

    /// Allows us to know whether a field requirements can be provided at all to order the next child
    /// candidates.
    fn could_plan_requirements(
        &mut self,
        boundary_fields: &mut BoundaryFields,
        origin_bound_field_id: BoundFieldId,
        requires: &FieldSet,
    ) -> bool {
        if requires.is_empty() {
            return true;
        }
        let parent_field_plan_id = self
            .maybe_parent
            .expect("Cannot have requirements without a parent plan")
            .plan_id();
        self.could_plan_requirements_on_previous_plans(
            parent_field_plan_id,
            boundary_fields,
            origin_bound_field_id,
            requires,
        )
    }

    fn could_plan_requirements_on_previous_plans(
        &mut self,
        parent_field_plan_id: PlanId,
        boundary_fields: &mut BoundaryFields,
        origin_bound_field_id: BoundFieldId,
        requires: &FieldSet,
    ) -> bool {
        if requires.is_empty() {
            return true;
        }
        let parent_selection_set_id = self.operation.parent_selection_set_id(origin_bound_field_id);
        'requires: for item in requires {
            // -- Existing fields --
            if let Some(boundary_field) = boundary_fields.get_mut(&item.field_id) {
                if item.subselection.is_empty() {
                    continue;
                }
                if boundary_field.lazy_subselection.is_none() {
                    let walker = self.walker();
                    let flat_selection_set = walker
                        .flatten_subselection_sets(&boundary_field.group.bound_field_ids)
                        .expect("Requirements expect a selection set");
                    let fields = walker
                        .group_by_schema_field_id(&flat_selection_set)
                        .into_iter()
                        .map(|(field_id, group)| {
                            let plan_id = self
                                .get_field_plan(group.final_bound_field_id)
                                .expect("Nested fields should already have been planned");
                            let boundary_field = BoundaryField::new(plan_id, group);
                            (field_id, boundary_field)
                        })
                        .collect();
                    boundary_field.lazy_subselection = Some(fields)
                }
                if self.could_plan_requirements_on_previous_plans(
                    boundary_field.plan_id,
                    boundary_field.lazy_subselection.as_mut().unwrap(),
                    boundary_field.group.final_bound_field_id,
                    &item.subselection,
                ) {
                    continue;
                } else {
                    return false;
                }
            }

            // -- Plannable by the parent --
            let field = self.schema.walker().walk(item.field_id);
            let parent_logic = self
                .maybe_parent
                .expect("Cannot have requirements without a parent plan");
            // no need to check for requires here, they're only relevant when it's a
            // plan root field and this is a nested field. So we expect the data source
            // to be able to provide anything it needed for a nested object it provides.
            if parent_logic.plan_id() == parent_field_plan_id && parent_logic.is_providable(field.id()) {
                if let Some(boundary_field) =
                    self.try_planning_boundary_field(parent_logic, parent_selection_set_id, item)
                {
                    boundary_fields.insert(item.field_id, boundary_field);
                    continue;
                }
            }

            // -- Plannable by existing children --
            for i in 0..self.children.len() {
                let plan_id = self.children[i];
                // ensures we don't have cycles between plans ensuring they can only depend on
                // plan_ids lower than theirs. Could be better.
                if plan_id <= parent_field_plan_id {
                    continue;
                }
                let resolver_id = self.get_plan(plan_id).resolver_id;
                for FieldResolverWalker {
                    resolver,
                    field_requires,
                } in field.resolvers()
                {
                    if resolver.id() != resolver_id
                        && self.could_plan_requirements_on_previous_plans(
                            plan_id,
                            boundary_fields,
                            origin_bound_field_id,
                            field_requires,
                        )
                    {
                        let logic = &AttributionLogic::CompatibleResolver {
                            plan_id,
                            resolver,
                            providable: field
                                .provides_for(&resolver)
                                .map(|field_set| field_set.into_owned())
                                .unwrap_or_default(),
                        };
                        if let Some(boundary_field) =
                            self.try_planning_boundary_field(logic, parent_selection_set_id, item)
                        {
                            boundary_fields.insert(item.field_id, boundary_field);
                            continue 'requires;
                        }
                    }
                }
            }

            // -- Add new child plan --
            // eventually?

            // -- Not plannable --
            return false;
        }

        true
    }

    fn try_planning_boundary_field(
        &mut self,
        logic: &AttributionLogic<'schema>,
        parent_selection_set_id: BoundSelectionSetId,
        item: &FieldSetItem,
    ) -> Option<BoundaryField> {
        self.try_planning_extra_fields_with_subselection(logic, Some(parent_selection_set_id), item)
            .map(|bound_field_id| {
                BoundaryField::new(
                    logic.plan_id(),
                    GroupedByFieldId {
                        edge: self.operation[bound_field_id].response_edge(),
                        bound_field_ids: vec![bound_field_id],
                        final_bound_field_id: bound_field_id,
                        subselection_set_ids: self.operation[bound_field_id].selection_set_id().into_iter().collect(),
                    },
                )
            })
    }

    fn try_planning_extra_fields_with_subselection(
        &mut self,
        logic: &AttributionLogic<'schema>,
        parent_selection_set_id: Option<BoundSelectionSetId>,
        item: &FieldSetItem,
    ) -> Option<BoundFieldId> {
        // We don't
        if !logic.is_providable(item.field_id) {
            return None;
        }
        let field = logic.resolver().walk(item.field_id);
        let selection_set_id = if let Some(ty) = SelectionSetType::maybe_from(field.ty().inner().id()) {
            let logic = logic.child(field.id());
            for _item in &item.subselection {
                // Not need to check field requirements, it's nested a field, so the resolver is
                // expected to provide anything it needs.
                if !logic.is_providable(field.id()) {
                    return None;
                }
            }
            let selection_set = BoundSelectionSet {
                ty,
                items: item
                    .subselection
                    .iter()
                    .map(|item| {
                        self.try_planning_extra_fields_with_subselection(&logic, None, item)
                            .map(BoundSelection::Field)
                    })
                    .collect::<Option<Vec<_>>>()?,
            };
            Some(self.push_extra_selection_set(logic.plan_id(), selection_set))
        } else {
            None
        };
        tracing::debug!(
            "Adding extra field {} provided by {}",
            self.schema.walker().walk(item.field_id).name(),
            logic.plan_id()
        );
        let bound_field = BoundField::Extra {
            edge: self.generate_unique_edge_for(item.field_id),
            field_id: item.field_id,
            selection_set_id,
            read: false,
        };
        Some(self.push_extra_field(logic.plan_id(), parent_selection_set_id, bound_field))
    }
}

fn select_best_child_plan<'c, 'op>(
    candidates: &'c mut FnvHashMap<ResolverId, ChildPlanCandidate<'op>>,
) -> Option<&'c mut ChildPlanCandidate<'op>> {
    // We could be smarter, but we need to be sure there is no intersection between
    // candidates (which impacts ordering among other things) and some fields may now be
    // available (requires can now be provided) after planning this candidate. So the easy
    // solution is to regenerate candidates after each plan.
    candidates
        .values_mut()
        .filter(|candidate| !candidate.providable_fields.is_empty())
        .max_by_key(|candidate| candidate.providable_fields.len())
}
