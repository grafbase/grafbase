use std::{
    borrow::Cow,
    cell::Cell,
    collections::{HashMap, HashSet},
};

use engine_parser::types::OperationType;
use itertools::Itertools;
use schema::{FieldResolverWalker, FieldSet, FieldSetItem, FieldWalker, ResolverId, ResolverWalker, Schema};

use crate::{
    plan::{
        attribution::AttributionBuilder, ChildPlan, CollectedSelectionSet, ConcreteField, ConcreteType, EntityType,
        ExpectedSelectionSet, ExtraSelectionSetId, FlatTypeCondition, Plan, PlanBoundary, PlanBoundaryId, PlanId,
        PlanInput, PlanOutput, PossibleField, UndeterminedSelectionSet,
    },
    request::{
        BoundFieldDefinitionWalker, BoundFieldId, BoundSelectionSetId, FlatField, FlatFieldWalker, FlatSelectionSet,
        FlatSelectionSetWalker, Operation, OperationWalker, QueryPath, SelectionSetType,
    },
    response::{ReadField, ReadSelectionSet, ResponseBoundaryItem, ResponseEdge},
};

use super::{ExpectationsBuilder, ExpectedField, ExpectedType, UndeterminedSelectionSetId};

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Could not plan fields: {}", .missing.join(", "))]
    CouldNotPlanAnyField { missing: Vec<String> },
    #[error("Could not satisfy required field named '{field}' for resolver named '{resolver}'")]
    CouldNotSatisfyRequires { resolver: String, field: String },
}

pub type PlanningResult<T> = Result<T, PlanningError>;

pub struct Planner<'op> {
    schema: &'op Schema,
    operation: &'op Operation,
    next_plan_id: Cell<usize>,
}

impl<'op> Planner<'op> {
    pub fn new(schema: &'op Schema, operation: &'op Operation) -> Self {
        Planner {
            schema,
            operation,
            next_plan_id: Cell::new(0),
        }
    }

    pub fn generate_root_plan_boundary(&mut self) -> PlanningResult<PlanBoundary> {
        let walker = self.default_operation_walker();
        let flat_selection_set = walker.flatten_selection_sets(vec![self.operation.root_selection_set_id]);

        // The default resolver is the introspection one which allows use deal nicely with queries
        // like `query { __typename }`. So all fields without a resolvers are considered to be providable by introspection.
        let (providable, missing) = flat_selection_set.partition_fields(|flat_field| {
            flat_field
                .bound_field()
                .definition()
                .as_field()
                .map(|field| field.resolvers().len() == 0)
                .unwrap_or(true)
        });
        let mut boundary = if matches!(self.operation.ty, OperationType::Mutation) {
            self.create_mutation_plan_boundary(missing)?
        } else {
            self.create_plan_boundary(None, missing)?
        };

        // Are there actually any introspection related fields?
        if !providable.is_empty() {
            let resolver_id = self.schema.introspection_resolver_id();
            boundary.children.push(ChildPlan {
                id: self.next_plan_id(),
                resolver_id,
                input_selection_set: ReadSelectionSet::default(),
                root_selection_set: {
                    let flat_selection_set = providable.into_inner();
                    FlatSelectionSet {
                        ty: EntityType::Object(self.operation.root_object_id),
                        id: flat_selection_set.id,
                        fields: flat_selection_set.fields,
                    }
                },
            });
        }
        Ok(boundary)
    }

    pub fn generate_subscription_plan(&mut self, mut boundary: PlanBoundary) -> PlanningResult<Plan> {
        assert!(boundary.children.len() == 1);
        let child = boundary.children.pop().unwrap();

        let id = child.id;
        let resolver_id = child.resolver_id;

        self.create_plan_output(&boundary.query_path, child)
            .map(|(output, boundaries)| Plan {
                id,
                resolver_id,
                input: None,
                output,
                boundaries,
            })
    }

