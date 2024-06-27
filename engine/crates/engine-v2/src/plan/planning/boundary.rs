use std::{
    borrow::Cow,
    collections::{hash_map::Entry, BTreeMap, HashMap},
};

use id_newtypes::IdRange;
use itertools::Itertools;
use schema::{FieldDefinitionId, RequiredFieldId, RequiredFieldSet, RequiredFieldSetItemWalker, ResolverId};
use tracing::{instrument, Level};

use super::{logic::PlanningLogic, planner::Planner, PlanningError, PlanningResult};
use crate::{
    operation::{
        Field, FieldArgument, FieldArgumentId, FieldId, ParentToChildEdge, PlanBoundaryId, PlanId, QueryInputValue,
        QueryPath, Selection, SelectionSet, SelectionSetId, SelectionSetType,
    },
    plan::{flatten_selection_sets, EntityId, FlatField, FlatSelectionSet},
    response::{ResponseKey, SafeResponseKey, UnpackedResponseEdge},
};

/// The Planner traverses the selection sets to plan all the fields, but it doesn't define the
/// plans directly. That's the job of the BoundaryPlanner which will attribute a plan for each
/// field for a given selection set and satisfy any requirements.
pub(super) struct BoundarySelectionSetPlanner<'schema, 'a> {
    planner: &'a mut Planner<'schema>,
    query_path: &'a QueryPath,
    maybe_parent: Option<&'a PlanningLogic<'schema>>,
    plan_boundary_id: PlanBoundaryId,
    required_field_id_to_field_id: HashMap<RequiredFieldId, FieldId>,
    children: Vec<PlanId>,
    extra_response_key_suffix: usize,
}

impl<'schema, 'a> BoundarySelectionSetPlanner<'schema, 'a> {
    #[instrument(
        level = Level::DEBUG,
        skip_all,
        fields(parent = %maybe_parent.as_ref().map(|p| p.to_string()).unwrap_or_default(),
               path = %planner.walker().walk(query_path))
    )]
    pub(super) fn plan(
        planner: &'a mut Planner<'schema>,
        query_path: &'a QueryPath,
        maybe_parent: Option<&'a PlanningLogic<'schema>>,
        providable: FlatSelectionSet,
        unplanned: FlatSelectionSet,
    ) -> PlanningResult<()> {
        let mut boundary_planner = Self {
            plan_boundary_id: planner.next_plan_boundary_id(),
            planner,
            query_path,
            maybe_parent,
            children: Vec::new(),
            required_field_id_to_field_id: HashMap::default(),
            extra_response_key_suffix: 0,
        };
        let mut boundary_fields = boundary_planner.group_fields(providable);
        boundary_planner.plan_selection_set(&mut boundary_fields, unplanned)?;

        let Self {
            planner,
            required_field_id_to_field_id,
            plan_boundary_id,
            ..
        } = boundary_planner;
        planner
            .operation
            .field_dependencies
            .extend(
                required_field_id_to_field_id
                    .into_iter()
                    .map(|(required_field_id, field_id)| crate::operation::FieldDependency {
                        plan_boundary_id,
                        required_field_id,
                        field_id,
                    }),
            );
        Ok(())
    }

    fn group_subselection_fields(&self, field_ids: &[FieldId]) -> GroupedProvidableFields {
        let subselection_set_ids = field_ids
            .iter()
            .filter_map(|id| self.operation[*id].selection_set_id())
            .collect();
        let flat_selection_set = flatten_selection_sets(self.schema, &self.operation, subselection_set_ids);
        self.group_fields(flat_selection_set)
    }

    fn group_fields(&self, providable: FlatSelectionSet) -> GroupedProvidableFields {
        self.group_by_definition_id_then_response_key_sorted_by_query_position(
            providable.into_iter().map(|field| field.id),
        )
    }

    fn group_by_definition_id_then_response_key_sorted_by_query_position(
        &self,
        values: impl IntoIterator<Item = FieldId>,
    ) -> GroupedProvidableFields {
        let mut grouped: GroupedProvidableFields = values.into_iter().fold(Default::default(), |mut groups, id| {
            let field = &self.operation[id];
            if let Some(definition_id) = field.definition_id() {
                groups
                    .entry(definition_id)
                    .or_default()
                    .entry(field.response_key())
                    .and_modify(|group| group.field_ids.push(id))
                    .or_insert_with(|| {
                        // At this stage we're generating boundary fields for an existing selection set which
                        // was already planned. By construction, as soon as we create a new plan with
                        // push_plan() it plans all of the nested selection sets.
                        // And for extra fields we add during planning, those are attributed immediately.
                        let plan_id = self.get_field_plan(id).expect("field should be planned");

                        GroupedByDefinitionThenResponseKey::new(plan_id, vec![id])
                    });
            }
            groups
        });
        for group in grouped.values_mut().flat_map(|groups| groups.values_mut()) {
            group
                .field_ids
                .sort_unstable_by_key(|id| self.operation[*id].query_position())
        }
        grouped
    }
}

