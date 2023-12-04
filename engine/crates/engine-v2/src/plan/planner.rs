use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
};

use itertools::Itertools;
use schema::{FieldId, FieldSet, InterfaceId, Names, ObjectId, ResolverId, ResolverWalker, Schema};

use super::{
    attribution::AttributionBuilder, ExpectedArbitraryFields, ExpectedGoupedField, ExpectedGroupedFields,
    ExpectedUngroupedField, FieldOrTypeName, FlatTypeCondition, Plan, PlanBoundary, PlanBoundaryId, PlanId, PlanInput,
    PlanOutput,
};
use crate::{
    plan::{ChildPlan, EntityType, ExpectedSelectionSet, ExpectedType},
    request::{
        BoundAnyFieldDefinitionId, BoundFieldDefinitionWalker, BoundFieldId, BoundSelectionSetId, FlatField,
        FlatSelectionSet, Operation, OperationWalker, QueryPath, SelectionSetType,
    },
    response::{BoundResponseKey, GraphqlError, ReadField, ReadSelectionSet, ResponseBoundaryItem, ResponseKey},
};

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Could not plan fields: {}", .missing.join(", "))]
    CouldNotPlanAnyField {
        query_path: Vec<String>,
        missing: Vec<String>,
    },
    #[error("Could not satisfy required fields")]
    CouldNotSatisfyRequires,
}

impl From<PlanningError> for GraphqlError {
    fn from(err: PlanningError) -> Self {
        let message = err.to_string();
        let query_path = match err {
            PlanningError::CouldNotPlanAnyField { query_path, .. } => query_path,
            PlanningError::CouldNotSatisfyRequires { .. } => vec![],
        };
        GraphqlError {
            message,
            locations: vec![],
            path: None,
            extensions: HashMap::from([(
                "queryPath".into(),
                serde_json::Value::Array(query_path.into_iter().map(serde_json::Value::String).collect()),
            )]),
        }
    }
}

pub type PlanningResult<T> = Result<T, PlanningError>;

pub struct Planner<'a> {
    schema: &'a Schema,
    operation: &'a Operation,
    next_plan_id: Cell<usize>,
}

impl<'a> Planner<'a> {
    pub fn new(schema: &'a Schema, operation: &'a Operation) -> Self {
        Planner {
            schema,
            operation,
            next_plan_id: Cell::new(0),
        }
    }

    pub fn generate_initial_boundary(&mut self) -> PlanningResult<PlanBoundary> {
        let walker = self.default_operation_walker();
        let flat_selection_set = walker.flatten_selection_sets(vec![self.operation.root_selection_set_id]);

        // The default resolver is the introspection one which allows use deal nicely with queries
        // like `query { __typename }`. So all fields without a resolvers are considered to be provideable by introspection.
        let (provideable, missing): (Vec<_>, Vec<_>) = flat_selection_set
            .fields
            .into_iter()
            .map(|flat_field| {
                let field = walker.walk(flat_field.bound_field_id);
                if field
                    .definition()
                    .as_field()
                    .map(|field| field.resolvers.is_empty())
                    .unwrap_or(true)
                {
                    Ok(flat_field.bound_field_id)
                } else {
                    Err(flat_field)
                }
            })
            .partition_result();
        let mut boundary = self.create_plan_boundary(
            &FlatSelectionSet {
                ty: SelectionSetType::Object(self.operation.root_object_id),
                fields: vec![],
            },
            None,
            FlatSelectionSet {
                fields: missing,
                ..flat_selection_set
            },
        )?;

        // Are there actually any introspection related fields?
        if !provideable.is_empty() {
            let resolver_id = self.schema.introspection_resolver_id();
            boundary.children.push(ChildPlan {
                id: self.next_plan_id(),
                path: QueryPath::default(),
                entity_type: EntityType::Object(match walker.walk(self.operation.root_selection_set_id).ty {
                    SelectionSetType::Object(id) => id,
                    _ => unreachable!("The root selection set is always an object"),
                }),
                resolver_id,
                input_selection_set: ReadSelectionSet::default(),
                bound_field_ids: provideable,
            });
        }
        Ok(boundary)
    }