    pub fn generate_plans(
        &mut self,
        boundary: PlanBoundary,
        response_boundary: &[ResponseBoundaryItem],
    ) -> PlanningResult<Vec<Plan>> {
        if response_boundary.is_empty() {
            return Ok(vec![]);
        }
        boundary
            .children
            .into_iter()
            .filter_map(|mut child| {
                // we could certainly be smarter and avoids copies with an Arc.
                let response_boundary = match (boundary.selection_set_type, child.root_selection_set.ty) {
                    (SelectionSetType::Object(_), _) => response_boundary.to_owned(),
                    (SelectionSetType::Interface(a), EntityType::Interface(b)) if a == b => {
                        response_boundary.to_owned()
                    }
                    (_, EntityType::Interface(id)) => {
                        let possible_types = &self.schema[id].possible_types;
                        response_boundary
                            .iter()
                            .filter(|root| possible_types.binary_search(&root.object_id).is_ok())
                            .cloned()
                            .collect()
                    }
                    (_, EntityType::Object(id)) => response_boundary
                        .iter()
                        .filter(|root| root.object_id == id)
                        .cloned()
                        .collect(),
                };
                if response_boundary.is_empty() {
                    None
                } else {
                    let id = child.id;
                    let resolver_id = child.resolver_id;
                    let input = Some(PlanInput {
                        response_boundary,
                        selection_set: std::mem::take(&mut child.input_selection_set),
                    });
                    Some(
                        self.create_plan_output(&boundary.query_path, child)
                            .map(|(output, boundaries)| Plan {
                                id,
                                resolver_id,
                                input,
                                output,
                                boundaries,
                            }),
                    )
                }
            })
            .collect()
    }

    fn next_plan_id(&self) -> PlanId {
        let current = self.next_plan_id.get();
        let id = PlanId::from(current);
        self.next_plan_id.set(current + 1);
        id
    }