impl<'schema, 'a> std::ops::Deref for BoundarySelectionSetPlanner<'schema, 'a> {
    type Target = Planner<'schema>;
    fn deref(&self) -> &Self::Target {
        self.planner
    }
}

impl<'schema, 'a> std::ops::DerefMut for BoundarySelectionSetPlanner<'schema, 'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.planner
    }
}

/// During the planning of the boundary we need to keep track of fields by their FieldId
/// to satisfy requirements. The goal is not only to know what's present but also to have the
/// correct ResponseEdge for those when reading data from the response later.
type GroupedProvidableFields = HashMap<FieldDefinitionId, BTreeMap<ResponseKey, GroupedByDefinitionThenResponseKey>>;

#[derive(Debug)]
struct GroupedByDefinitionThenResponseKey {
    plan_id: PlanId,
    field_ids: Vec<FieldId>,
    lazy_subselection: Option<GroupedProvidableFields>,
}

impl GroupedByDefinitionThenResponseKey {
    fn new(plan_id: PlanId, field_ids: Vec<FieldId>) -> Self {
        Self {
            plan_id,
            field_ids,
            lazy_subselection: None,
        }
    }
}

/// Potential child plan, but might not be the best one.
struct ChildPlanCandidate<'schema> {
    resolver_id: ResolverId,
    /// Entity type (object/interface id) of the fields
    entity_type: EntityId,
    /// Providable fields by the resolvers with their requirements
    providable_fields: Vec<(FieldId, &'schema RequiredFieldSet)>,
}

/// Field that the parent plan could not providable.
struct UnplannedField {
    entity_type: EntityId,
    flat_field: FlatField,
    definition_id: FieldDefinitionId,
}

impl std::ops::Deref for UnplannedField {
    type Target = FlatField;
    fn deref(&self) -> &Self::Target {
        &self.flat_field
    }
}

impl From<UnplannedField> for FlatField {
    fn from(unplanned: UnplannedField) -> Self {
        unplanned.flat_field
    }
}

