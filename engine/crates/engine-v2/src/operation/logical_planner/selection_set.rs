use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
};

use id_newtypes::IdRange;
use itertools::Itertools;
use schema::{
    EntityDefinitionId, FieldDefinition, FieldDefinitionId, RequiredFieldArgumentRecord, RequiredFieldId,
    RequiredFieldSetItem, RequiredFieldSetRecord, ResolverDefinitionId,
};
use tracing::{instrument, Level};
use walker::Walk;

use super::{logic::PlanningLogic, LogicalPlanner, LogicalPlanningError, LogicalPlanningResult, ParentToChildEdge};
use crate::{
    operation::{
        ExtraField, Field, FieldArgument, FieldArgumentId, FieldId, LogicalPlanId, QueryInputValue, QueryPath,
        SelectionSet, SelectionSetId, SolvedRequiredField, SolvedRequiredFieldSet,
    },
    response::{SafeResponseKey, UnpackedResponseEdge},
};

pub(super) struct SelectionSetLogicalPlanner<'schema, 'a> {
    planner: &'a mut LogicalPlanner<'schema>,
    query_path: &'a QueryPath,
    maybe_parent: Option<&'a PlanningLogic<'schema>>,
    children: Vec<LogicalPlanId>,
    extra_response_key_suffix: usize,
}

impl<'schema, 'a> SelectionSetLogicalPlanner<'schema, 'a> {
    pub(super) fn new(
        planner: &'a mut LogicalPlanner<'schema>,
        query_path: &'a QueryPath,
        maybe_parent: Option<&'a PlanningLogic<'schema>>,
    ) -> Self {
        Self {
            planner,
            query_path,
            maybe_parent,
            children: Vec::new(),
            extra_response_key_suffix: 0,
        }
    }

    /// Solves the selection set by planning all child fields recursively and resolving their requirements.
    ///
    /// This function is responsible for creating a logical plan for a selection set identified by
    /// `selection_set_id`. It considers the requirements of parent fields, discerns which fields are
    /// already planned, and identifies which fields need to be planned. During this process, it
    /// also tracks any extra fields that are necessary for the resolution of the selection set.
    #[instrument(
        level = Level::DEBUG,
        skip_all,
        fields(parent = %self.maybe_parent.as_ref().map(|p| p.to_string()).unwrap_or_default(),
               path = %self.planner.walker().walk(self.query_path))
    )]
    pub(super) fn solve(
        &mut self,
        selection_set_id: SelectionSetId,
        parent_field_requirements: Option<(FieldId, Cow<'schema, RequiredFieldSetRecord>)>,
        planned_field_ids: Vec<FieldId>,
        unplanned_field_ids: Vec<FieldId>,
    ) -> LogicalPlanningResult<()> {
        tracing::trace!("Solving selection set {}", selection_set_id);

        let mut planned_selection_set = self.build_planned_selection_set(selection_set_id, &planned_field_ids);
        let missing = self.build_unplanned_fields(unplanned_field_ids);

        self.plan_selection_set(&mut planned_selection_set, parent_field_requirements, missing)?;

        // During the planning we add extra fields as necessary but we don't add them in the
        // children plan root fields.
        let maybe_parent_plan_id = self.maybe_parent.map(|p| p.id());
        for fields in planned_selection_set.fields.values() {
            for field in fields {
                let PlannedField::Extra(extra) = field else {
                    continue;
                };
                let Some(field_id) = extra.field_id else {
                    continue;
                };
                // Except the parent plan, all others have for root this selection set.
                if Some(extra.logical_plan_id) != maybe_parent_plan_id {
                    // Sorted at the end.
                    self.planner[extra.logical_plan_id]
                        .root_field_ids_ordered_by_parent_entity_id_then_position
                        .push(field_id);
                }
            }
        }

        self.build_solved_requirements(planned_selection_set);

        if !self.children.is_empty() {
            // At least one child will read something from this selection set
            self.selection_set_to_objects_must_be_tracked
                .set(selection_set_id, true);
        }

        Ok(())
    }

    fn build_solved_requirements(&mut self, planned_selection_set: PlannedSelectionSet) -> SolvedRequiredFieldSet {
        let mut solved_fields = Vec::new();
        for fields in planned_selection_set.fields.into_values() {
            for field in fields {
                let solved_field = match field {
                    PlannedField::Query {
                        field_id,
                        required_field_id,
                        lazy_subselection,
                        ..
                    } => required_field_id.map(|id| SolvedRequiredField {
                        id,
                        field_id,
                        subselection: lazy_subselection
                            .map(|subselection| self.build_solved_requirements(subselection))
                            .unwrap_or_default(),
                    }),
                    PlannedField::Extra(ExtraPlannedField {
                        required_field_id,
                        field_id,
                        subselection,
                        ..
                    }) => field_id.map(|field_id| SolvedRequiredField {
                        id: required_field_id,
                        field_id,
                        subselection: self.build_solved_requirements(subselection),
                    }),
                };
                if let Some(solved) = solved_field {
                    self.field_to_solved_requirement[usize::from(solved.field_id)] = Some(solved.id);
                    solved_fields.push(solved);
                }
            }
        }

        if !solved_fields.is_empty() {
            self.planner
                .solved_requirements
                .push((planned_selection_set.id.expect("kekw"), solved_fields.clone()));
        }

        solved_fields
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
                let plan_id = self.planner[field_id].expect("field should be planned");

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

    fn build_unplanned_fields(&self, unplanned_field_ids: Vec<FieldId>) -> HashMap<FieldId, FieldDefinition<'schema>> {
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

impl<'schema, 'a> std::ops::Deref for SelectionSetLogicalPlanner<'schema, 'a> {
    type Target = LogicalPlanner<'schema>;
    fn deref(&self) -> &Self::Target {
        self.planner
    }
}

impl<'schema, 'a> std::ops::DerefMut for SelectionSetLogicalPlanner<'schema, 'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.planner
    }
}