    fn create_plan_boundary(
        &mut self,
        mut maybe_parent: Option<PlanBoundaryParent<'op, '_, '_>>,
        missing_selection_set: FlatSelectionSetWalker<'_>,
    ) -> PlanningResult<PlanBoundary> {
        let walker = self.default_operation_walker();
        let selection_set_type = missing_selection_set.ty();
        let flat_selection_set_id = missing_selection_set.id();
        let mut id_to_missing_fields: HashMap<BoundFieldId, FlatField> = missing_selection_set
            .into_fields()
            .map(|flat_field| (flat_field.bound_field_id, flat_field.into_inner()))
            .collect();

        let mut children = Vec::new();
        'candidates_generation: while !id_to_missing_fields.is_empty() {
            let count = id_to_missing_fields.len();
            pub struct ChildPlanCandidate<'a> {
                entity_type: EntityType,
                resolver: ResolverWalker<'a>,
                fields: Vec<(BoundFieldId, &'a FieldSet)>,
            }

            let mut candidates = HashMap::<ResolverId, ChildPlanCandidate<'_>>::new();
            for field in id_to_missing_fields.values() {
                for FieldResolverWalker {
                    resolver,
                    field_requires: requires,
                } in walker
                    .walk(field.bound_field_id)
                    .definition()
                    .as_field()
                    .expect("Meta fields are always providable so a missing field can't be one.")
                    .resolvers()
                {
                    let candidate = candidates.entry(resolver.id()).or_insert_with(|| {
                        let entity_type = match self.operation[*field.selection_set_path.last().unwrap()].ty {
                            SelectionSetType::Object(id) => EntityType::Object(id),
                            SelectionSetType::Interface(id) => EntityType::Interface(id),
                            SelectionSetType::Union(_) => {
                                unreachable!("not a meta field and only interfaces & objects can have fields")
                            }
                        };
                        ChildPlanCandidate {
                            // We don't need to care about type conditions here as the resolver naturally
                            // enforces that only fields of the same interface/object can be grouped
                            // together.
                            entity_type,
                            resolver,
                            fields: Vec::new(),
                        }
                    });
                    candidate.fields.push((field.bound_field_id, requires));
                }
            }
            let mut candidates = candidates.into_values().collect::<Vec<_>>();
            candidates.sort_unstable_by_key(|candidate| std::cmp::Reverse(candidate.fields.len()));
            for candidate in candidates {
                // if we just planned any fields, we're ensuring there is no intersection with the
                // previous candidate. Otherwise we regenerate the candidates as their ordering is
                // incorrect now.
                if id_to_missing_fields.len() < count {
                    for (id, _) in &candidate.fields {
                        if !id_to_missing_fields.contains_key(id) {
                            continue 'candidates_generation;
                        }
                    }
                }

                let resolver = candidate.resolver.with_own_names();
                let mut providable = vec![];
                let mut requires = resolver.requires();
                for (id, field_requires) in candidate.fields {
                    let flat_field = id_to_missing_fields.remove(&id).unwrap();
                    if !field_requires.is_empty() {
                        requires = Cow::Owned(FieldSet::merge(&requires, field_requires));
                    }
                    providable.push(flat_field);
                }
                children.push(ChildPlan {
                    id: self.next_plan_id(),
                    resolver_id: candidate.resolver.id(),
                    input_selection_set: maybe_parent
                        .as_mut()
                        .map(|parent| {
                            // Parent selection might be a union/interface and current resolver
                            // apply on a object. It doesn't matter for the ReadSelectionSet
                            // directly as it'll only read data from response objects that are
                            // relevant to the plan. But it does matter for any extra fields we may
                            // add to the parent.
                            let type_condition = FlatTypeCondition::flatten(
                                self.schema,
                                selection_set_type,
                                vec![candidate.entity_type.into()],
                            );
                            self.create_read_selection_set(parent, resolver, type_condition, &requires)
                        })
                        .transpose()?
                        .unwrap_or_default(),
                    root_selection_set: FlatSelectionSet {
                        ty: candidate.entity_type,
                        id: flat_selection_set_id,
                        fields: providable,
                    },
                });
            }

            // No fields were planned
            if count == id_to_missing_fields.len() {
                return Err(PlanningError::CouldNotPlanAnyField {
                    missing: id_to_missing_fields
                        .into_keys()
                        .map(|id| walker.walk(id).response_key_str().to_string())
                        .collect(),
                });
            }
        }
        Ok(PlanBoundary {
            query_path: maybe_parent
                .as_ref()
                .map(|parent| parent.path.clone())
                .unwrap_or_default(),
            selection_set_type,
            children,
        })
    }

    /// Mutation fields need to be executed sequentially. So instead of grouping fields by
    /// resolvers, we create a plan for each field in the order the fields appear in.
    fn create_mutation_plan_boundary(
        &mut self,
        missing_selection_set: FlatSelectionSetWalker<'_>,
    ) -> PlanningResult<PlanBoundary> {
        let walker = self.default_operation_walker();
        let selection_set_type = missing_selection_set.ty();
        let flat_selection_set_id = missing_selection_set.id();
        let entity_type = EntityType::Object(self.operation.root_object_id);

        let mut groups = missing_selection_set
            .group_by_response_key()
            .into_values()
            .collect::<Vec<_>>();

        // Ordering groups by their position in the query, ensuring proper ordering of plans.
        groups.sort_unstable_by(|a, b| a.key.cmp(&b.key));

        let children = groups
            .into_iter()
            .map(|group| {
                let FieldResolverWalker {
                    resolver,
                    field_requires: requires,
                } = walker
                    .walk(group.definition_id)
                    .as_field()
                    .expect("Introspection resolver should have taken metadata fields")
                    .resolvers()
                    .next()
                    .ok_or_else(|| PlanningError::CouldNotPlanAnyField {
                        missing: vec![walker.walk(group.bound_field_ids[0]).response_key_str().to_string()],
                    })?;
                if !requires.is_empty() {
                    return Err(PlanningError::CouldNotSatisfyRequires {
                        resolver: resolver.name().to_string(),
                        field: requires
                            .into_iter()
                            .map(|item| walker.schema().walk(item.field_id).name())
                            .collect(),
                    });
                }
                Ok(ChildPlan {
                    id: self.next_plan_id(),
                    resolver_id: resolver.id(),
                    input_selection_set: ReadSelectionSet::default(),
                    root_selection_set: FlatSelectionSet {
                        ty: entity_type,
                        id: flat_selection_set_id,
                        fields: group
                            .bound_field_ids
                            .into_iter()
                            .map(|id| FlatField {
                                type_condition: None,
                                selection_set_path: vec![flat_selection_set_id.into()],
                                bound_field_id: id,
                            })
                            .collect(),
                    },
                })
            })
            .collect::<PlanningResult<Vec<_>>>()?;

        Ok(PlanBoundary {
            selection_set_type,
            query_path: QueryPath::default(),
            children,
        })
    }

    fn create_plan_output(
        &mut self,
        path: &QueryPath,
        ChildPlan {
            resolver_id,
            root_selection_set: providable,
            ..
        }: ChildPlan,
    ) -> PlanningResult<(PlanOutput, Vec<PlanBoundary>)> {
        let resolver = self.schema.walker().walk(resolver_id).with_own_names();
        let walker = self.operation.walker_with(resolver.walk(()));
        let flat_selection_set: FlatSelectionSetWalker<'_, EntityType> = walker.walk(Cow::Owned(providable));
        let entity_type = flat_selection_set.ty();

        let mut boundaries = Vec::new();
        let mut attribution = AttributionBuilder::new(&self.operation.response_keys, resolver);
        let mut expectations = ExpectationsBuilder::default();
        let mut builder = PlanOutputBuilderContext {
            planner: self,
            path: path.clone(),
            walker,
            resolver,
            logic: AttributionLogic::CompatibleResolver {
                resolver,
                providable: FieldSet::default(),
            },
            attribution: &mut attribution,
            boundaries: &mut boundaries,
            expectations: &mut expectations,
        };
        let fields = flat_selection_set
            .fields()
            .map(|flat_field| flat_field.bound_field_id)
            .collect();
        let root_selection_set = builder.collect_fields(flat_selection_set, None)?;
        Ok((
            PlanOutput {
                entity_type,
                root_fields: fields,
                attribution: attribution.build(),
                expectations: expectations.build(root_selection_set),
            },
            boundaries,
        ))
    }

    fn create_read_selection_set(
        &mut self,
        parent: &mut PlanBoundaryParent<'op, '_, '_>,
        resolver: ResolverWalker<'_>,
        type_condition: Option<FlatTypeCondition>,
        requires: &FieldSet,
    ) -> PlanningResult<ReadSelectionSet> {
        if requires.is_empty() {
            return Ok(ReadSelectionSet::default());
        }
        let mut groups = parent.flat_selection_set.group_by_field_id();

        let (providable, missing): (Vec<_>, Vec<_>) = requires
            .iter()
            .map(|item| {
                if let Some(group) = groups.remove(&item.field_id) {
                    Ok((item, group))
                } else {
                    Err(item)
                }
            })
            .partition_result();

        let mut read_selection_set = providable
            .into_iter()
            .map(|(item, group)| {
                if !parent.logic.is_providable(*group.field) {
                    return Err(PlanningError::CouldNotSatisfyRequires {
                        resolver: resolver.name().to_string(),
                        // We want the actual schema name here.
                        field: self.schema.walker().walk(item.field_id).name().to_string(),
                    });
                }

                let subselection = if item.subselection.is_empty() {
                    ReadSelectionSet::default()
                } else {
                    let flat_selection_set = self
                        .default_operation_walker()
                        .merged_selection_sets(&group.bound_field_ids);
                    self.create_read_selection_set(
                        &mut parent.child(*group.field, flat_selection_set),
                        resolver,
                        None,
                        &item.subselection,
                    )?
                };

                Ok(ReadField {
                    edge: group.key.into(),
                    // We want the resolver's Names here.
                    name: resolver.walk(group.field.id()).name().to_string(),
                    subselection,
                })
            })
            .collect::<PlanningResult<ReadSelectionSet>>()?;

        if !missing.is_empty() {
            let id = BoundSelectionSetId::from(parent.flat_selection_set.id());
            // If it wasn't done already, we're now attributing this selection_set to the parent
            // plan as it'll contain at least one extra field.
            parent.attribution.attributed_selection_sets.insert(id);
            let extra_selection_set_id = parent.attribution.extra_selection_set_for(id, self.operation[id].ty);
            read_selection_set.extend_disjoint(self.create_extra_read_selection_set(
                parent.attribution,
                extra_selection_set_id,
                parent.logic.clone(),
                resolver,
                type_condition,
                missing,
            )?);
        }
        Ok(read_selection_set)
    }

    fn create_extra_read_selection_set<'a>(
        &mut self,
        parent_attribution: &mut AttributionBuilder<'op>,
        extra_selection_set_id: ExtraSelectionSetId,
        logic: AttributionLogic<'_>,
        resolver: ResolverWalker<'_>,
        type_condition: Option<FlatTypeCondition>,
        missing: impl IntoIterator<Item = &'a FieldSetItem>,
    ) -> PlanningResult<ReadSelectionSet> {
        missing
            .into_iter()
            .map(|missing_item| {
                let field = resolver.walk(missing_item.field_id);
                if !logic.is_providable(field) {
                    return Err(PlanningError::CouldNotSatisfyRequires {
                        resolver: resolver.name().to_string(),
                        // We want the actual schema name here.
                        field: self.schema.walker().walk(field.id()).name().to_string(),
                    });
                }
                let extra_field = parent_attribution.get_or_insert_extra_field_with(
                    extra_selection_set_id,
                    type_condition.as_ref(),
                    field.id(),
                );
                Ok(ReadField {
                    edge: extra_field.edge,
                    // We want the name that the resolver expects here.
                    name: field.name().to_string(),
                    subselection: match extra_field.ty {
                        ExpectedType::SelectionSet(subselection_id) => self.create_extra_read_selection_set(
                            parent_attribution,
                            subselection_id,
                            logic.child(field),
                            resolver,
                            None,
                            &missing_item.subselection,
                        )?,
                        _ => ReadSelectionSet::default(),
                    },
                })
            })
            .collect()
    }

    fn default_operation_walker(&self) -> OperationWalker<'op> {
        self.operation.walker_with(self.schema.walker())
    }
}