impl<'schema, 'a> BoundarySelectionSetPlanner<'schema, 'a> {
    /// Iteratively plan fields.
    /// 1. Generate all potential plan candidates satisfying their requirements if possible.
    /// 2. Select the best candidate, generate its input and remove its output fields from the
    ///    unplanned ones.
    /// 3. Continue until there are no more unplanned fields.
    fn plan_selection_set(
        &mut self,
        grouped_fields: &mut GroupedProvidableFields,
        mut unplanned_selection_set: FlatSelectionSet,
    ) -> PlanningResult<()> {
        // Fields that couldn't be provided by the parent and that have yet to be planned by one
        // child plan.
        let mut id_to_unplanned_fields: HashMap<FieldId, UnplannedField> =
            self.build_unplanned_fields(std::mem::take(&mut unplanned_selection_set.fields));

        // unplanned_field may be still be provided by the parent plan, but at this stage it
        // means they had requirements which weren't sure whether we could provide them.
        if let Some(parent_logic) = self.maybe_parent {
            let mut planned = Vec::new();
            for unplanned_field in id_to_unplanned_fields.values() {
                let definition = self.schema.walk(unplanned_field.definition_id);

                // If the parent plan can provide the field, we don't need to plan it.
                if parent_logic.is_providable(unplanned_field.definition_id)
                    && self.could_plan_requirements(
                        grouped_fields,
                        unplanned_field.id,
                        definition.requires(parent_logic.resolver().subgraph_id()),
                    )?
                {
                    planned.push(unplanned_field.id);
                    continue;
                }
            }

            let mut requires = RequiredFieldSet::default();
            let mut fields = vec![];
            for id in planned {
                let unplanned_field = id_to_unplanned_fields.remove(&id).unwrap();
                requires = requires.union(
                    self.schema
                        .walk(unplanned_field.definition_id)
                        .requires(parent_logic.resolver().subgraph_id()),
                );
                fields.push(FlatField::from(unplanned_field));
            }

            self.planner.plan_obviously_providable_subselections(
                self.query_path,
                parent_logic,
                &unplanned_selection_set.clone_with_fields(fields),
            )?;
            self.push_plan_requires_dependencies(parent_logic.plan_id(), &requires);
        }

        if id_to_unplanned_fields.is_empty() {
            return Ok(());
        }

        // Actual planning, we plan one child plan at a time.
        let mut candidates: HashMap<ResolverId, ChildPlanCandidate<'schema>> = HashMap::default();
        while !id_to_unplanned_fields.is_empty() {
            candidates.clear();
            self.generate_all_candidates(id_to_unplanned_fields.values(), grouped_fields, &mut candidates)?;

            let Some(candidate) = select_best_child_plan(&mut candidates) else {
                let walker = self.walker();
                tracing::debug!(
                    "Could not plan fields:\n=== PARENT ===\n{:#?}\n=== CURRENT ===\n{}\n=== MISSING ===\n{}",
                    self.maybe_parent.map(|parent| parent.resolver()),
                    grouped_fields
                        .keys()
                        .map(|id| self.schema.walk(*id))
                        .format_with("\n", |field, f| f(&format_args!("{field:#?}"))),
                    id_to_unplanned_fields
                        .keys()
                        .map(|id| walker.walk(*id).definition().unwrap())
                        .format_with("\n", |field, f| f(&format_args!("{field:#?}")))
                );
                return Err(PlanningError::CouldNotPlanAnyField {
                    missing: id_to_unplanned_fields
                        .into_keys()
                        .map(|id| walker.walk(id).response_key_str().to_string())
                        .collect(),
                    query_path: walker.walk(self.query_path).iter().map(|s| s.to_string()).collect(),
                });
            };

            let mut requires = Cow::Borrowed(self.schema.walk(candidate.resolver_id).requires());
            let mut output = vec![];
            for (id, field_requires) in std::mem::take(&mut candidate.providable_fields) {
                let flat_field = FlatField::from(id_to_unplanned_fields.remove(&id).unwrap());
                if !field_requires.is_empty() {
                    requires = Cow::Owned(requires.union(field_requires));
                }
                output.push(flat_field);
            }
            let output = unplanned_selection_set.clone_with_fields(output);
            self.push_child(candidate, requires, output, grouped_fields)?;
        }

        Ok(())
    }

    fn push_child(
        &mut self,
        candidate: &mut ChildPlanCandidate<'schema>,
        requires: Cow<'_, RequiredFieldSet>,
        providable: FlatSelectionSet,
        grouped_fields: &mut GroupedProvidableFields,
    ) -> PlanningResult<()> {
        let path = self.query_path.clone();
        let plan_id = self.push_plan(path, candidate.resolver_id, candidate.entity_type, &providable)?;
        self.push_plan_requires_dependencies(plan_id, &requires);
        for (definition_id, groups) in self.group_fields(providable) {
            grouped_fields.insert(definition_id, groups);
        }

        self.children.push(plan_id);
        Ok(())
    }