#[derive(Debug, Default, Clone)]
// TODO: add deleted fields
struct PlannedSelectionSet {
    /// For extra fields sub selection set, we only reserve an id if it's actually needed.
    id: Option<SelectionSetId>,
    fields: HashMap<FieldDefinitionId, Vec<PlannedField>>,
}

#[derive(Debug, Clone)]
enum PlannedField {
    Query {
        field_id: FieldId,
        required_field_id: Option<RequiredFieldId>,
        plan_id: LogicalPlanId,
        lazy_subselection: Option<PlannedSelectionSet>,
    },
    Extra(ExtraPlannedField),
}

#[derive(Debug, Clone)]
pub struct ExtraPlannedField {
    field_id: Option<FieldId>,
    petitioner_field_id: FieldId,
    required_field_id: RequiredFieldId,
    logical_plan_id: LogicalPlanId,
    subselection: PlannedSelectionSet,
}

impl PlannedField {
    fn required_field_id(&self) -> Option<RequiredFieldId> {
        match self {
            Self::Query { required_field_id, .. } => *required_field_id,
            Self::Extra(ExtraPlannedField { required_field_id, .. }) => Some(*required_field_id),
        }
    }
}

/// Potential child plan, but might not be the best one.
struct ChildPlanCandidate<'schema> {
    entity_id: EntityDefinitionId,
    resolver_id: ResolverDefinitionId,
    /// Providable fields by the resolvers with their requirements
    providable_fields: Vec<(FieldId, Cow<'schema, RequiredFieldSetRecord>)>,
}

