use std::{
    borrow::Cow,
    cell::Cell,
    collections::{BTreeMap, HashMap, HashSet},
};

use engine_parser::types::OperationType;
use schema::{FieldId, FieldResolverWalker, FieldSet, FieldWalker, ResolverWalker, Schema};

use crate::{
    plan::{
        attribution::AttributionBuilder, ChildPlan, CollectedSelectionSet, ConcreteField, ConcreteType, EntityType,
        ExpectedSelectionSet, FlatTypeCondition, Plan, PlanBoundary, PlanBoundaryId, PlanId, PlanInput, PlanOutput,
        PossibleField, UndeterminedSelectionSet,
    },
    request::{
        BoundFieldId, BoundFieldWalker, BoundSelectionSetId, FlatField, FlatFieldWalker, FlatSelectionSet,
        FlatSelectionSetWalker, Operation, OperationWalker, QueryPath, SelectionSetType,
    },
    response::{GraphqlError, ReadSelectionSet, ResponseBoundaryItem, ResponseEdge},
};

use boundary_planner::{PlanBoundaryChildrenPlanner, PlanBoundaryParent};
pub(super) use boundary_selection_set::ExtraBoundarySelectionSet;

use super::{ExpectationsBuilder, ExpectedField, ExpectedType, UndeterminedSelectionSetId};

mod boundary_planner;
mod boundary_selection_set;

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Could not plan fields: {}", .missing.join(", "))]
    CouldNotPlanAnyField {
        missing: Vec<String>,
        query_path: Vec<String>,
    },
    #[error("Could not satisfy required field named '{field}' for resolver named '{resolver}'")]
    CouldNotSatisfyRequires { resolver: String, field: String },
}

impl From<PlanningError> for GraphqlError {
    fn from(error: PlanningError) -> Self {
        let message = error.to_string();
        let query_path = match error {
            PlanningError::CouldNotPlanAnyField { query_path, .. } => query_path
                .into_iter()
                .map(serde_json::Value::String)
                .collect::<Vec<_>>(),
            PlanningError::CouldNotSatisfyRequires { .. } => vec![],
        };

        GraphqlError {
            message,
            locations: vec![],
            path: None,
            extensions: BTreeMap::from([("queryPath".into(), serde_json::Value::Array(query_path))]),
        }
    }
}

pub type PlanningResult<T> = Result<T, PlanningError>;

pub struct Planner<'op> {
    schema: &'op Schema,
    operation: &'op Operation,
    next_plan_id: Cell<usize>,
    extra_field_names: HashMap<FieldId, String>,
}

