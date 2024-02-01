use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap, HashSet},
};

use schema::{FieldId, FieldResolverWalker, FieldSet, FieldSetItem, ResolverId, ResolverWalker, Schema};

use crate::{
    plan::{
        attribution::{AttributionBuilder, ExtraField},
        ChildPlan, EntityType, FlatTypeCondition, PlanId,
    },
    request::{
        BoundFieldId, BoundSelectionSetId, FlatField, FlatSelectionSet, FlatSelectionSetWalker, OperationWalker,
        QueryPath, SelectionSetType,
    },
    response::{ReadField, ReadSelectionSet},
};

use super::boundary_selection_set::*;
use super::{AttributionLogic, ExpectedType, Planner, PlanningError, PlanningResult};

/// Plans the children of a plan at a boundary (where some fields could not be planned).
pub(super) struct PlanBoundaryChildrenPlanner<'op, 'a> {
    planner: &'a mut Planner<'op>,
    walker: OperationWalker<'op>,
    /// There is no parent for the root plans.
    maybe_parent: Option<PlanBoundaryParent<'op, 'a, 'a>>,
    children: Vec<ChildPlan>,
}

/// Parent plan of a boundary
pub(super) struct PlanBoundaryParent<'op, 'plan, 'ctx> {
    pub plan_id: PlanId,
    /// Path within the query for errors
    pub path: &'ctx QueryPath,
    pub logic: AttributionLogic<'op>,
    pub attribution: &'plan mut AttributionBuilder,
    pub provided_selection_set: FlatSelectionSetWalker<'op, 'plan>,
}

/// Potential child plan, but might not be the best one.
struct ChildPlanCandidate<'op> {
    resolver_id: ResolverId,
    /// Entity type (object/interface id) of the fields
    entity_type: EntityType,
    /// Providable fields by the resolvers with their requirements
    providable_fields: Vec<(BoundFieldId, &'op FieldSet)>,
}

impl<'op, 'a> PlanBoundaryChildrenPlanner<'op, 'a> {
    pub fn new(planner: &'a mut Planner<'op>, maybe_parent: Option<PlanBoundaryParent<'op, 'a, 'a>>) -> Self {
        let walker = planner.default_operation_walker();
        PlanBoundaryChildrenPlanner {
            planner,
            walker,
            maybe_parent,
            children: vec![],
        }
    }

    pub fn plan_children(
        mut self,
        missing_selection_set: FlatSelectionSetWalker<'op, '_>,
    ) -> PlanningResult<Vec<ChildPlan>> {
        // All planned fields at the boundary from the parent & children plans and any extra fields
        // added to satisfy the @requires.
        let mut boundary_selection_set = BoundarySelectionSet {
            id: missing_selection_set.id(),
            fields: self.create_boundary_selection_set_fields(),
        };

        // Fields that couldn't be provided by the parent and that have yet to be planned by one
        // child plan.
        let mut id_to_missing_fields: HashMap<BoundFieldId, MissingField<'op>> =
            build_missing_fields(self.walker.schema().as_ref(), missing_selection_set);

        // Actual planning, we plan one child plan at a time.
        let mut candidates: HashMap<ResolverId, ChildPlanCandidate<'op>> = HashMap::new();
        while !id_to_missing_fields.is_empty() {
            candidates.clear();
            self.generate_all_candidates(
                id_to_missing_fields.values(),
                &mut boundary_selection_set,
                &mut candidates,
            );

            let Some(candidate) = select_best_child_plan(&mut candidates) else {
                return Err(PlanningError::CouldNotPlanAnyField {
                    missing: id_to_missing_fields
                        .into_keys()
                        .map(|id| self.walker.walk(id).response_key_str().to_string())
                        .collect(),
                    query_path: self
                        .maybe_parent
                        .map(|parent| {
                            parent
                                .path
                                .iter_strings(&self.planner.operation.response_keys)
                                .collect()
                        })
                        .unwrap_or_default(),
                });
            };

            self.push_child_plan(&mut boundary_selection_set, &mut id_to_missing_fields, candidate)?;
        }

        // All extra fields are within the boundary_selection_set, now we need to distribute them
        // to their respective plan.
        self.attribute_extra_fields(boundary_selection_set);

        Ok(self.children)
    }

    fn create_boundary_selection_set_fields(&self) -> HashMap<FieldId, BoundaryField> {
        if let Some(ref parent) = self.maybe_parent {
            parent
                .provided_selection_set
                .group_by_field_id()
                .into_iter()
                .map(|(field_id, group)| {
                    (
                        field_id,
                        BoundaryField::Planned(PlannedBoundaryField::new(parent.plan_id, group)),
                    )
                })
                .collect()
        } else {
            HashMap::new()
        }
    }