struct PlanBoundaryParent<'op, 'plan, 'ctx> {
    path: &'ctx QueryPath,
    logic: AttributionLogic<'op>,
    attribution: &'plan mut AttributionBuilder<'op>,
    flat_selection_set: FlatSelectionSetWalker<'op>,
}

impl<'op, 'plan, 'ctx> PlanBoundaryParent<'op, 'plan, 'ctx> {
    fn child<'s>(
        &'s mut self,
        field: FieldWalker<'op>,
        flat_selection_set: FlatSelectionSetWalker<'op>,
    ) -> PlanBoundaryParent<'op, 's, 'ctx> {
        PlanBoundaryParent {
            path: self.path,
            logic: self.logic.child(field),
            attribution: self.attribution,
            flat_selection_set,
        }
    }
}

struct PlanOutputBuilderContext<'op, 'plan> {
    planner: &'plan mut Planner<'op>,
    path: QueryPath,
    walker: OperationWalker<'op>,
    resolver: ResolverWalker<'op>,
    logic: AttributionLogic<'op>,
    attribution: &'plan mut AttributionBuilder<'op>,
    expectations: &'plan mut ExpectationsBuilder,
    boundaries: &'plan mut Vec<PlanBoundary>,
}

#[derive(Debug, Clone)]
enum AttributionLogic<'a> {
    /// Having a resolver in the same group or having no resolver at all.
    CompatibleResolver {
        resolver: ResolverWalker<'a>,
        providable: FieldSet,
    },
    /// Only an explicitly providable (@provide) field can be attributed. This is an optimization
    /// overriding the CompatibleResolver logic
    OnlyProvidable { providable: FieldSet },
}