    /// Create the input selection set of a Plan given its resolver and requirements.
    /// We iterate over the requirements and find the matching fields inside the boundary fields,
    /// which contains all providable & extra fields. During the iteration we track all the dependency
    /// plans.
    fn push_plan_requires_dependencies(&mut self, plan_id: PlanId, requires: &RequiredFieldSet) {
        for required_field in requires {
            let field_id = self.required_field_id_to_field_id[&required_field.id];
            // We add a bunch of fields during the planning to the operation when trying to
            // satisfy requirements. But only those marked as read will be retrieved.
            self.operation[field_id].mark_as_read();
            let parent_plan_id = self.get_field_plan(field_id).expect("field should be planned");
            let edge = ParentToChildEdge {
                parent: parent_plan_id,
                child: plan_id,
                boundary: self.plan_boundary_id,
            };
            self.push_plan_dependency(edge);
            self.push_plan_requires_dependencies(plan_id, &required_field.subselection)
        }
    }

    fn build_unplanned_fields(&self, flat_fields: Vec<FlatField>) -> HashMap<FieldId, UnplannedField> {
        let mut id_to_unplanned_fields = HashMap::default();
        for flat_field in flat_fields {
            let entity_type = match self.operation[flat_field.parent_selection_set_id()].ty {
                SelectionSetType::Object(id) => EntityId::Object(id),
                SelectionSetType::Interface(id) => EntityId::Interface(id),
                SelectionSetType::Union(_) => unreachable!("Unions have no fields."),
            };
            let definition_id = self.operation[flat_field.id]
                .definition_id()
                .expect("Meta fields are always providable, it can't be missing.");
            id_to_unplanned_fields.insert(
                flat_field.id,
                UnplannedField {
                    entity_type,
                    flat_field,
                    definition_id,
                },
            );
        }
        id_to_unplanned_fields
    }