impl<'op> Planner<'op> {
    pub fn new(schema: &'op Schema, operation: &'op Operation) -> Self {
        Planner {
            schema,
            operation,
            next_plan_id: Cell::new(0),
            extra_field_names: HashMap::new(),
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
                .schema_field()
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
                sibling_dependencies: HashSet::with_capacity(0),
                extra_selection_sets: HashMap::with_capacity(0),
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
            .map(|(sibling_dependencies, output, boundaries)| Plan {
                id,
                resolver_id,
                sibling_dependencies,
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
                    Some(self.create_plan_output(&boundary.query_path, child).map(
                        |(sibling_dependencies, output, boundaries)| Plan {
                            id,
                            resolver_id,
                            sibling_dependencies,
                            input,
                            output,
                            boundaries,
                        },
                    ))
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
        maybe_parent: Option<PlanBoundaryParent<'op, '_, '_>>,
        missing_selection_set: FlatSelectionSetWalker<'op, '_>,
    ) -> PlanningResult<PlanBoundary> {
        let query_path = maybe_parent
            .as_ref()
            .map(|parent| parent.path.clone())
            .unwrap_or_default();
        let selection_set_type = missing_selection_set.ty();
        let children = PlanBoundaryChildrenPlanner::new(self, maybe_parent).plan_children(missing_selection_set)?;
        Ok(PlanBoundary {
            selection_set_type,
            query_path,
            children,
        })
    }

    /// Mutation fields need to be executed sequentially. So instead of grouping fields by
    /// resolvers, we create a plan for each field in the order the fields appear in.
    fn create_mutation_plan_boundary(
        &mut self,
        missing_selection_set: FlatSelectionSetWalker<'_, '_>,
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

        let mut children = groups
            .into_iter()
            .map(|group| {
                let FieldResolverWalker {
                    resolver,
                    field_requires,
                } = walker
                    .walk(group.bound_field_ids[0])
                    .schema_field()
                    .expect("Introspection resolver should have taken metadata fields")
                    .resolvers()
                    .next()
                    .ok_or_else(|| PlanningError::CouldNotPlanAnyField {
                        missing: vec![walker.walk(group.bound_field_ids[0]).response_key_str().to_string()],
                        query_path: vec![],
                    })?;
                if !field_requires.is_empty() {
                    return Err(PlanningError::CouldNotSatisfyRequires {
                        resolver: resolver.name().to_string(),
                        field: field_requires
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
                    sibling_dependencies: HashSet::new(),
                    extra_selection_sets: HashMap::with_capacity(0),
                })
            })
            .collect::<PlanningResult<Vec<_>>>()?;

        // Ensuring mutation fields are executed in order.
        let mut previous_plan = None;
        for child in &mut children {
            if let Some(plan_id) = previous_plan {
                child.sibling_dependencies.insert(plan_id);
            }
            previous_plan = Some(child.id);
        }

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
            id,
            resolver_id,
            root_selection_set,
            extra_selection_sets,
            sibling_dependencies,
            ..
        }: ChildPlan,
    ) -> PlanningResult<(HashSet<PlanId>, PlanOutput, Vec<PlanBoundary>)> {
        let resolver = self.schema.walker().walk(resolver_id).with_own_names();
        let walker = self.operation.walker_with(resolver.walk(()));
        let root_selection_set_id = BoundSelectionSetId::from(root_selection_set.id);
        let entity_type = root_selection_set.ty;
        let flat_selection_set: FlatSelectionSetWalker<'_, '_, EntityType> =
            walker.walk(Cow::Owned(root_selection_set));

        let mut boundaries = Vec::new();
        let mut attribution = AttributionBuilder::default();
        attribution.add_extra_selection_sets(extra_selection_sets);
        let mut expectations = ExpectationsBuilder::default();
        let mut builder = PlanOutputBuilderContext {
            plan_id: id,
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
            sibling_dependencies,
            PlanOutput {
                root_selection_set_id,
                entity_type,
                root_fields: fields,
                attribution: attribution.build(),
                expectations: expectations.build(root_selection_set),
            },
            boundaries,
        ))
    }

    fn default_operation_walker(&self) -> OperationWalker<'op> {
        self.operation.walker_with(self.schema.walker())
    }

    fn get_extra_field_name(&mut self, field_id: FieldId) -> String {
        // When the resolver supports aliases, we must ensure that extra fields
        // don't collide with existing response keys. And to avoid duplicates
        // during field collection, we have a single unique name per field id.
        self.extra_field_names
            .entry(field_id)
            .or_insert_with(|| {
                let short_id = hex::encode(u32::from(field_id).to_be_bytes())
                    .trim_start_matches('0')
                    .to_uppercase();
                let name = format!("_{}{}", self.schema.walker().walk(field_id).name(), short_id);
                // name is unique, but may collide with existing keys so
                // iterating over candidates until we find a valid one.
                // This is only a safeguard, it most likely won't ever run.
                if self.operation.response_keys.contains(&name) {
                    let mut index = 0;
                    loop {
                        let candidate = format!("{name}{index}");
                        if !self.operation.response_keys.contains(&candidate) {
                            break candidate;
                        }
                        index += 1;
                    }
                } else {
                    name
                }
            })
            .to_string()
    }
}

struct PlanOutputBuilderContext<'op, 'plan> {
    planner: &'plan mut Planner<'op>,
    plan_id: PlanId,
    path: QueryPath,
    walker: OperationWalker<'op>,
    resolver: ResolverWalker<'op>,
    logic: AttributionLogic<'op>,
    attribution: &'plan mut AttributionBuilder,
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
    OnlyProvidable {
        resolver: ResolverWalker<'a>,
        providable: FieldSet,
    },
}

impl<'a> AttributionLogic<'a> {
    fn is_providable(&self, field: FieldWalker<'_>) -> bool {
        match self {
            AttributionLogic::CompatibleResolver { resolver, providable } => {
                providable.get(field.id()).is_some() || resolver.can_provide(field)
            }
            AttributionLogic::OnlyProvidable { providable, .. } => providable.get(field.id()).is_some(),
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
                    AttributionLogic::OnlyProvidable {
                        resolver: *resolver,
                        providable,
                    }
                }
            }
            AttributionLogic::OnlyProvidable { resolver, providable } => AttributionLogic::OnlyProvidable {
                resolver: *resolver,
                providable: providable
                    .get(field.id())
                    .map(|s| &s.subselection)
                    .cloned()
                    .unwrap_or_default(),
            },
        }
    }

    fn resolver(&self) -> &ResolverWalker<'a> {
        match self {
            AttributionLogic::CompatibleResolver { resolver, .. } => resolver,
            AttributionLogic::OnlyProvidable { resolver, .. } => resolver,
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

    fn partition_providable_missing<'a>(
        &mut self,
        flat_selection_set: FlatSelectionSetWalker<'op, 'a>,
    ) -> PlanningResult<(FlatSelectionSetWalker<'op, 'a>, Option<PlanBoundaryId>)> {
        let (providable, missing) = flat_selection_set.partition_fields(|flat_field| {
            flat_field
                .bound_field()
                .schema_field()
                .map(|field| self.logic.is_providable(field))
                .unwrap_or(true)
        });

        let maybe_boundary_id = if missing.is_empty() {
            None
        } else {
            let boundary = self.planner.create_plan_boundary(
                Some(PlanBoundaryParent {
                    plan_id: self.plan_id,
                    path: &self.path,
                    logic: self.logic.clone(),
                    attribution: self.attribution,
                    provided_selection_set: providable.clone(),
                }),
                missing,
            )?;
            Some(self.push_boundary(boundary))
        };

        Ok((providable, maybe_boundary_id))
    }

    fn collect_fields<Ty: Copy + Into<SelectionSetType> + std::fmt::Debug>(
        &mut self,
        flat_selection_set: FlatSelectionSetWalker<'op, '_, Ty>,
        maybe_boundary_id: Option<PlanBoundaryId>,
    ) -> PlanningResult<CollectedSelectionSet> {
        let mut fields = vec![];
        let mut typename_fields = vec![];
        for group in flat_selection_set.group_by_response_key().into_values() {
            self.attribution.attributed_fields.extend(&group.bound_field_ids);
            self.attribution
                .attributed_selection_sets
                .extend(group.origin_selection_set_ids);
            let bound_field = self.walker.walk(group.bound_field_ids[0]);
            if let Some(schema_field) = bound_field.schema_field() {
                let expected_key = if self.resolver.supports_aliases() {
                    bound_field.response_key_str().to_string()
                } else {
                    schema_field.name().to_string()
                };
                let ty = match schema_field.ty().inner().data_type() {
                    Some(data_type) => ConcreteType::Scalar(data_type),
                    None => ConcreteType::SelectionSet(
                        self.child(bound_field, schema_field)
                            .expected_selection_set(group.bound_field_ids)?,
                    ),
                };
                fields.push(ConcreteField {
                    expected_key,
                    edge: group.key.into(),
                    bound_field_id: Some(bound_field.id()),
                    wrapping: schema_field.ty().wrapping().clone(),
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
                bound_field_id: None,
                ty: match extra_field.ty {
                    ExpectedType::Scalar(data_type) => ConcreteType::Scalar(data_type),
                    ExpectedType::SelectionSet(id) => ConcreteType::ExtraSelectionSet(self.attribution[id].clone()),
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
        flat_selection_set: FlatSelectionSetWalker<'op, '_>,
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

    fn expected_ungrouped_field(&mut self, flat_field: FlatFieldWalker<'_, '_>) -> PlanningResult<PossibleField> {
        self.attribution.attributed_fields.push(flat_field.bound_field_id);
        self.attribution
            .attributed_selection_sets
            .extend(flat_field.selection_set_path.clone());
        let bound_field = flat_field.bound_field();

        if let Some(schema_field) = bound_field.schema_field() {
            let expected_key = if self.resolver.supports_aliases() {
                bound_field.response_key_str().to_string()
            } else {
                schema_field.name().to_string()
            };
            let ty = match schema_field.ty().inner().data_type() {
                Some(data_type) => ExpectedType::Scalar(data_type),
                None => ExpectedType::SelectionSet(
                    self.child(bound_field, schema_field)
                        .expected_undetermined_nested_selection_set(&flat_field)?,
                ),
            };
            Ok(PossibleField::Query(self.expectations.push_field(ExpectedField {
                type_condition: flat_field.into_item().type_condition,
                expected_key,
                ty,
                field_id: schema_field.id(),
                bound_response_key: bound_field.bound_response_key(),
                bound_field_id: bound_field.id(),
            })))
        } else {
            Ok(PossibleField::TypeName {
                type_condition: flat_field.into_item().type_condition,
                key: bound_field.bound_response_key(),
            })
        }
    }

    fn push_boundary(&mut self, boundary: PlanBoundary) -> PlanBoundaryId {
        let id = PlanBoundaryId::from(self.boundaries.len());
        self.boundaries.push(boundary);
        id
    }

    fn child<'s>(
        &'s mut self,
        bound_field: BoundFieldWalker<'_>,
        field: FieldWalker<'_>,
    ) -> PlanOutputBuilderContext<'op, 's> {
        PlanOutputBuilderContext {
            plan_id: self.plan_id,
            planner: self.planner,
            path: self.path.child(bound_field.response_key()),
            walker: self.walker,
            resolver: self.resolver,
            logic: self.logic.child(field),
            attribution: self.attribution,
            boundaries: self.boundaries,
            expectations: self.expectations,
        }
    }
}