impl<'a> AttributionLogic<'a> {
    fn is_providable(&self, field: FieldWalker<'_>) -> bool {
        match self {
            AttributionLogic::CompatibleResolver { resolver, providable } => {
                let providable_field = providable.get(field.id());
                providable_field.is_some() || resolver.can_provide(field)
            }
            AttributionLogic::OnlyProvidable { providable } => providable.get(field.id()).is_some(),
        }
    }

    fn child(&self, field: FieldWalker<'_>) -> Self {
        match self {
            AttributionLogic::CompatibleResolver { resolver, providable } => {
                let providable = FieldSet::merge_opt(
                    providable.get(field.id()).map(|s| &s.subselection),
                    field.provides_for(resolver).as_deref(),
                );
                if resolver.can_provide(field) {
                    AttributionLogic::CompatibleResolver {
                        resolver: *resolver,
                        providable,
                    }
                } else {
                    AttributionLogic::OnlyProvidable { providable }
                }
            }
            AttributionLogic::OnlyProvidable { providable } => AttributionLogic::OnlyProvidable {
                providable: providable
                    .get(field.id())
                    .map(|s| &s.subselection)
                    .cloned()
                    .unwrap_or_default(),
            },
        }
    }
}

impl<'op, 'plan> PlanOutputBuilderContext<'op, 'plan> {
    fn expected_selection_set(&mut self, bound_field_ids: Vec<BoundFieldId>) -> PlanningResult<ExpectedSelectionSet> {
        let (providable, maybe_boundary_id) = {
            let flat_selection_set = self.walker.merged_selection_sets(&bound_field_ids);
            self.partition_providable_missing(flat_selection_set)?
        };
        let mut conditions = HashSet::<Option<EntityType>>::new();
        let mut too_complex = false;
        for field in providable.fields() {
            match &field.type_condition {
                Some(type_condition) => match type_condition {
                    FlatTypeCondition::Interface(id) => {
                        conditions.insert(Some(EntityType::Interface(*id)));
                    }
                    FlatTypeCondition::Objects(ids) => {
                        if ids.len() == 1 {
                            conditions.insert(Some(EntityType::Object(ids[0])));
                        } else {
                            too_complex = true;
                        }
                    }
                },
                None => {
                    conditions.insert(None);
                }
            }
        }

        if !too_complex && conditions == HashSet::from([None]) {
            self.collect_fields(providable, maybe_boundary_id)
                .map(ExpectedSelectionSet::Collected)
        } else {
            self.expected_undetermined_selection_set(providable, maybe_boundary_id)
                .map(ExpectedSelectionSet::Undetermined)
        }
    }