    fn generate_all_candidates<'field>(
        &mut self,
        missing_fields: impl IntoIterator<Item = &'field MissingField<'op>>,
        boundary_selection_set: &mut BoundarySelectionSet,
        candidates: &mut HashMap<ResolverId, ChildPlanCandidate<'op>>,
    ) where
        'op: 'field,
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
                        if self.could_plan_requirements(boundary_selection_set, field_requires, &field.type_condition) {
                            candidate
                                .providable_fields
                                .push((field.flat_field.bound_field_id, field_requires));
                        }
                    }
                    Entry::Vacant(entry) => {
                        if self.could_plan_requirements(
                            boundary_selection_set,
                            &resolver.requires(),
                            &field.type_condition,
                        ) && self.could_plan_requirements(
                            boundary_selection_set,
                            field_requires,
                            &field.type_condition,
                        ) {
                            entry.insert(ChildPlanCandidate {
                                entity_type: field.entity_type,
                                resolver_id: resolver.id(),
                                providable_fields: vec![(field.flat_field.bound_field_id, field_requires)],
                            });
                        }
                    }
                }
            }
        }
    }

    fn push_child_plan(
        &mut self,
        boundary_selection_set: &mut BoundarySelectionSet,
        id_to_missing_fields: &mut HashMap<BoundFieldId, MissingField<'op>>,
        candidate: &mut ChildPlanCandidate<'op>,
    ) -> PlanningResult<()> {
        let resolver = self.walker.schema().walk(candidate.resolver_id).with_own_names();
        let (requires, providable) = {
            let mut providable = vec![];
            let mut requires = resolver.requires();
            for (id, field_requires) in std::mem::take(&mut candidate.providable_fields) {
                let flat_field = id_to_missing_fields.remove(&id).unwrap().flat_field;
                if !field_requires.is_empty() {
                    requires = Cow::Owned(FieldSet::merge(&requires, field_requires));
                }
                providable.push(flat_field);
            }
            (requires, providable)
        };
        let mut sibling_dependencies = HashSet::new();
        let input_selection_set = self.create_read_selection_set(
            &mut sibling_dependencies,
            &resolver,
            &requires,
            &mut boundary_selection_set.fields,
        )?;

        // Currently, we don't track the global execution state nor do we keep track of the
        // parent during the execution. But we know children will only be executed once the
        // parent finishes. So only keeping sibling dependencies.
        if let Some(parent) = self.maybe_parent.as_ref() {
            sibling_dependencies.remove(&parent.plan_id);
        }

        let root_selection_set = FlatSelectionSet {
            ty: candidate.entity_type,
            id: boundary_selection_set.id,
            fields: providable,
        };
        let plan_id = self.planner.next_plan_id();
        for (field_id, group) in self.walker.walk(Cow::Borrowed(&root_selection_set)).group_by_field_id() {
            boundary_selection_set
                .fields
                .entry(field_id)
                .or_insert_with(|| BoundaryField::Planned(PlannedBoundaryField::new(plan_id, group)));
        }

        self.children.push(ChildPlan {
            id: plan_id,
            resolver_id: resolver.id(),
            input_selection_set,
            root_selection_set,
            sibling_dependencies,
            // replaced later if necessary.
            extra_selection_sets: HashMap::with_capacity(0),
        });

        Ok(())
    }

    fn attribute_extra_fields(&mut self, boundary_selection_set: BoundarySelectionSet) {
        let mut plan_id_to_extra_selection_sets: HashMap<
            PlanId,
            HashMap<BoundSelectionSetId, ExtraBoundarySelectionSet>,
        > = HashMap::new();

        let mut selection_sets = vec![boundary_selection_set];
        while let Some(selection_set) = selection_sets.pop() {
            let id = BoundSelectionSetId::from(selection_set.id);
            for boundary_field in selection_set.fields.into_values() {
                match boundary_field {
                    BoundaryField::Planned(planned) => {
                        if let Some(subselection) = planned.take_subselection_if_read() {
                            selection_sets.push(subselection);
                        }
                    }
                    BoundaryField::Extra { plan_id, field, .. } => {
                        if field.read {
                            plan_id_to_extra_selection_sets
                                .entry(plan_id)
                                .or_default()
                                .entry(id)
                                .or_insert_with(|| ExtraBoundarySelectionSet {
                                    ty: self.walker.walk(id).as_ref().ty,
                                    fields: HashMap::new(),
                                })
                                .fields
                                .insert(field.extra_field.field_id, field);
                        }
                    }
                }
            }
        }

        for child in &mut self.children {
            if let Some(extra_selection_sets) = plan_id_to_extra_selection_sets.remove(&child.id) {
                child.extra_selection_sets = extra_selection_sets;
            }
        }

        if let Some(extra_selection_sets) = plan_id_to_extra_selection_sets.into_values().next() {
            self.maybe_parent
                .as_mut()
                .expect("PlanId which doesn't match any children, so should be the parent")
                .attribution
                .add_extra_selection_sets(extra_selection_sets);
        }
    }

    /// Allows us to know whether a field requirements can be provided at all to order the next child
    /// candidates.
    fn could_plan_requirements(
        &mut self,
        boundary_selection_set: &mut BoundarySelectionSet,
        requires: &FieldSet,
        type_condition: &Option<FlatTypeCondition>,
    ) -> bool {
        if requires.is_empty() {
            return true;
        }
        self.could_plan_requirements_on_previous_plans(PlanId::MAX, boundary_selection_set, requires, type_condition)
    }

    fn could_plan_requirements_on_previous_plans(
        &mut self,
        current_child_plan_id: PlanId,
        boundary_selection_set: &mut BoundarySelectionSet,
        requires: &FieldSet,
        type_condition: &Option<FlatTypeCondition>,
    ) -> bool {
        if requires.is_empty() {
            return true;
        }
        let schema = self.walker.schema();
        'requires: for item in requires {
            if let Some(field) = boundary_selection_set.fields.get_mut(&item.field_id) {
                if item.subselection.is_empty() {
                    continue;
                }
                match field {
                    BoundaryField::Planned(planned) => {
                        let Some(subselection) = planned.subselection_mut(self.walker) else {
                            return false;
                        };
                        if self.could_plan_requirements(subselection, requires, &None) {
                            continue;
                        } else {
                            return false;
                        }
                    }
                    BoundaryField::Extra { resolver_id, field, .. } => {
                        self.update_extra_field_subselection(&schema.walk(*resolver_id), field, requires);
                        continue;
                    }
                }
            } else {
                let field = schema.walk(item.field_id);
                if let Some((plan_id, resolver)) = self.maybe_parent.as_ref().and_then(|parent| {
                    // no need to check for requires here, they're only relevant when it's a
                    // plan root field and this is a nested field. So we expect the data source
                    // to be able to provide anything it needed for a nested object it provides.
                    parent
                        .logic
                        .is_providable(field)
                        .then(|| (parent.plan_id, *parent.logic.resolver()))
                }) {
                    boundary_selection_set.fields.insert(
                        item.field_id,
                        BoundaryField::Extra {
                            plan_id,
                            resolver_id: resolver.id(),
                            field: self.create_extra_field(&resolver, type_condition, item),
                        },
                    );
                    continue;
                }

                for i in 0..self.children.len() {
                    let plan_id = self.children[i].id;
                    if plan_id >= current_child_plan_id {
                        break;
                    }
                    let resolver = schema.walk(self.children[i].resolver_id);
                    let logic = AttributionLogic::CompatibleResolver {
                        resolver,
                        providable: FieldSet::default(),
                    };
                    if logic.is_providable(field)
                        && self.could_plan_requirements_on_previous_plans(
                            plan_id,
                            boundary_selection_set,
                            &resolver.requires(),
                            type_condition,
                        )
                    {
                        boundary_selection_set.fields.insert(
                            item.field_id,
                            BoundaryField::Extra {
                                plan_id,
                                resolver_id: resolver.id(),
                                field: self.create_extra_field(&resolver, type_condition, item),
                            },
                        );
                        continue 'requires;
                    }
                }
            }

            return false;
        }

        true
    }

    fn update_extra_field_subselection(
        &mut self,
        resolver: &ResolverWalker<'_>,
        extra_boundary_field: &mut ExtraBoundaryField,
        field_set: &FieldSet,
    ) {
        let ExpectedType::SelectionSet(ref mut selection_set) = extra_boundary_field.extra_field.ty else {
            return;
        };
        for item in field_set {
            match selection_set.fields.entry(item.field_id) {
                Entry::Occupied(mut entry) => {
                    self.update_extra_field_subselection(resolver, entry.get_mut(), &item.subselection);
                }
                Entry::Vacant(entry) => {
                    entry.insert(self.create_extra_field(resolver, &None, item));
                }
            }
        }
    }

    fn create_extra_field(
        &mut self,
        resolver: &ResolverWalker<'_>,
        type_condition: &Option<FlatTypeCondition>,
        item: &FieldSetItem,
    ) -> ExtraBoundaryField {
        let field = resolver.walk(item.field_id);
        ExtraBoundaryField {
            read: false,
            extra_field: ExtraField {
                edge: item.field_id.into(),
                type_condition: type_condition.clone(),
                field_id: item.field_id,
                expected_key: {
                    if resolver.supports_aliases() {
                        // When the resolver supports aliases, we must ensure that extra fields
                        // don't collide with existing response keys. And to avoid duplicates
                        // during field collection, we have a single unique name per field id.
                        self.planner.get_extra_field_name(item.field_id)
                    } else {
                        field.name().to_string()
                    }
                },
                ty: {
                    let definition = field.ty().inner();
                    definition.data_type().map(ExpectedType::Scalar).unwrap_or_else(|| {
                        ExpectedType::SelectionSet(ExtraBoundarySelectionSet {
                            ty: SelectionSetType::maybe_from(definition.id()).expect("not a scalar"),
                            fields: item
                                .subselection
                                .iter()
                                .map(|item| {
                                    let field = self.create_extra_field(resolver, &None, item);
                                    (item.field_id, field)
                                })
                                .collect(),
                        })
                    })
                },
            },
        }
    }

    /// Create the the input selection set of a Plan given its resolver and requirements.
    /// We iterate over the requirements and find the matching fields inside the boundary fields,
    /// which contains all providable & extra fields. During the iteration we track all the dependency
    /// plans.
    fn create_read_selection_set(
        &self,
        dependencies: &mut HashSet<PlanId>,
        resolver: &ResolverWalker<'_>,
        requires: &FieldSet,
        boundary_fields: &mut HashMap<FieldId, BoundaryField>,
    ) -> PlanningResult<ReadSelectionSet> {
        if requires.is_empty() {
            return Ok(ReadSelectionSet::default());
        }
        requires
            .iter()
            .map(|item| {
                match boundary_fields
                    .get_mut(&item.field_id)
                    .expect("field should be present, we could plan it")
                {
                    BoundaryField::Planned(planned) => {
                        dependencies.insert(planned.plan_id);
                        Ok(ReadField {
                            edge: planned.field.key.into(),
                            name: resolver.walk(item.field_id).name().to_string(),
                            subselection: if item.subselection.is_empty() {
                                ReadSelectionSet::default()
                            } else {
                                self.create_read_selection_set(
                                    dependencies,
                                    resolver,
                                    &item.subselection,
                                    &mut planned
                                        .subselection_mut(self.walker)
                                        .expect("subselection should be present, we could plan the subselection")
                                        .fields,
                                )?
                            },
                        })
                    }
                    BoundaryField::Extra { plan_id, field, .. } => {
                        dependencies.insert(*plan_id);
                        field.read = true;
                        Ok(ReadField {
                            edge: field.extra_field.edge,
                            name: resolver.walk(item.field_id).name().to_string(),
                            subselection: create_read_selection_set_from_extras(
                                resolver,
                                &item.subselection,
                                &mut field.extra_field.ty,
                            )?,
                        })
                    }
                }
            })
            .collect::<PlanningResult<ReadSelectionSet>>()
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

struct MissingField<'op> {
    entity_type: EntityType,
    flat_field: FlatField,
    type_condition: Option<FlatTypeCondition>,
    field_resolvers: Vec<FieldResolverWalker<'op>>,
}

