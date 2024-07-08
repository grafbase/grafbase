use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
};

use id_newtypes::IdRange;
use itertools::Itertools;
use schema::{
    FieldDefinitionId, FieldDefinitionWalker, RequiredFieldId, RequiredFieldSet, RequiredFieldSetItemWalker, ResolverId,
};
use tracing::{instrument, Level};

use super::{logic::PlanningLogic, planner::OperationSolver, PlanningError, PlanningResult};
use crate::{
    operation::{
        ExtraField, Field, FieldArgument, FieldArgumentId, FieldId, ParentToChildEdge, PlanId, QueryInputValue,
        QueryPath, SelectionSet, SelectionSetId, SolvedRequiredField, SolvedRequiredFieldSet,
    },
    response::{SafeResponseKey, UnpackedResponseEdge},
};

/// The Planner traverses the selection sets to plan all the fields, but it doesn't define the
/// plans directly. That's the job of the BoundaryPlanner which will attribute a plan for each
/// field for a given selection set and satisfy any requirements.
pub(super) struct SelectionSetSolver<'schema, 'a> {
    solver: &'a mut OperationSolver<'schema>,
    query_path: &'a QueryPath,
    maybe_parent: Option<&'a PlanningLogic<'schema>>,
    children: Vec<PlanId>,
    extra_response_key_suffix: usize,
}

impl<'schema, 'a> SelectionSetSolver<'schema, 'a> {
    pub(super) fn new(
        solver: &'a mut OperationSolver<'schema>,
        query_path: &'a QueryPath,
        maybe_parent: Option<&'a PlanningLogic<'schema>>,
    ) -> Self {
        Self {
            solver,
            query_path,
            maybe_parent,
            children: Vec::new(),
            extra_response_key_suffix: 0,
        }
    }

    #[instrument(
        level = Level::DEBUG,
        skip_all,
        fields(parent = %self.maybe_parent.as_ref().map(|p| p.to_string()).unwrap_or_default(),
               path = %self.solver.walker().walk(self.query_path))
    )]
    pub(super) fn solve(
        &mut self,
        selection_set_id: SelectionSetId,
        planned_field_ids: Vec<FieldId>,
        unplanned_field_ids: Vec<FieldId>,
    ) -> PlanningResult<()> {
        let mut planned_selection_set = self.build_planned_selection_set(selection_set_id, &planned_field_ids);
        let missing = self.build_unplanned_fields(unplanned_field_ids);
        self.solve_selection_set(&mut planned_selection_set, missing)?;

        self.solver
            .solved_requirements
            .push((selection_set_id, Self::build_solved_requirements(planned_selection_set)));

        Ok(())
    }

    fn build_solved_requirements(planned_selection_set: PlannedSelectionSet) -> SolvedRequiredFieldSet {
        planned_selection_set
            .fields
            .into_values()
            .flat_map(|fields| {
                fields.into_iter().filter_map(|field| match field {
                    PlannedField::Query {
                        field_id,
                        required_field_id,
                        lazy_subselection,
                        ..
                    } => required_field_id.map(|id| SolvedRequiredField {
                        id,
                        field_id,
                        subselection: lazy_subselection
                            .map(Self::build_solved_requirements)
                            .unwrap_or_default(),
                    }),
                    PlannedField::Extra {
                        required_field_id,
                        field_id,
                        subselection,
                        ..
                    } => field_id.map(|field_id| SolvedRequiredField {
                        id: required_field_id,
                        field_id,
                        subselection: Self::build_solved_requirements(subselection),
                    }),
                })
            })
            .collect()
    }

    fn build_planned_selection_set(&self, id: SelectionSetId, planned_field_ids: &[FieldId]) -> PlannedSelectionSet {
        let mut fields = HashMap::<_, Vec<_>>::with_capacity(planned_field_ids.len());
        for field_id in planned_field_ids {
            let field_id = *field_id;
            if let Some(definition_id) = self.operation[field_id].definition_id() {
                // At this stage we're generating boundary fields for an existing selection set which
                // was already planned. By construction, as soon as we create a new plan with
                // push_plan() it plans all of the nested selection sets.
                // And for extra fields we add during planning, those are attributed immediately.
                let plan_id = self.solver[field_id].expect("field should be planned");

                fields.entry(definition_id).or_default().push(PlannedField::Query {
                    field_id,
                    plan_id,
                    required_field_id: None,
                    lazy_subselection: None,
                })
            }
        }
        PlannedSelectionSet { id: Some(id), fields }
    }

    fn build_unplanned_fields(
        &self,
        unplanned_field_ids: Vec<FieldId>,
    ) -> HashMap<FieldId, FieldDefinitionWalker<'schema>> {
        unplanned_field_ids
            .into_iter()
            .map(|field_id| {
                let definition_id = self.operation[field_id]
                    .definition_id()
                    .expect("__typename doesn't need any planning.");
                (field_id, self.schema.walk(definition_id))
            })
            .collect()
    }
}