    fn expected_undetermined_nested_selection_set(
        &mut self,
        flat_field: &FlatField,
    ) -> PlanningResult<UndeterminedSelectionSetId> {
        let (providable, maybe_boundary_id) = {
            let flat_selection_set = self.walker.merged_selection_sets(&[flat_field.bound_field_id]);
            self.partition_providable_missing(flat_selection_set)?
        };
        self.expected_undetermined_selection_set(providable, maybe_boundary_id)
    }

    fn partition_providable_missing(
        &mut self,
        flat_selection_set: FlatSelectionSetWalker<'op>,
    ) -> PlanningResult<(FlatSelectionSetWalker<'op>, Option<PlanBoundaryId>)> {
        let (providable, missing) = flat_selection_set.partition_fields(|flat_field| {
            flat_field
                .bound_field()
                .definition()
                .as_field()
                .map(|field| self.logic.is_providable(*field))
                .unwrap_or(true)
        });

        let maybe_boundary_id = if missing.is_empty() {
            None
        } else {
            let boundary = self.planner.create_plan_boundary(
                Some(PlanBoundaryParent {
                    path: &self.path,
                    logic: self.logic.clone(),
                    attribution: self.attribution,
                    flat_selection_set: providable.clone(),
                }),
                missing,
            )?;
            Some(self.push_boundary(boundary))
        };

        Ok((providable, maybe_boundary_id))
    }

    fn collect_fields<Ty: Copy + Into<SelectionSetType> + std::fmt::Debug>(
        &mut self,
        flat_selection_set: FlatSelectionSetWalker<'op, Ty>,
        maybe_boundary_id: Option<PlanBoundaryId>,
    ) -> PlanningResult<CollectedSelectionSet> {
        let mut fields = vec![];
        let mut typename_fields = vec![];
        for group in flat_selection_set.group_by_response_key().into_values() {
            self.attribution.attributed_fields.extend(&group.bound_field_ids);
            self.attribution
                .attributed_selection_sets
                .extend(group.origin_selection_set_ids);
            if let Some(field) = self.walker.walk(group.definition_id).as_field() {
                let expected_key = if self.resolver.supports_aliases() {
                    field.response_key_str().to_string()
                } else {
                    field.name().to_string()
                };
                let ty = match field.ty().inner().data_type() {
                    Some(data_type) => ConcreteType::Scalar(data_type),
                    None => {
                        ConcreteType::SelectionSet(self.child(field).expected_selection_set(group.bound_field_ids)?)
                    }
                };
                fields.push(ConcreteField {
                    expected_key,
                    edge: group.key.into(),
                    definition_id: Some(group.definition_id),
                    wrapping: field.ty().wrapping().clone(),
                    ty,
                });
            } else {
                typename_fields.push(group.key.into());
            }
        }
        if let Some(extra_fields) = self.attribution.extra_fields(flat_selection_set.id().into()) {
            fields.extend(extra_fields.map(|extra_field| ConcreteField {
                edge: extra_field.edge,
                expected_key: extra_field.expected_key.clone(),
                definition_id: None,
                ty: match extra_field.ty {
                    ExpectedType::Scalar(data_type) => ConcreteType::Scalar(data_type),
                    ExpectedType::SelectionSet(id) => {
                        ConcreteType::ExtraSelectionSet(self.attribution[id].clone().build())
                    }
                },
                wrapping: self.walker.schema().walk(extra_field.field_id).ty().wrapping().clone(),
            }));
        }
        fields.sort_unstable_by(|a, b| a.expected_key.cmp(&b.expected_key));
        Ok(CollectedSelectionSet {
            ty: flat_selection_set.ty().into(),
            boundary_ids: maybe_boundary_id.into_iter().collect(),
            fields,
            typename_fields,
        })
    }