    fn generate_all_candidates<'field>(
        &mut self,
        unplanned_fields: impl IntoIterator<Item = &'field UnplannedField>,
        grouped_fields: &mut GroupedProvidableFields,
        candidates: &mut HashMap<ResolverId, ChildPlanCandidate<'schema>>,
    ) -> PlanningResult<()>
    where
        'schema: 'field,
    {
        for unplanned_field in unplanned_fields {
            let definition = self.schema.walk(unplanned_field.definition_id);
            for resolver in definition.resolvers() {
                tracing::trace!("Trying to plan '{}' with: {}", definition.name(), resolver.name());
                let field_requires = definition.requires(resolver.subgraph_id());
                match candidates.entry(resolver.id()) {
                    Entry::Occupied(mut entry) => {
                        let candidate = entry.get_mut();
                        if self.could_plan_requirements(grouped_fields, unplanned_field.id, field_requires)? {
                            candidate.providable_fields.push((unplanned_field.id, field_requires));
                        }
                    }
                    Entry::Vacant(entry) => {
                        if self.could_plan_requirements(grouped_fields, unplanned_field.id, resolver.requires())?
                            && self.could_plan_requirements(grouped_fields, unplanned_field.id, field_requires)?
                        {
                            entry.insert(ChildPlanCandidate {
                                entity_type: unplanned_field.entity_type,
                                resolver_id: resolver.id(),
                                providable_fields: vec![(unplanned_field.id, field_requires)],
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Allows us to know whether a field requirements can be provided at all to order the next child
    /// candidates.
    fn could_plan_requirements(
        &mut self,
        grouped_fields: &mut GroupedProvidableFields,
        petitioner_field_id: FieldId,
        requires: &'schema RequiredFieldSet,
    ) -> PlanningResult<bool> {
        if requires.is_empty() {
            return Ok(true);
        }
        let parent_field_plan_id = self
            .maybe_parent
            .expect("Cannot have requirements without a parent plan")
            .plan_id();
        self.could_plan_requirements_on_previous_plans(
            parent_field_plan_id,
            grouped_fields,
            petitioner_field_id,
            requires,
        )
    }

    fn could_plan_requirements_on_previous_plans(
        &mut self,
        parent_field_plan_id: PlanId,
        grouped_fields: &mut GroupedProvidableFields,
        petitioner_field_id: FieldId,
        requires: &'schema RequiredFieldSet,
    ) -> PlanningResult<bool> {
        if requires.is_empty() {
            return Ok(true);
        }
        'requires: for required in requires {
            let required_field = &self.schema[required.id];

            // -- Existing fields --
            if let Some(groups) = grouped_fields.get_mut(&required_field.definition_id) {
                for group in groups.values_mut() {
                    // TODO: we should likely validate explicitly that all fields for the same response key have
                    // the same arguments. The GraphQL spec doesn't mention it, but during
                    // in ExecuteField it just takes the first field and uses its arguments.
                    // https://spec.graphql.org/October2021/#ExecuteField()
                    let field_id = group.field_ids[0];

                    // If argument don't match, trying another group
                    if !self.walker().walk(field_id).eq(required_field) {
                        continue;
                    }

                    self.required_field_id_to_field_id.insert(required.id, field_id);

                    // If there is no require sub-selection, we already have everything we need.
                    if required.subselection.is_empty() {
                        continue 'requires;
                    }

                    if group.lazy_subselection.is_none() {
                        group.lazy_subselection = Some(self.group_subselection_fields(&group.field_ids));
                    }

                    // Now we only need to know whether we can plan the field, We don't bother with
                    // other groups. I'm not sure whether having response key groups with different
                    // plan ids for the same FieldDefinitionId would ever happen.
                    // So either we can plan the necessary requirements with this group or we
                    // don't.
                    if self.could_plan_requirements_on_previous_plans(
                        group.plan_id,
                        group.lazy_subselection.as_mut().unwrap(),
                        field_id,
                        &required.subselection,
                    )? {
                        continue 'requires;
                    } else {
                        return Ok(false);
                    }
                }
            }

            let required = self.schema.walk(required);

            // -- Plannable by the parent --
            let parent_logic = self
                .maybe_parent
                .expect("Cannot have requirements without a parent plan");
            // the parent field does come from the parent plan
            if parent_logic.plan_id() == parent_field_plan_id
                && self.could_plan_exra_field(grouped_fields, petitioner_field_id, parent_logic, required)
            {
                continue;
            }

            // -- Plannable by existing children --
            for i in 0..self.children.len() {
                let plan_id = self.children[i];
                // ensures we don't have cycles between plans ensuring they can only depend on
                // plan_ids lower than theirs. Could be better.
                if plan_id < parent_field_plan_id {
                    continue;
                }
                if self.could_plan_exra_field(
                    grouped_fields,
                    petitioner_field_id,
                    &PlanningLogic::new(plan_id, self.schema.walk(self.planner.operation[plan_id].resolver_id)),
                    required,
                ) {
                    continue 'requires;
                }
            }

            // -- Not plannable --
            return Ok(false);
        }

        Ok(true)
    }

    fn could_plan_exra_field(
        &mut self,
        grouped_fields: &mut GroupedProvidableFields,
        petitioner_field_id: FieldId,
        logic: &PlanningLogic<'schema>,
        required: RequiredFieldSetItemWalker<'schema>,
    ) -> bool {
        let parent_selection_set_id = self.operation.parent_selection_set_id(petitioner_field_id);
        let Some(field_id) =
            self.try_plan_extra_field(petitioner_field_id, logic, Some(parent_selection_set_id), required)
        else {
            return false;
        };
        self.required_field_id_to_field_id
            .insert(required.required_field_id(), field_id);
        let field = &self.operation[field_id];
        grouped_fields.entry(required.definition().id()).or_default().insert(
            field.response_key(),
            GroupedByDefinitionThenResponseKey {
                plan_id: logic.plan_id(),
                field_ids: vec![field_id],
                lazy_subselection: None,
            },
        );
        true
    }

    fn try_plan_extra_field(
        &mut self,
        petitioner_field_id: FieldId,
        logic: &PlanningLogic<'schema>,
        parent_selection_set_id: Option<SelectionSetId>,
        required: RequiredFieldSetItemWalker<'schema>,
    ) -> Option<FieldId> {
        if !logic.is_providable(required.definition().id()) {
            return None;
        }
        let field = required.definition();
        let selection_set_id = if let Some(ty) = SelectionSetType::maybe_from(field.ty().inner().id()) {
            let logic = logic.child(field.id());
            if required
                .subselection()
                .any(|nested| !logic.is_providable(nested.definition().id()))
            {
                return None;
            }
            let selection_set = SelectionSet {
                ty,
                items: required
                    .subselection()
                    .map(|nested| {
                        self.try_plan_extra_field(petitioner_field_id, &logic, None, nested)
                            .map(Selection::Field)
                    })
                    .collect::<Option<Vec<_>>>()?,
            };
            Some(self.push_extra_selection_set(logic.plan_id(), selection_set))
        } else {
            None
        };
        tracing::trace!(
            "Adding extra field '{}' provided by {} required by '{}'",
            self.schema.walker().walk(required.definition().id()).name(),
            logic.plan_id(),
            self.walker().walk(petitioner_field_id).response_key_str()
        );
        let key = self.generate_response_key_for(required.definition().id());
        let field = Field::Extra {
            edge: UnpackedResponseEdge::ExtraFieldResponseKey(key.into()).pack(),
            field_definition_id: required.definition().id(),
            selection_set_id,
            argument_ids: self.create_arguments_for(required.required_field_id()),
            petitioner_location: self.operation[petitioner_field_id].location(),
            is_read: true,
        };
        Some(self.push_extra_field(logic.plan_id(), parent_selection_set_id, field))
    }

    fn generate_response_key_for(&mut self, field_id: FieldDefinitionId) -> SafeResponseKey {
        // Try just using the field name
        let name = self.schema.walker().walk(field_id).name();
        let response_keys = &mut self.operation.response_keys;
        if !response_keys.contains(name) {
            return response_keys.get_or_intern(name);
        }

        // Generate a likely unique key
        let short_id = hex::encode(u32::from(field_id).to_be_bytes())
            .trim_start_matches('0')
            .to_uppercase();
        let name = format!("_{}{}", name, short_id);
        if !response_keys.contains(&name) {
            return response_keys.get_or_intern(&name);
        }

        // Previous key may still not be enough if we need multiple times the same field with
        // different arguments for example.
        loop {
            let candidate = format!("{name}{}", self.extra_response_key_suffix);
            if !self.operation.response_keys.contains(&candidate) {
                return self.operation.response_keys.get_or_intern(&candidate);
            }
            self.extra_response_key_suffix += 1;
        }
    }

    fn create_arguments_for(&mut self, id: RequiredFieldId) -> IdRange<FieldArgumentId> {
        let start = self.operation.field_arguments.len();
        for &(input_value_definition_id, value_id) in &self.schema[id].arguments {
            let input_value_id = self
                .operation
                .query_input_values
                .push_value(QueryInputValue::DefaultValue(value_id));
            self.operation.field_arguments.push(FieldArgument {
                name_location: None,
                value_location: None,
                input_value_id,
                input_value_definition_id,
            });
        }
        let end = self.operation.field_arguments.len();
        (start..end).into()
    }
}

fn select_best_child_plan<'c, 'op>(
    candidates: &'c mut HashMap<ResolverId, ChildPlanCandidate<'op>>,
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