impl<'schema, 'a> SelectionSetLogicalPlanner<'schema, 'a> {
    /// Iteratively plan fields.
    /// 1. Generate all potential plan candidates satisfying their requirements if possible.
    /// 2. Select the best candidate, generate its input and remove its output fields from the
    ///    unplanned ones.
    /// 3. Continue until there are no more unplanned fields.
    fn plan_selection_set(
        &mut self,
        planned_selection_set: &mut PlannedSelectionSet,
        mut parent_field_requirements: Option<(FieldId, Cow<'schema, RequiredFieldSetRecord>)>,
        mut unplanned_fields: HashMap<FieldId, FieldDefinition<'schema>>,
    ) -> LogicalPlanningResult<()> {
        // unplanned_field may be still be provided by the parent plan, but at this stage it
        // means they had requirements.
        if let Some(parent_logic) = self.maybe_parent {
            let mut requires = Cow::Owned(RequiredFieldSetRecord::default());
            let mut planned_field_ids = vec![];

            for (&id, definition) in &unplanned_fields {
                // If the parent plan can provide the field, we don't need to plan it.
                let required_fields =
                    definition.all_requires_for_subgraph(parent_logic.resolver().as_ref().subgraph_id());
                if parent_logic.is_providable(definition.id())
                    && self.could_plan_requirements(planned_selection_set, id, &required_fields)?
                {
                    requires = RequiredFieldSetRecord::union_cow(requires, required_fields);
                    planned_field_ids.push(id);
                    continue;
                }
            }

            if let Some((parent_field_id, parent_extra_requirements)) = &mut parent_field_requirements {
                // If the parent plan can provide the field, we don't need to plan it.
                if self.could_plan_requirements(planned_selection_set, *parent_field_id, parent_extra_requirements)? {
                    requires = RequiredFieldSetRecord::union_cow(requires, std::mem::take(parent_extra_requirements));
                }
            }

            for id in &planned_field_ids {
                unplanned_fields.remove(id);
            }

            self.planner.grow_with_obviously_providable_subselections(
                self.query_path,
                parent_logic,
                &planned_field_ids,
            )?;
            self.register_necessary_extra_fields(None, planned_selection_set, &requires);
        }

        if unplanned_fields.is_empty()
            && parent_field_requirements
                .as_ref()
                .map(|(_, requirements)| requirements.is_empty())
                .unwrap_or(true)
        {
            return Ok(());
        }

        // Actual planning, we plan one child plan at a time.
        let mut candidates: HashMap<ResolverDefinitionId, ChildPlanCandidate<'schema>> = HashMap::default();
        while !unplanned_fields.is_empty() {
            candidates.clear();
            self.generate_all_candidates(&mut unplanned_fields, planned_selection_set, &mut candidates)?;

            let Some(candidate) = select_best_child_plan(&mut candidates) else {
                let walker = self.walker();
                let parent_subgraph_id = self.maybe_parent.map(|parent| parent.resolver().as_ref().subgraph_id());

                tracing::error!(
                    "Could not plan fields:\n=== PARENT ===\n{:#?}\n=== CURRENT ===\n{}\n=== MISSING ===\n{}",
                    self.maybe_parent.map(|parent| parent.resolver()),
                    planned_selection_set
                        .fields
                        .keys()
                        .map(|id| self.schema.walk(*id))
                        .format_with("\n", |field, f| f(&format_args!("{field:#?}")))
                        // with opentelemetry this string might be formatted more than once... Leading to a
                        // panic with .format_with()
                        .to_string(),
                    unplanned_fields
                        .keys()
                        .map(|id| walker.walk(*id).definition().unwrap())
                        .format_with("\n\n", |field, f| f(&format_args!(
                            "{field:#?}\n{:#?}",
                            parent_subgraph_id.map(|id| field.all_requires_for_subgraph(id))
                        )))
                        // with opentelemetry this string might be formatted more than once... Leading to a
                        // panic with .format_with()
                        .to_string()
                );
                return Err(LogicalPlanningError::CouldNotPlanAnyField {
                    missing: unplanned_fields
                        .into_keys()
                        .map(|id| walker.walk(id).response_key_str().to_string())
                        .collect(),
                    query_path: walker.walk(self.query_path).iter().map(|s| s.to_string()).collect(),
                });
            };

            let mut requires = Cow::Borrowed(self.schema.walk(candidate.resolver_id).requires_or_empty());
            let mut field_ids = vec![];

            for (id, required_fields) in std::mem::take(&mut candidate.providable_fields) {
                unplanned_fields.remove(&id);
                requires = RequiredFieldSetRecord::union_cow(requires, required_fields);
                field_ids.push(id);
            }

            self.push_child(
                planned_selection_set,
                candidate.resolver_id,
                requires,
                candidate.entity_id,
                field_ids,
            )?;
        }

        if let Some((parent_field_id, parent_extra_requirements)) =
            parent_field_requirements.filter(|(_, requirements)| !requirements.is_empty())
        {
            if self.could_plan_requirements(planned_selection_set, parent_field_id, &parent_extra_requirements)? {
                self.register_necessary_extra_fields(None, planned_selection_set, &parent_extra_requirements);
            } else {
                let walker = self.walker();
                tracing::error!(
                    "Could not plan extra requirements:\n=== PARENT ===\n{:#?}\n=== CURRENT ===\n{}\n=== MISSING ===\nFor {}\n{:#?}",
                    self.maybe_parent.map(|parent| parent.resolver()),
                    planned_selection_set
                        .fields
                        .keys()
                        .map(|id| self.schema.walk(*id))
                        .format_with("\n", |field, f| f(&format_args!("{field:#?}")))
                        // with opentelemetry this string might be formatted more than once... Leading to a
                        // panic with .format_with()
                        .to_string(),
                    walker.walk(parent_field_id).response_key_str(),
                    self.schema.walk(parent_extra_requirements.as_ref())
                );
                return Err(LogicalPlanningError::CouldNotPlanAnyField {
                    missing: parent_extra_requirements
                        .iter()
                        .map(|item| self.schema.walk(item).field().definition().name().to_string())
                        .collect(),
                    query_path: walker.walk(self.query_path).iter().map(|s| s.to_string()).collect(),
                });
            }
        }

        Ok(())
    }

    fn push_child(
        &mut self,
        planned_selection_set: &mut PlannedSelectionSet,
        resolver_id: ResolverDefinitionId,
        requires: Cow<'_, RequiredFieldSetRecord>,
        entity_id: EntityDefinitionId,
        root_field_ids: Vec<FieldId>,
    ) -> LogicalPlanningResult<()> {
        let path = self.query_path.clone();
        let plan_id = self.planner.push_plan(path, resolver_id, entity_id, &root_field_ids)?;

        self.register_necessary_extra_fields(Some(plan_id), planned_selection_set, &requires);

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
    fn register_necessary_extra_fields(
        &mut self,
        dependent_plan_id: Option<LogicalPlanId>,
        planned_selection_set: &mut PlannedSelectionSet,
        requires: &RequiredFieldSetRecord,
    ) {
        for required_field in requires {
            let definition_id = self.schema.walk(required_field).field().definition().id();

            let planned_field = planned_selection_set
                .fields
                .get_mut(&definition_id)
                .expect("We depend on it, so it must have been planned")
                .iter_mut()
                .find(|field| field.required_field_id() == Some(required_field.field_id))
                .expect("We depend on it, so it must have been planned");

            required_field.field_id.walk(self.schema).definition().name();
            match planned_field {
                PlannedField::Query {
                    lazy_subselection,
                    plan_id: parent_plan_id,
                    ..
                } => {
                    if let Some(child) = dependent_plan_id {
                        self.register_plan_child(ParentToChildEdge {
                            parent: *parent_plan_id,
                            child,
                        });
                    }
                    if let Some(planned_subselection) = lazy_subselection {
                        self.register_necessary_extra_fields(
                            dependent_plan_id,
                            planned_subselection,
                            &required_field.subselection,
                        )
                    }
                }
                PlannedField::Extra(field) => {
                    if let Some(child) = dependent_plan_id {
                        self.register_plan_child(ParentToChildEdge {
                            parent: field.logical_plan_id,
                            child,
                        });
                    }

                    let selection_set_id = if !required_field.subselection.is_empty() {
                        if field.subselection.id.is_none() {
                            self.operation.selection_sets.push(SelectionSet::default());
                            field.subselection.id = Some((self.operation.selection_sets.len() - 1).into());
                            self.selection_set_to_objects_must_be_tracked.push(false);
                        }

                        self.register_necessary_extra_fields(
                            dependent_plan_id,
                            &mut field.subselection,
                            &required_field.subselection,
                        );

                        field.subselection.id
                    } else {
                        None
                    };

                    // Now we're sure this filed is needed by plan, so it has to be in the
                    // operation. We will add it to a selection set at the end.
                    if field.field_id.is_none() {
                        self.insert_extra_field(
                            planned_selection_set.id.expect("Parent was required, so should exist"),
                            definition_id,
                            selection_set_id,
                            field,
                        );
                    }
                }
            }
        }
    }

    fn insert_extra_field(
        &mut self,
        parent_selection_set_id: SelectionSetId,
        definition_id: FieldDefinitionId,
        selection_set_id: Option<SelectionSetId>,
        planned_field: &mut ExtraPlannedField,
    ) {
        // Creating the field
        let key = self.generate_response_key_for(definition_id);

        let field = Field::Extra(ExtraField {
            edge: UnpackedResponseEdge::ExtraFieldResponseKey(key.into()).pack(),
            definition_id,
            selection_set_id,
            argument_ids: self.create_arguments_for(planned_field.required_field_id),
            petitioner_location: self.operation[planned_field.petitioner_field_id].location(),
            parent_selection_set_id,
        });

        self.operation.fields.push(field);
        self.field_to_logical_plan_id.push(Some(planned_field.logical_plan_id));

        self.field_to_solved_requirement
            .push(Some(planned_field.required_field_id));

        let id = (self.operation.fields.len() - 1).into();
        planned_field.field_id = Some(id);

        self.insert_field_in_parent_selection_set(parent_selection_set_id, definition_id, id);
    }

    fn insert_field_in_parent_selection_set(
        &mut self,
        parent_selection_set_id: SelectionSetId,
        definition_id: FieldDefinitionId,
        id: FieldId,
    ) {
        // Inserting into its parent selection set in order.
        let mut field_ids = std::mem::take(
            &mut self.operation[parent_selection_set_id].field_ids_ordered_by_parent_entity_id_then_position,
        );

        let extra_parent_entity_id = Some(self.schema[definition_id].parent_entity_id);
        let extra_query_position = self.operation[id].query_position();

        let extra_field_position = field_ids
            .binary_search_by(|probe_id| {
                let probe_field = &self.operation[*probe_id];
                probe_field
                    .definition_id()
                    .map(|id| self.schema[id].parent_entity_id)
                    .cmp(&extra_parent_entity_id)
                    .then(probe_field.query_position().cmp(&extra_query_position))
            })
            .expect_err("extra field cannot be present already");

        field_ids.insert(extra_field_position, id);
        self.operation[parent_selection_set_id].field_ids_ordered_by_parent_entity_id_then_position = field_ids;
    }

    fn generate_all_candidates<'field>(
        &mut self,
        unplanned_fields: &mut HashMap<FieldId, FieldDefinition<'schema>>,
        planned_selection_set: &mut PlannedSelectionSet,
        candidates: &mut HashMap<ResolverDefinitionId, ChildPlanCandidate<'schema>>,
    ) -> LogicalPlanningResult<()>
    where
        'schema: 'field,
    {
        let mut interface_fields_to_replan = Vec::new();

        for (&id, definition) in unplanned_fields.iter() {
            definition.name();
            for resolver_id in definition.as_ref().resolver_ids.iter().copied() {
                let resolver = self.schema.walk(resolver_id);
                tracing::trace!("Trying to plan '{}' with: {}", definition.name(), resolver.name());

                let required_fields = definition.all_requires_for_subgraph(resolver.as_ref().subgraph_id());

                match candidates.entry(resolver_id) {
                    Entry::Occupied(mut entry) => {
                        let candidate = entry.get_mut();
                        if self.could_plan_requirements(planned_selection_set, id, &required_fields)? {
                            candidate.providable_fields.push((id, required_fields));
                        }
                    }
                    Entry::Vacant(entry) => {
                        if self.could_plan_requirements(planned_selection_set, id, resolver.requires_or_empty())?
                            && self.could_plan_requirements(planned_selection_set, id, &required_fields)?
                        {
                            entry.insert(ChildPlanCandidate {
                                resolver_id,
                                entity_id: definition.parent_entity().id(),
                                providable_fields: vec![(id, required_fields)],
                            });
                        }
                    }
                }
            }

            // We did not find any candicates from unplanned fields. If our field we try to plan with
            // is in an interface, which does not have keys set, we need to re-plan these fields from all
            // the types implementing the interface.
            match definition.parent_entity() {
                schema::EntityDefinition::Interface(interface) if candidates.is_empty() => {
                    let mut add_to_plan = Vec::new();

                    for r#type in interface.possible_types() {
                        if let Some(field) = r#type.fields().find(|field| field.name() == definition.name()) {
                            add_to_plan.push(field);
                        }
                    }

                    interface_fields_to_replan.push((id, add_to_plan));
                }
                _ => (),
            }
        }