impl<'schema, 'a> std::ops::Deref for SelectionSetSolver<'schema, 'a> {
    type Target = OperationSolver<'schema>;
    fn deref(&self) -> &Self::Target {
        self.solver
    }
}

impl<'schema, 'a> std::ops::DerefMut for SelectionSetSolver<'schema, 'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.solver
    }
}

#[derive(Default)]
struct PlannedSelectionSet {
    /// For extra fields sub selection set, we only reserve an id if it's actually needed.
    id: Option<SelectionSetId>,
    fields: HashMap<FieldDefinitionId, Vec<PlannedField>>,
}

enum PlannedField {
    Query {
        field_id: FieldId,
        required_field_id: Option<RequiredFieldId>,
        plan_id: PlanId,
        lazy_subselection: Option<PlannedSelectionSet>,
    },
    Extra {
        field_id: Option<FieldId>,
        petitioner_field_id: FieldId,
        required_field_id: RequiredFieldId,
        plan_id: PlanId,
        subselection: PlannedSelectionSet,
    },
}

impl PlannedField {
    fn required_field_id(&self) -> Option<RequiredFieldId> {
        match self {
            Self::Query { required_field_id, .. } => *required_field_id,
            Self::Extra { required_field_id, .. } => Some(*required_field_id),
        }
    }
}

/// Potential child plan, but might not be the best one.
struct ChildPlanCandidate<'schema> {
    resolver_id: ResolverId,
    /// Providable fields by the resolvers with their requirements
    providable_fields: Vec<(FieldId, &'schema RequiredFieldSet)>,
}