    fn expected_undetermined_selection_set(
        &mut self,
        flat_selection_set: FlatSelectionSetWalker<'op>,
        maybe_boundary_id: Option<PlanBoundaryId>,
    ) -> PlanningResult<UndeterminedSelectionSetId> {
        let ty = flat_selection_set.ty();
        let id = BoundSelectionSetId::from(flat_selection_set.id());
        let mut fields = flat_selection_set
            .into_fields()
            .map(|flat_field| self.expected_ungrouped_field(flat_field))
            .collect::<PlanningResult<Vec<_>>>()?;

        if let Some(extra_field_ids) = self.attribution.extra_field_ids(id) {
            fields.extend(extra_field_ids.map(PossibleField::Extra));
        }
        // Sorting by the ResponseEdge, so field position in the query, ensures proper field
        // collection later.
        fields.sort_unstable_by_key(|field| match field {
            PossibleField::TypeName { key, .. } => ResponseEdge::from(*key),
            PossibleField::Query(id) => self.expectations[*id].bound_response_key.into(),
            PossibleField::Extra(id) => self.attribution[*id].edge,
        });
        Ok(self
            .expectations
            .push_ungrouped_selection_set(UndeterminedSelectionSet {
                ty,
                maybe_boundary_id,
                fields,
            }))
    }

    fn expected_ungrouped_field(&mut self, flat_field: FlatFieldWalker<'_>) -> PlanningResult<PossibleField> {
        self.attribution.attributed_fields.push(flat_field.bound_field_id);
        self.attribution
            .attributed_selection_sets
            .extend(flat_field.selection_set_path.clone());
        let bound_field = flat_field.bound_field();

        if let Some(field) = bound_field.definition().as_field() {
            let expected_key = if self.resolver.supports_aliases() {
                field.response_key_str().to_string()
            } else {
                field.name().to_string()
            };
            let ty = match field.ty().inner().data_type() {
                Some(data_type) => ExpectedType::Scalar(data_type),
                None => ExpectedType::SelectionSet(
                    self.child(field)
                        .expected_undetermined_nested_selection_set(&flat_field)?,
                ),
            };
            let bound_field = flat_field.bound_field();
            Ok(PossibleField::Query(self.expectations.push_field(ExpectedField {
                type_condition: flat_field.into_inner().type_condition,
                expected_key,
                ty,
                field_id: field.id(),
                bound_response_key: bound_field.bound_response_key(),
                definition_id: bound_field.definition_id(),
            })))
        } else {
            Ok(PossibleField::TypeName {
                type_condition: flat_field.into_inner().type_condition,
                key: bound_field.bound_response_key(),
            })
        }
    }

    fn push_boundary(&mut self, boundary: PlanBoundary) -> PlanBoundaryId {
        let id = PlanBoundaryId::from(self.boundaries.len());
        self.boundaries.push(boundary);
        id
    }

    fn child<'s>(&'s mut self, field: BoundFieldDefinitionWalker<'_>) -> PlanOutputBuilderContext<'op, 's> {
        PlanOutputBuilderContext {
            planner: self.planner,
            path: self.path.child(field.response_key()),
            walker: self.walker,
            resolver: self.resolver,
            logic: self.logic.child(*field),
            attribution: self.attribution,
            boundaries: self.boundaries,
            expectations: self.expectations,
        }
    }
}