        match planned_selection_set.id {
            Some(selection_set_id) if !interface_fields_to_replan.is_empty() => {
                self.replan_interface_fields(selection_set_id, interface_fields_to_replan, unplanned_fields);
                self.generate_all_candidates(unplanned_fields, planned_selection_set, candidates)?
            }
            _ => (),
        }

        Ok(())
    }

    /// Replans interface fields by removing existing fields and adding new ones
    /// based on the fields in types implementing the interface.
    fn replan_interface_fields(
        &mut self,
        selection_set_id: SelectionSetId,
        interface_fields_to_replan: Vec<(FieldId, Vec<FieldDefinition<'schema>>)>,
        unplanned_fields: &mut HashMap<FieldId, FieldDefinition<'schema>>,
    ) {
        for (interface_field_id, add_to_plan) in interface_fields_to_replan {
            unplanned_fields.remove(&interface_field_id);
            self.mark_field_as_never_planned(interface_field_id);

            let position = self.operation[selection_set_id]
                .field_ids_ordered_by_parent_entity_id_then_position
                .iter()
                .position(|p| p == &interface_field_id)
                .unwrap();

            self.operation[selection_set_id]
                .field_ids_ordered_by_parent_entity_id_then_position
                .remove(position);

            for field_definition in add_to_plan {
                let field = {
                    let operation_field = self.walker().walk(interface_field_id).as_ref().clone();
                    let selection_set_id = operation_field.selection_set_id();

                    ExtraField {
                        edge: operation_field.response_edge(),
                        definition_id: field_definition.id(),
                        selection_set_id: selection_set_id.map(|id| self.clone_selection_set(id)),
                        argument_ids: operation_field.argument_ids(),
                        petitioner_location: operation_field.location(),
                        parent_selection_set_id: operation_field.parent_selection_set_id(),
                    }
                };

                let parent_selection_set_id = field.parent_selection_set_id;
                let definition_id = field.definition_id;

                self.operation.fields.push(Field::Extra(field));
                let id = (self.operation.fields.len() - 1).into();
                unplanned_fields.insert(id, field_definition);

                self.field_to_logical_plan_id.push(None);
                self.field_to_solved_requirement.push(None);

                self.insert_field_in_parent_selection_set(parent_selection_set_id, definition_id, id);
            }
        }
    }

    /// Marks a field as never planned in the logical planning process.
    ///
    /// This function updates the necessary fields to indicate that the given `field_id`
    /// has been determined to be unplannable. It sets the logical plan ID to a maximum value
    /// to signal that the field cannot be included in any selection set planning. If the field
    /// is part of a selection set, it recursively marks any dependent fields as never planned.
    fn mark_field_as_never_planned(&mut self, field_id: FieldId) {
        self.field_to_logical_plan_id[usize::from(field_id)] = Some(LogicalPlanId::from(u16::MAX - 1));
        self.field_to_solved_requirement[usize::from(field_id)] = None;

        if let Some(selection_set_id) = self.operation[field_id].selection_set_id() {
            let field_ids = self.operation[selection_set_id]
                .field_ids_ordered_by_parent_entity_id_then_position
                .clone();

            for field_id in field_ids {
                self.mark_field_as_never_planned(field_id);
            }
        }
    }

    /// Clones an existing selection set identified by `selection_set_id` and returns the new selection set's ID.
    ///
    /// This function creates a new selection set that replicates the fields and structures of the existing
    /// one while updating references such as `parent_selection_set_id` and `selection_set_id` in the
    /// cloned fields.
    fn clone_selection_set(&mut self, selection_set_id: SelectionSetId) -> SelectionSetId {
        self.operation.selection_sets.push(SelectionSet::default());

        let previous = self.walker().walk(selection_set_id);
        let selection_set_id = SelectionSetId::from(self.operation.selection_sets.len() - 1);
        let mut next = SelectionSet::default();

        for previous_field_id in previous
            .as_ref()
            .field_ids_ordered_by_parent_entity_id_then_position
            .clone()
        {
            let previous_field = self.walker().walk(previous_field_id);
            let mut field = previous_field.as_ref().clone();

            let Field::Query(ref mut query_field) = field else {
                todo!("cloning non-query fields not yet supported");
            };

            query_field.parent_selection_set_id = selection_set_id;

            let old_id = previous_field.as_ref().selection_set_id();
            query_field.selection_set_id = old_id.map(|id| self.clone_selection_set(id));

            self.operation.fields.push(field);
            self.field_to_logical_plan_id.push(None);
            self.field_to_solved_requirement.push(None);

            let id = (self.operation.fields.len() - 1).into();
            next.field_ids_ordered_by_parent_entity_id_then_position.push(id);
        }

        self.operation[selection_set_id] = next;
        self.selection_set_to_objects_must_be_tracked.push(false);

        selection_set_id
    }

    /// Allows us to know whether a field requirements can be provided at all to order the next child
    /// candidates.
    fn could_plan_requirements(
        &mut self,
        planned_selection_set: &mut PlannedSelectionSet,
        petitioner_field_id: FieldId,
        requires: &RequiredFieldSetRecord,
    ) -> LogicalPlanningResult<bool> {
        if requires.is_empty() {
            return Ok(true);
        }
        self.could_plan_requirements_on_previous_plans(None, planned_selection_set, petitioner_field_id, requires)
    }

    fn could_plan_requirements_on_previous_plans(
        &mut self,
        parent_logical_plan_id: Option<LogicalPlanId>,
        planned_selection_set: &mut PlannedSelectionSet,
        petitioner_field_id: FieldId,
        requires: &RequiredFieldSetRecord,
    ) -> LogicalPlanningResult<bool> {
        if requires.is_empty() {
            return Ok(true);
        }
        'requires: for required in requires {
            let required_field = &self.schema[required.field_id];

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

                            *required_field_id = Some(required.field_id);

                            // If there is no require sub-selection, we already have everything we need.
                            if required.subselection.is_empty() {
                                continue 'requires;
                            }

                            if lazy_subselection.is_none() {
                                *lazy_subselection = self.operation[*field_id].selection_set_id().map(|id| {
                                    self.build_planned_selection_set(
                                        id,
                                        &self.operation[id].field_ids_ordered_by_parent_entity_id_then_position,
                                    )
                                });
                            }

                            // Now we only need to know whether we can plan the field, We don't bother with
                            // other groups. I'm not sure whether having response key groups with different
                            // plan ids for the same FieldDefinitionId would ever happen.
                            // So either we can plan the necessary requirements with this group or we
                            // don't.
                            if self.could_plan_requirements_on_previous_plans(
                                Some(*plan_id),
                                lazy_subselection.as_mut().unwrap(),
                                petitioner_field_id,
                                &required.subselection,
                            )? {
                                continue 'requires;
                            } else {
                                return Ok(false);
                            }
                        }
                        PlannedField::Extra(ExtraPlannedField {
                            required_field_id,
                            logical_plan_id,
                            subselection,
                            ..
                        }) => {
                            if *required_field_id != required.field_id {
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
                                Some(*logical_plan_id),
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

            // if we're within a nested selection set, we only handle the case where the parent
            // resolver can provide this field.
            if let Some(parent_resolved_query_part_id) = parent_logical_plan_id {
                return Ok(self.could_plan_exra_field(
                    planned_selection_set,
                    petitioner_field_id,
                    &PlanningLogic::new(
                        parent_resolved_query_part_id,
                        self.schema,
                        self.schema.walk(self[parent_resolved_query_part_id].resolver_id),
                    ),
                    required,
                ));
            }

            // -- Plannable by the parent --
            if let Some(parent_logic) = self.maybe_parent {
                if self.could_plan_exra_field(planned_selection_set, petitioner_field_id, parent_logic, required) {
                    continue;
                }
            }

            // -- Plannable by existing children --
            for i in 0..self.children.len() {
                let plan_id = self.children[i];
                if self.could_plan_exra_field(
                    planned_selection_set,
                    petitioner_field_id,
                    &PlanningLogic::new(plan_id, self.schema, self.schema.walk(self[plan_id].resolver_id)),
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
        required: RequiredFieldSetItem<'_>,
    ) -> bool {
        if !logic.is_providable(required.field().definition().id()) {
            return false;
        }
        let definition = required.field().definition();
        let field_logic = logic.child(definition.id());
        let mut subselection = PlannedSelectionSet::default();
        for field in required.subselection().items() {
            if !self.could_plan_exra_field(&mut subselection, petitioner_field_id, &field_logic, field) {
                return false;
            }
        }

        planned_selection_set
            .fields
            .entry(definition.id())
            .or_default()
            .push(PlannedField::Extra(ExtraPlannedField {
                field_id: None,
                petitioner_field_id,
                required_field_id: required.field().id(),
                logical_plan_id: logic.id(),
                subselection,
            }));

        tracing::debug!(
            "Added extra field '{}' provided by {} required by '{}'",
            required.field().definition().name(),
            logic.id(),
            self.walker().walk(petitioner_field_id).response_key_str()
        );

        true
    }

    fn generate_response_key_for(&mut self, field_id: FieldDefinitionId) -> SafeResponseKey {
        // Try just using the field name
        let name = self.schema.walk(field_id).name();
        let response_keys = &mut self.operation.response_keys;

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

        for &RequiredFieldArgumentRecord {
            definition_id,
            value_id,
        } in &self.schema.walk(id).as_ref().argument_records
        {
            let input_value_id = self
                .operation
                .query_input_values
                .push_value(QueryInputValue::DefaultValue(value_id));

            self.operation.field_arguments.push(FieldArgument {
                name_location: None,
                value_location: None,
                input_value_id,
                input_value_definition_id: definition_id,
            });
        }

        let end = self.operation.field_arguments.len();
        (start..end).into()
    }
}

fn select_best_child_plan<'c, 'op>(
    candidates: &'c mut HashMap<ResolverDefinitionId, ChildPlanCandidate<'op>>,
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