impl<'schema, 'a> SelectionSetSolver<'schema, 'a> {
    /// Iteratively plan fields.
    /// 1. Generate all potential plan candidates satisfying their requirements if possible.
    /// 2. Select the best candidate, generate its input and remove its output fields from the
    ///    unplanned ones.
    /// 3. Continue until there are no more unplanned fields.
    fn solve_selection_set(
        &mut self,
        planned_selection_set: &mut PlannedSelectionSet,
        mut unplanned_fields: HashMap<FieldId, FieldDefinitionWalker<'schema>>,
    ) -> PlanningResult<()> {
        // unplanned_field may be still be provided by the parent plan, but at this stage it
        // means they had requirements.
        if let Some(parent_logic) = self.maybe_parent {
            let mut planned = Vec::new();
            for (&id, definition) in &unplanned_fields {
                // If the parent plan can provide the field, we don't need to plan it.
                if parent_logic.is_providable(definition.id())
                    && self.could_plan_requirements(
                        planned_selection_set,
                        id,
                        definition.requires(parent_logic.resolver().subgraph_id()),
                    )?
                {
                    planned.push(id);
                    continue;
                }
            }

            let mut requires = RequiredFieldSet::default();
            let mut field_ids = vec![];
            for id in planned {
                let definition = unplanned_fields.remove(&id).unwrap();
                requires = requires.union(definition.requires(parent_logic.resolver().subgraph_id()));
                field_ids.push(id);
            }

            self.solver
                .plan_obviously_providable_subselections(self.query_path, parent_logic, &field_ids)?;
            self.push_plan_requires_dependencies(planned_selection_set, parent_logic.plan_id(), &requires);
        }

        if unplanned_fields.is_empty() {
            return Ok(());
        }

        // Actual planning, we plan one child plan at a time.
        let mut candidates: HashMap<ResolverId, ChildPlanCandidate<'schema>> = HashMap::default();
        while !unplanned_fields.is_empty() {
            candidates.clear();
            self.generate_all_candidates(&unplanned_fields, planned_selection_set, &mut candidates)?;

            let Some(candidate) = select_best_child_plan(&mut candidates) else {
                let walker = self.walker();
                tracing::debug!(
                    "Could not plan fields:\n=== PARENT ===\n{:#?}\n=== CURRENT ===\n{}\n=== MISSING ===\n{}",
                    self.maybe_parent.map(|parent| parent.resolver()),
                    planned_selection_set
                        .fields
                        .keys()
                        .map(|id| self.schema.walk(*id))
                        .format_with("\n", |field, f| f(&format_args!("{field:#?}"))),
                    unplanned_fields
                        .keys()
                        .map(|id| walker.walk(*id).definition().unwrap())
                        .format_with("\n", |field, f| f(&format_args!("{field:#?}")))
                );
                return Err(PlanningError::CouldNotPlanAnyField {
                    missing: unplanned_fields
                        .into_keys()
                        .map(|id| walker.walk(id).response_key_str().to_string())
                        .collect(),
                    query_path: walker.walk(self.query_path).iter().map(|s| s.to_string()).collect(),
                });
            };

            let mut requires = Cow::Borrowed(self.schema.walk(candidate.resolver_id).requires());
            let mut field_ids = vec![];
            for (id, field_requires) in std::mem::take(&mut candidate.providable_fields) {
                unplanned_fields.remove(&id);
                if !field_requires.is_empty() {
                    requires = Cow::Owned(requires.union(field_requires));
                }
                field_ids.push(id);
            }
            self.push_child(planned_selection_set, candidate.resolver_id, requires, field_ids)?;
        }

        Ok(())
    }

    fn push_child(
        &mut self,
        planned_selection_set: &mut PlannedSelectionSet,
        resolver_id: ResolverId,
        requires: Cow<'_, RequiredFieldSet>,
        root_field_ids: Vec<FieldId>,
    ) -> PlanningResult<()> {
        let path = self.query_path.clone();
        let plan_id = self.solver.push_plan(path, resolver_id, &root_field_ids)?;
        self.push_plan_requires_dependencies(planned_selection_set, plan_id, &requires);
        for field_id in root_field_ids {
            let definition_id = self.operation[field_id]
                .definition_id()
                .expect("field should have a definition");
            planned_selection_set
                .fields
                .entry(definition_id)
                .or_default()
                .push(PlannedField::Query {
                    field_id,
                    required_field_id: None,
                    plan_id,
                    lazy_subselection: None,
                });
        }

        self.children.push(plan_id);
        Ok(())
    }