fn build_missing_fields<'op>(
    schema: &Schema,
    missing_selection_set: FlatSelectionSetWalker<'op, '_>,
) -> HashMap<BoundFieldId, MissingField<'op>> {
    let selection_set_type = missing_selection_set.ty();
    missing_selection_set
        .into_fields()
        .map(|flat_field_walker| {
            let entity_type = flat_field_walker.entity_type();
            let field_resolvers = flat_field_walker
                .bound_field()
                .schema_field()
                .expect("Meta fields are always providable, it can't be missing.")
                .resolvers()
                .collect::<Vec<_>>();

            let flat_field = flat_field_walker.into_item();
            (
                flat_field.bound_field_id,
                MissingField {
                    entity_type,
                    flat_field,
                    // Parent selection might be a union/interface and current resolver
                    // apply on a object.
                    type_condition: FlatTypeCondition::flatten(schema, selection_set_type, vec![entity_type.into()]),
                    field_resolvers,
                },
            )
        })
        .collect()
}

fn create_read_selection_set_from_extras(
    resolver: &ResolverWalker<'_>,
    requires: &FieldSet,
    parent_ty: &mut ExpectedType<ExtraBoundarySelectionSet>,
) -> PlanningResult<ReadSelectionSet> {
    let ExpectedType::SelectionSet(ref mut selection_set) = parent_ty else {
        return Ok(ReadSelectionSet::default());
    };
    if requires.is_empty() {
        return Ok(ReadSelectionSet::default());
    }

    requires
        .iter()
        .map(|item| {
            let field = selection_set
                .fields
                .get_mut(&item.field_id)
                .expect("field should be present");
            field.read = true;
            let subselection =
                create_read_selection_set_from_extras(resolver, &item.subselection, &mut field.extra_field.ty)?;
            Ok(ReadField {
                edge: field.extra_field.edge,
                name: resolver.walk(item.field_id).name().to_string(),
                subselection,
            })
        })
        .collect()
}