    pub fn generate_plans(
        &mut self,
        boundary: PlanBoundary,
        response_objects: &Vec<ResponseBoundaryItem>,
    ) -> PlanningResult<Vec<Plan>> {
        if response_objects.is_empty() {
            return Ok(vec![]);
        }
        boundary
            .children
            .into_iter()
            .filter_map(|mut child| {
                // we could certainly be smarter and avoids copies with an Arc.
                let root_response_objects = match (boundary.selection_set_type, child.entity_type) {
                    (SelectionSetType::Object(_), _) => response_objects.clone(),
                    (SelectionSetType::Interface(a), EntityType::Interface(b)) if a == b => response_objects.clone(),
                    (_, EntityType::Interface(id)) => {
                        let possible_types = &self.schema[id].possible_types;
                        response_objects
                            .iter()
                            .filter(|root| possible_types.binary_search(&root.object_id).is_ok())
                            .cloned()
                            .collect()
                    }
                    (_, EntityType::Object(id)) => response_objects
                        .iter()
                        .filter(|root| root.object_id == id)
                        .cloned()
                        .collect(),
                };
                if root_response_objects.is_empty() {
                    None
                } else {
                    let id = child.id;
                    let resolver_id = child.resolver_id;
                    let selection_set = std::mem::take(&mut child.input_selection_set);
                    Some(
                        self.create_plan_output(self.schema.walker().walk(resolver_id).names(), child)
                            .map(|(output, boundaries)| Plan {
                                id,
                                resolver_id,
                                input: PlanInput {
                                    response_boundary: root_response_objects,
                                    selection_set,
                                },
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

    fn partition_provideable_missing(
        &self,
        ctx: &mut AttributionContext<'_>,
        FlatSelectionSet { ty, fields }: FlatSelectionSet,
    ) -> PlanningResult<(FlatSelectionSet, Option<PlanBoundaryId>)> {
        let (provideable, missing): (Vec<_>, Vec<_>) = fields
            .into_iter()
            .map(
                |flat_field| match ctx.walker.walk(flat_field.bound_field_id).definition().as_field() {
                    Some(field) => {
                        if ctx.is_provideable(field) {
                            Ok(flat_field)
                        } else {
                            Err(flat_field)
                        }
                    }
                    None => Ok(flat_field),
                },
            )
            .partition_result();

        let provideable = FlatSelectionSet {
            ty,
            fields: provideable,
        };
        let maybe_boundary_id = if missing.is_empty() {
            None
        } else {
            let missing = FlatSelectionSet { ty, fields: missing };
            let boundary = self.create_plan_boundary(&provideable, Some(ctx), missing)?;
            Some(ctx.push_boundary(boundary))
        };

        Ok((provideable, maybe_boundary_id))
    }

    fn create_plan_boundary(
        &self,
        provideable: &FlatSelectionSet,
        maybe_parent: Option<&mut AttributionContext<'_>>,
        missing: FlatSelectionSet,
    ) -> PlanningResult<PlanBoundary> {
        let walker = self.default_operation_walker();
        let field_id_to_bound_field_id =
            provideable
                .fields
                .iter()
                .fold(HashMap::<FieldId, BoundFieldId>::new(), |mut acc, flat_field| {
                    if let Some(field) = walker.walk(flat_field.bound_field_id).definition().as_field() {
                        let id = acc.entry(field.id()).or_insert(flat_field.bound_field_id);
                        // If the field is actually read by a child plan input it means the object is present and any present
                        // type condition was satisfied for this flat field, the one with the lowesed
                        // bound_field_id will be present as it's the one with the lowest position in
                        // the query.
                        *id = (*id).min(flat_field.bound_field_id);
                    }
                    acc
                });
        let mut id_to_missing_fields: HashMap<BoundFieldId, FlatField> = missing
            .fields
            .into_iter()
            .map(|field| (field.bound_field_id, field))
            .collect();

        let mut children = Vec::new();
        'candidates_generation: while !id_to_missing_fields.is_empty() {
            let count = id_to_missing_fields.len();
            pub struct ChildPlanCandidate<'a> {
                entity_type: EntityType,
                resolver_id: ResolverId,
                fields: Vec<(BoundFieldId, &'a FieldSet)>,
            }

            let mut candidates = HashMap::<ResolverId, ChildPlanCandidate<'_>>::new();
            for field in id_to_missing_fields.values() {
                for field_resolver in walker
                    .walk(field.bound_field_id)
                    .definition()
                    .as_field()
                    .expect("Meta fields are always provideable so a missing field can't be one.")
                    .resolvers()
                {
                    // We don't need to care about type conditions here as the resolver naturally
                    // enforces that only fields of the same interface/object can be grouped
                    // together.
                    let entity_type = match self.operation[*field.selection_set_path.last().unwrap()].ty {
                        SelectionSetType::Object(id) => EntityType::Object(id),
                        SelectionSetType::Interface(id) => EntityType::Interface(id),
                        SelectionSetType::Union(_) => unreachable!("An union doesn't have fields"),
                    };
                    let candidate = candidates
                        .entry(field_resolver.resolver.id())
                        .or_insert_with_key(|&resolver_id| ChildPlanCandidate {
                            entity_type,
                            resolver_id,
                            fields: Vec::new(),
                        });
                    candidate.fields.push((field.bound_field_id, field_resolver.requires));
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
                let mut bound_field_ids = vec![];
                let mut requires = walker.schema().walk(candidate.resolver_id).requires().into_owned();
                for (id, field_requires) in candidate.fields {
                    let flat_field = id_to_missing_fields.remove(&id).unwrap();
                    bound_field_ids.push(flat_field.bound_field_id);
                    requires = FieldSet::merge(&requires, field_requires);
                }
                let input_selection_set = requires
                    .into_iter()
                    .map(
                        |schema::FieldSetItem {
                             field_id,
                             selection_set,
                         }| {
                            // For now we only support the most basic requires. root fields already
                            // present in the selection.
                            if !selection_set.is_empty() {
                                return Err(PlanningError::CouldNotSatisfyRequires);
                            }
                            let bound_field_id = field_id_to_bound_field_id
                                .get(&field_id)
                                .ok_or(PlanningError::CouldNotSatisfyRequires)?;
                            let bound_field = walker.walk(*bound_field_id);
                            Ok(ReadField {
                                bound_response_key: bound_field.bound_response_key,
                                field_id,
                                subselection: ReadSelectionSet::default(),
                            })
                        },
                    )
                    .collect::<PlanningResult<ReadSelectionSet>>()?;
                children.push(ChildPlan {
                    id: self.next_plan_id(),
                    path: maybe_parent
                        .as_ref()
                        .map(|parent| parent.path.clone())
                        .unwrap_or_default(),
                    entity_type: candidate.entity_type,
                    resolver_id: candidate.resolver_id,
                    input_selection_set,
                    bound_field_ids,
                });
            }

            // No fields were planned
            if count == id_to_missing_fields.len() {
                let query_path = maybe_parent
                    .map(|parent| {
                        parent
                            .path
                            .into_iter()
                            .map(|key| self.operation.response_keys[*key].to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                let missing = id_to_missing_fields
                    .into_keys()
                    .map(|id| walker.walk(id).response_key_str().to_string())
                    .collect();
                return Err(PlanningError::CouldNotPlanAnyField { query_path, missing });
            }
        }
        Ok(PlanBoundary {
            selection_set_type: missing.ty,
            children,
        })
    }

    fn create_plan_output(
        &self,
        names: &'a dyn Names,
        ChildPlan {
            path,
            entity_type,
            resolver_id,
            bound_field_ids: fields,
            ..
        }: ChildPlan,
    ) -> PlanningResult<(PlanOutput, Vec<PlanBoundary>)> {
        let walker = self.operation.walker_with(self.schema.walker_with(names), ());
        let resolver = walker.schema().walk(resolver_id);
        let mut groups = HashMap::<ResponseKey, Vec<BoundFieldId>>::new();
        let walker = self.default_operation_walker();
        for id in &fields {
            groups.entry(walker.walk(*id).response_key()).or_default().push(*id)
        }

        let mut boundaries = Vec::new();
        let mut attribution = AttributionBuilder::default();
        let mut ctx = AttributionContext {
            path,
            walker,
            resolver,
            logic: AttributionLogic::CompatibleResolver,
            provideable: FieldSet::default(),
            attribution: &mut attribution,
            boundaries: &mut boundaries,
        };
        let ty = match entity_type {
            EntityType::Interface(id) => SelectionSetType::Interface(id),
            EntityType::Object(id) => SelectionSetType::Object(id),
        };
        let grouped_fields = groups
            .into_values()
            .map(|ids| {
                let first_bound_field = &self.operation[*ids.iter().min().unwrap()];
                let group = GroupForResponseKey {
                    key: first_bound_field.bound_response_key,
                    // the min id is the first field to appear as BoundFieldId are produced as we
                    // traverse the query in the right order..
                    definition_id: first_bound_field.definition_id,
                    origin_selection_set_ids: HashSet::with_capacity(0),
                    bound_field_ids: ids,
                };
                self.expected_grouped_field(&mut ctx, group)
            })
            .collect::<PlanningResult<Vec<_>>>()?;
        let expectation = ExpectedSelectionSet::Grouped(ExpectedGroupedFields::new(None, ty, grouped_fields));
        Ok((
            PlanOutput {
                entity_type,
                fields,
                attribution: attribution.build(),
                expectation,
            },
            boundaries,
        ))
    }

    fn expected_grouped_field(
        &self,
        ctx: &mut AttributionContext<'_>,
        group: GroupForResponseKey,
    ) -> PlanningResult<FieldOrTypeName> {
        for &id in &group.bound_field_ids {
            ctx.attribution.fields.push(id);
        }
        ctx.attribution.selection_sets.extend(group.origin_selection_set_ids);
        ctx.walker
            .walk(group.definition_id)
            .as_field()
            .map(|field| {
                let expected_name = if ctx.resolver.supports_aliases() {
                    field.response_key_str().to_string()
                } else {
                    field.name().to_string()
                };
                let ty = match field.ty().inner().data_type() {
                    Some(data_type) => ExpectedType::Scalar(data_type),
                    None => {
                        ExpectedType::Object(Box::new(self.expected_object(ctx.child(field), group.bound_field_ids)?))
                    }
                };
                Ok(FieldOrTypeName::Field(ExpectedGoupedField {
                    expected_name,
                    bound_response_key: group.key,
                    definition_id: group.definition_id,
                    wrapping: field.ty().wrapping.clone(),
                    ty,
                }))
            })
            .transpose()
            .map(|maybe_field| maybe_field.unwrap_or(FieldOrTypeName::TypeName(group.key)))
    }

    fn expected_object(
        &self,
        mut ctx: AttributionContext<'_>,
        bound_field_ids: Vec<BoundFieldId>,
    ) -> PlanningResult<ExpectedSelectionSet> {
        let flat_selection_set = ctx.walker.flatten_selection_sets(
            bound_field_ids
                .iter()
                .filter_map(|id| self.operation[*id].selection_set_id)
                .collect(),
        );
        let ty = flat_selection_set.ty;
        let (provideable, maybe_boundary_id) = self.partition_provideable_missing(&mut ctx, flat_selection_set)?;

        let mut conditions = HashSet::<Option<InterfaceOrObject>>::new();
        let mut too_complex = false;
        for field in &provideable.fields {
            match &field.type_condition {
                Some(type_condition) => match type_condition {
                    FlatTypeCondition::Interface(id) => {
                        conditions.insert(Some(InterfaceOrObject::Interface(*id)));
                    }
                    FlatTypeCondition::Objects(ids) => {
                        if ids.len() == 1 {
                            conditions.insert(Some(InterfaceOrObject::Object(ids[0])));
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

        Ok(if !too_complex && conditions == HashSet::from([None]) {
            ExpectedSelectionSet::Grouped(ExpectedGroupedFields::new(
                maybe_boundary_id,
                ty,
                self.optimize_arbitrary_flat_fields_into_expected_grouped_fields(&mut ctx, provideable)?,
            ))
        } else {
            ExpectedSelectionSet::Arbitrary(ExpectedArbitraryFields {
                maybe_boundary_id,
                ty,
                fields: provideable
                    .fields
                    .into_iter()
                    .map(|flat_field| self.expected_ungrouped_field(&mut ctx, flat_field))
                    .collect::<PlanningResult<_>>()?,
            })
        })
    }

    fn optimize_arbitrary_flat_fields_into_expected_grouped_fields(
        &self,
        ctx: &mut AttributionContext<'_>,
        flat_selection_set: FlatSelectionSet,
    ) -> PlanningResult<Vec<FieldOrTypeName>> {
        flat_selection_set
            .fields
            .into_iter()
            .fold(
                HashMap::<ResponseKey, GroupForResponseKey>::new(),
                |mut groups, flat_field| {
                    let field = &self.operation[flat_field.bound_field_id];
                    let definition = &self.operation[field.definition_id];
                    let group = groups
                        .entry(definition.response_key())
                        .or_insert_with(|| GroupForResponseKey {
                            key: field.bound_response_key,
                            definition_id: field.definition_id,
                            origin_selection_set_ids: HashSet::new(),
                            bound_field_ids: vec![],
                        });
                    group.key = group.key.min(field.bound_response_key);
                    group.bound_field_ids.push(flat_field.bound_field_id);
                    group.origin_selection_set_ids.extend(flat_field.selection_set_path);

                    groups
                },
            )
            .into_values()
            .map(|group| self.expected_grouped_field(ctx, group))
            .collect()
    }

    fn expected_ungrouped_field(
        &self,
        ctx: &mut AttributionContext<'_>,
        flat_field: FlatField,
    ) -> PlanningResult<ExpectedUngroupedField> {
        ctx.attribution.fields.push(flat_field.bound_field_id);
        ctx.attribution
            .selection_sets
            .extend(flat_field.selection_set_path.clone());
        let (expected_name, ty) = ctx
            .walker
            .walk(flat_field.bound_field_id)
            .definition()
            .as_field()
            .map(|field| {
                let expected_name = Some(if ctx.resolver.supports_aliases() {
                    field.response_key_str().to_string()
                } else {
                    field.name().to_string()
                });
                match field.ty().inner().data_type() {
                    Some(data_type) => Ok((expected_name, ExpectedType::Scalar(data_type))),
                    None => self
                        .expected_arbitrary_fields(ctx.child(field), vec![flat_field.bound_field_id])
                        .map(|object| {
                            let ty = ExpectedType::Object(Box::new(object));
                            (expected_name, ty)
                        }),
                }
            })
            .transpose()?
            .unwrap_or((None, ExpectedType::TypeName));
        Ok(ExpectedUngroupedField {
            expected_name,
            type_condition: flat_field.type_condition,
            origin: *flat_field.selection_set_path.last().unwrap(),
            bound_field_id: flat_field.bound_field_id,
            ty,
        })
    }

    fn expected_arbitrary_fields(
        &self,
        mut ctx: AttributionContext<'_>,
        bound_field_ids: Vec<BoundFieldId>,
    ) -> PlanningResult<ExpectedArbitraryFields> {
        let flat_selection_set = ctx.walker.flatten_selection_sets(
            bound_field_ids
                .iter()
                .filter_map(|id| self.operation[*id].selection_set_id)
                .collect(),
        );
        let ty = flat_selection_set.ty;
        let (provideable, maybe_boundary_id) = self.partition_provideable_missing(&mut ctx, flat_selection_set)?;

        Ok(ExpectedArbitraryFields {
            maybe_boundary_id,
            ty,
            fields: provideable
                .fields
                .into_iter()
                .map(|flat_field| self.expected_ungrouped_field(&mut ctx, flat_field))
                .collect::<PlanningResult<_>>()?,
        })
    }

    fn default_operation_walker(&self) -> OperationWalker<'a> {
        self.operation.walker_with(self.schema.walker(), ())
    }
}

struct GroupForResponseKey {
    key: BoundResponseKey,
    definition_id: BoundAnyFieldDefinitionId,
    origin_selection_set_ids: HashSet<BoundSelectionSetId>,
    bound_field_ids: Vec<BoundFieldId>,
}

#[derive(Debug)]
struct AttributionContext<'a> {
    path: QueryPath,
    walker: OperationWalker<'a>,
    provideable: FieldSet,
    resolver: ResolverWalker<'a>,
    logic: AttributionLogic,
    attribution: &'a mut AttributionBuilder,
    boundaries: &'a mut Vec<PlanBoundary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AttributionLogic {
    /// Having a resolver in the same group or having no resolver at all.
    CompatibleResolver,
    /// Only an explicitly provideable (@provide) field can be attributed. This is an optimization
    /// overriding the CompatibleResolver logic
    Provideable,
}

impl AttributionLogic {
    fn must_be_provideable(&self) -> bool {
        matches!(self, AttributionLogic::Provideable)
    }
}

impl<'a> AttributionContext<'a> {
    fn push_boundary(&mut self, boundary: PlanBoundary) -> PlanBoundaryId {
        let id = PlanBoundaryId::from(self.boundaries.len());
        self.boundaries.push(boundary);
        id
    }

    fn is_provideable(&self, field: BoundFieldDefinitionWalker<'_>) -> bool {
        let provideable_field = self.provideable.get(field.id());
        provideable_field.is_some() || (!self.logic.must_be_provideable() && self.has_compatible_resolver(field))
    }

    fn has_compatible_resolver(&self, field: BoundFieldDefinitionWalker<'_>) -> bool {
        if let Some(compatible_group) = self.resolver.group() {
            field.resolvers.is_empty()
                || field
                    .resolvers()
                    .filter_map(|fr| fr.resolver.group())
                    .any(|group| group == compatible_group)
        } else {
            field.resolvers.is_empty()
        }
    }

    fn child<'s>(&'s mut self, field: BoundFieldDefinitionWalker<'_>) -> AttributionContext<'s> {
        AttributionContext {
            path: self.path.child(field.response_key()),
            walker: self.walker,
            resolver: self.resolver,
            logic: match (self.logic, self.has_compatible_resolver(field)) {
                (AttributionLogic::CompatibleResolver, true) => AttributionLogic::CompatibleResolver,
                _ => AttributionLogic::Provideable,
            },
            provideable: FieldSet::merge_opt(
                self.provideable.get(field.id()).map(|s| &s.selection_set),
                Some(&field.provides),
            ),
            attribution: self.attribution,
            boundaries: self.boundaries,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InterfaceOrObject {
    Interface(InterfaceId),
    Object(ObjectId),
}