    /// Create the input selection set of a Plan given its resolver and requirements.
    /// We iterate over the requirements and find the matching fields inside the boundary fields,
    /// which contains all providable & extra fields. During the iteration we track all the dependency
    /// plans.
    fn push_plan_requires_dependencies(
        &mut self,
        planned_selection_set: &mut PlannedSelectionSet,
        plan_id: PlanId,
        requires: &RequiredFieldSet,
    ) {
        for required_field in requires {
            let definition_id = self.schema.walk(required_field).definition().id();
            let planned_field = planned_selection_set
                .fields
                .get_mut(&definition_id)
                .expect("We depend on it, so it must have been planned")
                .iter_mut()
                .find(|field| field.required_field_id() == Some(required_field.id))
                .expect("We depend on it, so it must have been planned");
            match planned_field {
                PlannedField::Query {
                    plan_id: parent_plan_id,
                    lazy_subselection,
                    ..
                } => {
                    self.solver.push_plan_dependency(ParentToChildEdge {
                        parent: *parent_plan_id,
                        child: plan_id,
                    });
                    if let Some(planned_subselection) = lazy_subselection {
                        self.push_plan_requires_dependencies(
                            planned_subselection,
                            plan_id,
                            &required_field.subselection,
                        )
                    }
                }
                PlannedField::Extra {
                    field_id,
                    petitioner_field_id,
                    required_field_id,
                    plan_id: parent_plan_id,
                    subselection,
                } => {
                    // Now we're sure this filed is needed by plan, so it has to be in the
                    // operation. We will add it to a selection set at the end.
                    if field_id.is_none() {
                        let key = self.generate_response_key_for(definition_id);
                        let parent_selection_set_id =
                            planned_selection_set.id.expect("Parent was required, so should exist");
                        let field = Field::Extra(ExtraField {
                            edge: UnpackedResponseEdge::ExtraFieldResponseKey(key.into()).pack(),
                            field_definition_id: definition_id,
                            selection_set_id: None,
                            argument_ids: self.create_arguments_for(*required_field_id),
                            petitioner_location: self.operation[*petitioner_field_id].location(),
                            condition: None,
                            parent_selection_set_id,
                        });
                        self.operation.fields.push(field);
                        self.field_to_plan_id.push(Some(*parent_plan_id));
                        let id = (self.operation.fields.len() - 1).into();
                        *field_id = Some(id);
                        self.operation[parent_selection_set_id].field_ids.push(id);
                    }

                    self.solver.push_plan_dependency(ParentToChildEdge {
                        parent: *parent_plan_id,
                        child: plan_id,
                    });

                    if !required_field.subselection.is_empty() {
                        if subselection.id.is_none() {
                            self.operation.selection_sets.push(SelectionSet::default());
                            subselection.id = Some((self.operation.selection_sets.len() - 1).into());
                        }
                        self.push_plan_requires_dependencies(subselection, plan_id, &required_field.subselection)
                    }
                }
            }
        }
    }

    fn generate_all_candidates<'field>(
        &mut self,
        unplanned_fields: &HashMap<FieldId, FieldDefinitionWalker<'schema>>,
        planned_selection_set: &mut PlannedSelectionSet,
        candidates: &mut HashMap<ResolverId, ChildPlanCandidate<'schema>>,
    ) -> PlanningResult<()>
    where
        'schema: 'field,
    {
        for (&id, definition) in unplanned_fields {
            for resolver in definition.resolvers() {
                tracing::trace!("Trying to plan '{}' with: {}", definition.name(), resolver.name());
                let field_requires = definition.requires(resolver.subgraph_id());
                match candidates.entry(resolver.id()) {
                    Entry::Occupied(mut entry) => {
                        let candidate = entry.get_mut();
                        if self.could_plan_requirements(planned_selection_set, id, field_requires)? {
                            candidate.providable_fields.push((id, field_requires));
                        }
                    }
                    Entry::Vacant(entry) => {
                        if self.could_plan_requirements(planned_selection_set, id, resolver.requires())?
                            && self.could_plan_requirements(planned_selection_set, id, field_requires)?
                        {
                            entry.insert(ChildPlanCandidate {
                                resolver_id: resolver.id(),
                                providable_fields: vec![(id, field_requires)],
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
        planned_selection_set: &mut PlannedSelectionSet,
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
            planned_selection_set,
            petitioner_field_id,
            requires,
        )
    }

    fn could_plan_requirements_on_previous_plans(
        &mut self,
        parent_field_plan_id: PlanId,
        planned_selection_set: &mut PlannedSelectionSet,
        petitioner_field_id: FieldId,
        requires: &'schema RequiredFieldSet,
    ) -> PlanningResult<bool> {
        if requires.is_empty() {
            return Ok(true);
        }
        'requires: for required in requires {
            let required_field = &self.schema[required.id];

            // -- Existing fields --
            if let Some(fields) = planned_selection_set.fields.get_mut(&required_field.definition_id) {
                for field in fields {
                    match field {
                        PlannedField::Query {
                            field_id,
                            plan_id,
                            required_field_id,
                            lazy_subselection,
                        } => {
                            // If argument don't match, trying another group
                            if !self.walker().walk(*field_id).eq(required_field) {
                                continue;
                            }

                            *required_field_id = Some(required.id);

                            // If there is no require sub-selection, we already have everything we need.
                            if required.subselection.is_empty() {
                                continue 'requires;
                            }

                            if lazy_subselection.is_none() {
                                *lazy_subselection = self.operation[*field_id]
                                    .selection_set_id()
                                    .map(|id| self.build_planned_selection_set(id, &self.operation[id].field_ids));
                            }

                            // Now we only need to know whether we can plan the field, We don't bother with
                            // other groups. I'm not sure whether having response key groups with different
                            // plan ids for the same FieldDefinitionId would ever happen.
                            // So either we can plan the necessary requirements with this group or we
                            // don't.
                            if self.could_plan_requirements_on_previous_plans(
                                *plan_id,
                                lazy_subselection.as_mut().unwrap(),
                                petitioner_field_id,
                                &required.subselection,
                            )? {
                                continue 'requires;
                            } else {
                                return Ok(false);
                            }
                        }
                        PlannedField::Extra {
                            required_field_id,
                            plan_id,
                            subselection,
                            ..
                        } => {
                            if *required_field_id != required.id {
                                continue;
                            }

                            // If there is no require sub-selection, we already have everything we need.
                            if required.subselection.is_empty() {
                                continue 'requires;
                            }

                            // Now we only need to know whether we can plan the field, We don't bother with
                            // other groups. I'm not sure whether having response key groups with different
                            // plan ids for the same FieldDefinitionId would ever happen.
                            // So either we can plan the necessary requirements with this group or we
                            // don't.
                            if self.could_plan_requirements_on_previous_plans(
                                *plan_id,
                                subselection,
                                petitioner_field_id,
                                &required.subselection,
                            )? {
                                continue 'requires;
                            } else {
                                return Ok(false);
                            }
                        }
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
                && self.could_plan_exra_field(planned_selection_set, petitioner_field_id, parent_logic, required)
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
                    planned_selection_set,
                    petitioner_field_id,
                    &PlanningLogic::new(plan_id, self.schema.walk(self[plan_id].resolver_id)),
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
        planned_selection_set: &mut PlannedSelectionSet,
        petitioner_field_id: FieldId,
        logic: &PlanningLogic<'schema>,
        required: RequiredFieldSetItemWalker<'schema>,
    ) -> bool {
        if !logic.is_providable(required.definition().id()) {
            return false;
        }
        let definition = required.definition();
        let field_logic = logic.child(definition.id());
        let mut subselection = PlannedSelectionSet::default();
        for field in required.subselection() {
            if !self.could_plan_exra_field(&mut subselection, petitioner_field_id, &field_logic, field) {
                return false;
            }
        }

        planned_selection_set
            .fields
            .entry(definition.id())
            .or_default()
            .push(PlannedField::Extra {
                field_id: None,
                petitioner_field_id,
                required_field_id: required.required_field_id(),
                plan_id: logic.plan_id(),
                subselection,
            });

        tracing::trace!(
            "Added extra field '{}' provided by {} required by '{}'",
            self.schema.walker().walk(required.definition().id()).name(),
            logic.plan_id(),
            self.walker().walk(petitioner_field_id).response_key_str()
        );

        true
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
