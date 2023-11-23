use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;
use schema::{DataSourceId, Definition, FieldSet, InterfaceId, Names, ObjectId, ResolverId};

use super::{
    attribution::AttributionBuilder, ExecutionPlan, ExecutionPlanRoot, ExecutionPlans, ExpectedArbitraryFields,
    ExpectedGoupedField, ExpectedGroupedFields, ExpectedUngroupedField, FieldOrTypeName, PlanId,
};
use crate::{
    plan::{ExpectedSelectionSet, ExpectedType},
    request::{
        BoundAnyFieldDefinition, BoundAnyFieldDefinitionId, BoundAnyFieldDefinitionWalker, BoundFieldId,
        BoundFieldWalker, BoundSelection, BoundSelectionSetId, FlatTypeCondition, Operation, QueryPath,
        QueryPathSegment, SelectionSetRoot, TypeCondition,
    },
    response::{BoundResponseKey, ReadSelectionSet, ResponseKey},
    Engine,
};

pub(super) struct Planner<'a> {
    pub(super) engine: &'a Engine,
    pub(super) operation: &'a Operation,
    pub(super) plans: &'a mut ExecutionPlans,
    pub(super) to_be_planned: VecDeque<ToBePlanned>,
}

#[derive(Debug)]
pub(super) struct ToBePlanned {
    pub(super) parent: Option<PlanId>,
    pub(super) root: ExecutionPlanRoot,
    pub(super) object_id: ObjectId,
}

impl<'a> Planner<'a> {
    pub(super) fn plan_fields(
        &mut self,
        ToBePlanned {
            parent,
            root,
            object_id,
        }: ToBePlanned,
    ) {
        let mut grouped_fields = self
            .collect_fields(object_id, &root.merged_selection_set_ids)
            .into_iter()
            .filter_map(|(key, group)| match &self.operation[group.definition_id] {
                // Meta fields don't need to be planned.
                BoundAnyFieldDefinition::Field(field) => Some((key, (field, group))),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        while !grouped_fields.is_empty() {
            pub struct Candidate {
                resolver_id: ResolverId,
                requires: FieldSet,
                output: Vec<ResponseKey>,
            }

            let mut candidates = HashMap::<ResolverId, Candidate>::new();
            for (key, (definition, _)) in &grouped_fields {
                for field_resolver in self
                    .engine
                    .schema
                    .default_walker()
                    .walk(definition.field_id)
                    .resolvers()
                {
                    let candidate = candidates
                        .entry(field_resolver.resolver_id)
                        .or_insert_with_key(|&resolver_id| Candidate {
                            resolver_id,
                            requires: FieldSet::default(),
                            output: Vec::new(),
                        });
                    candidate.requires = schema::FieldSet::merge(&candidate.requires, &field_resolver.requires);
                    candidate.output.push(*key);
                }
            }

            // We assume no inputs and separate outputs for now.
            // Later we could:
            // - Determine which candidate need additional data (mapping requires to actual fields or
            //   check whether they could be provided from parent/sibling plans).
            // - plan the one with most fields.
            let candidate = candidates.into_iter().next().unwrap().1;
            assert!(candidate.requires.is_empty());
            // We must progress by at least one field.
            assert!(!candidate.output.is_empty());

            let resolver = &self.engine.schema[candidate.resolver_id];
            let mut attribution = AttributionBuilder {
                selection_sets: HashSet::new(),
                fields: Vec::new(),
            };
            let expectation = {
                let mut ctx = AttributionContext {
                    names: self.engine.schema.as_ref(),
                    path: root.path.clone(),
                    provideable: schema::FieldSet::default(),
                    data_source_id: resolver.data_source_id(),
                    supports_aliases: resolver.supports_aliases(),
                    continuous: true,
                    attribution: &mut attribution,
                };
                let fields = candidate
                    .output
                    .into_iter()
                    .map(|key| grouped_fields.remove(&key).unwrap())
                    .map(|(_, group)| self.expected_grouped_field(&mut ctx, group, true));
                ExpectedSelectionSet::Grouped(ExpectedGroupedFields::new(SelectionSetRoot::Object(object_id), fields))
            };
            let plan_id = self.plans.push(ExecutionPlan {
                root: root.clone(),
                input: ReadSelectionSet::empty(),
                resolver_id: candidate.resolver_id,
                attribution: attribution.build(),
                expectation,
            });
            if let Some(parent) = parent {
                self.plans.add_dependency(plan_id, parent);
            }
        }
    }
}

struct GroupForResponseKey {
    key: BoundResponseKey,
    definition_id: BoundAnyFieldDefinitionId,
    origin_selection_set_ids: HashSet<BoundSelectionSetId>,
    bound_field_ids: Vec<BoundFieldId>,
}

type GoupedFields = HashMap<ResponseKey, GroupForResponseKey>;

impl<'a> Planner<'a> {
    fn collect_fields(&self, object_id: ObjectId, selection_set_ids: &Vec<BoundSelectionSetId>) -> GoupedFields {
        let mut fields = GoupedFields::new();
        let mut selections = VecDeque::new();
        for &id in selection_set_ids {
            selections.extend(self.operation[id].items.iter().map(|selection| (vec![id], selection)));
        }
        while let Some((mut selection_set_ids, selection)) = selections.pop_front() {
            match selection {
                BoundSelection::Field(id) => {
                    let field = &self.operation[*id];
                    let definition = &self.operation[field.definition_id];
                    let group = fields
                        .entry(definition.response_key())
                        .or_insert_with(|| GroupForResponseKey {
                            key: field.bound_response_key,
                            definition_id: field.definition_id,
                            bound_field_ids: vec![],
                            origin_selection_set_ids: HashSet::new(),
                        });
                    group.key = group.key.min(field.bound_response_key);
                    group.bound_field_ids.push(*id);
                    group.origin_selection_set_ids.extend(selection_set_ids);
                }

                BoundSelection::FragmentSpread(spread) => {
                    let type_condition = self.operation[spread.fragment_id].type_condition;
                    selection_set_ids.push(spread.selection_set_id);
                    if self.does_fragment_type_apply(object_id, type_condition) {
                        selections.extend(
                            self.operation[spread.selection_set_id]
                                .items
                                .iter()
                                .map(|selection| (selection_set_ids.clone(), selection)),
                        );
                    }
                }
                BoundSelection::InlineFragment(fragment) => {
                    if let Some(type_condition) = fragment.type_condition {
                        if self.does_fragment_type_apply(object_id, type_condition) {
                            selection_set_ids.push(fragment.selection_set_id);
                            selections.extend(
                                self.operation[fragment.selection_set_id]
                                    .items
                                    .iter()
                                    .map(|selection| (selection_set_ids.clone(), selection)),
                            );
                        }
                    } else {
                        selection_set_ids.push(fragment.selection_set_id);
                        selections.extend(
                            self.operation[fragment.selection_set_id]
                                .items
                                .iter()
                                .map(|selection| (selection_set_ids.clone(), selection)),
                        );
                    }
                }
            }
        }
        fields
    }

    fn does_fragment_type_apply(&self, object_id: ObjectId, type_condition: TypeCondition) -> bool {
        match type_condition {
            TypeCondition::Interface(interface_id) => {
                self.engine.schema[interface_id].possible_types.contains(&object_id)
            }
            TypeCondition::Object(id) => object_id == id,
            TypeCondition::Union(union_id) => self.engine.schema[union_id].possible_types.contains(&object_id),
        }
    }
}

struct AttributionContext<'a> {
    path: QueryPath,
    names: &'a dyn Names,
    provideable: FieldSet,
    data_source_id: DataSourceId,
    supports_aliases: bool,
    continuous: bool,
    attribution: &'a mut AttributionBuilder,
}

#[derive(Debug)]
struct FlatField {
    type_condition: Option<FlatTypeCondition>,
    selection_set_path: Vec<BoundSelectionSetId>,
    bound_field_id: BoundFieldId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InterfaceOrObject {
    Interface(InterfaceId),
    Object(ObjectId),
}

impl<'a> Planner<'a> {
    fn walk(&self, field: &FlatField) -> BoundFieldWalker<'a> {
        self.operation
            .walk_field(self.engine.schema.default_walker(), field.bound_field_id)
    }

    fn expected_object(
        &mut self,
        mut ctx: AttributionContext<'_>,
        root: SelectionSetRoot,
        bound_field_ids: Vec<BoundFieldId>,
    ) -> ExpectedSelectionSet {
        let flat_fields = self.flatten_selection_set(root, bound_field_ids);
        let (provideable, to_be_planned) = self.split_provideable_fields(&ctx, flat_fields);
        assert!(to_be_planned.is_empty());

        let mut conditions = HashSet::<Option<InterfaceOrObject>>::new();
        let mut too_complex = false;
        for field in &provideable {
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

        if !too_complex && conditions == HashSet::from([None]) {
            ExpectedSelectionSet::Grouped(ExpectedGroupedFields::new(
                root,
                self.optimize_arbitrary_flat_fields_into_expected_grouped_fields(&mut ctx, provideable),
            ))
        } else {
            ExpectedSelectionSet::Arbitrary(ExpectedArbitraryFields {
                root,
                fields: provideable
                    .into_iter()
                    .map(|flat_field| self.expected_ungrouped_field(&mut ctx, flat_field))
                    .collect(),
            })
        }
    }

    fn expected_arbitrary_fields(
        &mut self,
        mut ctx: AttributionContext<'_>,
        root: SelectionSetRoot,
        bound_field_ids: Vec<BoundFieldId>,
    ) -> ExpectedArbitraryFields {
        let flat_fields = self.flatten_selection_set(root, bound_field_ids);
        let (provideable, to_be_planned) = self.split_provideable_fields(&ctx, flat_fields);
        assert!(to_be_planned.is_empty());

        ExpectedArbitraryFields {
            root,
            fields: provideable
                .into_iter()
                .map(|flat_field| self.expected_ungrouped_field(&mut ctx, flat_field))
                .collect(),
        }
    }

    fn split_provideable_fields(
        &self,
        ctx: &AttributionContext<'_>,
        flat_fields: Vec<FlatField>,
    ) -> (Vec<FlatField>, Vec<FlatField>) {
        flat_fields
            .into_iter()
            .map(|flat_field| {
                match self.walk(&flat_field).definition() {
                    // __typename is always provideable if you could provide the object in the
                    // first place.
                    BoundAnyFieldDefinitionWalker::TypeName(_) => Ok(flat_field),
                    BoundAnyFieldDefinitionWalker::Field(field) => {
                        let provideable_field = ctx.provideable.get(field.id);
                        if provideable_field.is_some() || (ctx.continuous && field.resolvers.is_empty()) {
                            Ok(flat_field)
                        } else {
                            Err(flat_field)
                        }
                    }
                }
            })
            .partition_result()
    }

    fn flatten_selection_set(
        &self,
        root: SelectionSetRoot,
        grouped_bound_field_ids: Vec<BoundFieldId>,
    ) -> Vec<FlatField> {
        let mut fields = Vec::new();
        let mut selections = VecDeque::new();
        selections.extend(
            grouped_bound_field_ids
                .into_iter()
                .filter_map(|bound_field_id| self.operation[bound_field_id].selection_set_id)
                .flat_map(|selection_set_id| {
                    self.operation[selection_set_id]
                        .items
                        .iter()
                        .map(move |selection| (Vec::<TypeCondition>::new(), vec![selection_set_id], selection))
                }),
        );
        while let Some((mut type_condition_chain, mut selection_set_path, selection)) = selections.pop_front() {
            match selection {
                &BoundSelection::Field(id) => {
                    let type_condition = FlatTypeCondition::flatten(&self.engine.schema, root, type_condition_chain);
                    if type_condition.as_ref().map(|cond| cond.is_possible()).unwrap_or(true) {
                        fields.push(FlatField {
                            type_condition,
                            selection_set_path,
                            bound_field_id: id,
                        });
                    }
                }
                BoundSelection::FragmentSpread(spread) => {
                    let fragment = &self.operation[spread.fragment_id];
                    type_condition_chain.push(fragment.type_condition);
                    selection_set_path.push(spread.selection_set_id);
                    selections.extend(
                        self.operation[spread.selection_set_id]
                            .items
                            .iter()
                            .map(|selection| (type_condition_chain.clone(), selection_set_path.clone(), selection)),
                    );
                }
                BoundSelection::InlineFragment(fragment) => {
                    if let Some(type_condition) = fragment.type_condition {
                        type_condition_chain.push(type_condition);
                    }
                    selection_set_path.push(fragment.selection_set_id);
                    selections.extend(
                        self.operation[fragment.selection_set_id]
                            .items
                            .iter()
                            .map(|selection| (type_condition_chain.clone(), selection_set_path.clone(), selection)),
                    );
                }
            }
        }
        fields
    }

    fn optimize_arbitrary_flat_fields_into_expected_grouped_fields(
        &mut self,
        ctx: &mut AttributionContext<'_>,
        flat_fields: Vec<FlatField>,
    ) -> Vec<FieldOrTypeName> {
        flat_fields
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
            .map(|group| self.expected_grouped_field(ctx, group, false))
            .collect()
    }

    fn expected_grouped_field(
        &mut self,
        ctx: &mut AttributionContext<'_>,
        group: GroupForResponseKey,
        plan_root_field: bool,
    ) -> FieldOrTypeName {
        for &id in &group.bound_field_ids {
            ctx.attribution.fields.push(id);
        }
        ctx.attribution.selection_sets.extend(group.origin_selection_set_ids);
        match &self.operation[group.definition_id] {
            BoundAnyFieldDefinition::TypeName(_) => FieldOrTypeName::TypeName(group.key),
            BoundAnyFieldDefinition::Field(definition) => {
                let field = self.engine.schema.default_walker().walk(definition.field_id);
                let expected_name = if ctx.supports_aliases {
                    self.operation.response_keys[group.key].to_string()
                } else {
                    ctx.names.field(field.id).to_string()
                };

                let ctx = AttributionContext {
                    path: ctx.path.child(QueryPathSegment {
                        type_condition: None,
                        bound_response_key: group.key,
                    }),
                    provideable: FieldSet::merge_opt(
                        ctx.provideable.get(definition.field_id).map(|s| &s.selection_set),
                        field.provides(ctx.data_source_id),
                    ),
                    continuous: plan_root_field || field.resolvers.is_empty(),
                    data_source_id: ctx.data_source_id,
                    supports_aliases: ctx.supports_aliases,
                    attribution: ctx.attribution,
                    names: ctx.names,
                };
                let ty = match field.ty().inner().id {
                    Definition::Object(object_id) => ExpectedType::Object(Box::new(self.expected_object(
                        ctx,
                        SelectionSetRoot::Object(object_id),
                        group.bound_field_ids,
                    ))),
                    Definition::Interface(interface_id) => ExpectedType::Object(Box::new(self.expected_object(
                        ctx,
                        SelectionSetRoot::Interface(interface_id),
                        group.bound_field_ids,
                    ))),
                    Definition::Union(union_id) => ExpectedType::Object(Box::new(self.expected_object(
                        ctx,
                        SelectionSetRoot::Union(union_id),
                        group.bound_field_ids,
                    ))),
                    _ => ExpectedType::Scalar(
                        field
                            .ty()
                            .inner()
                            .data_type()
                            .expect("Only Scalar and Enum should be left"),
                    ),
                };

                FieldOrTypeName::Field(ExpectedGoupedField {
                    expected_name,
                    bound_response_key: group.key,
                    definition_id: group.definition_id,
                    wrapping: field.ty().wrapping.clone(),
                    ty,
                })
            }
        }
    }

    fn expected_ungrouped_field(
        &mut self,
        ctx: &mut AttributionContext<'_>,
        flat_field: FlatField,
    ) -> ExpectedUngroupedField {
        ctx.attribution.fields.push(flat_field.bound_field_id);
        ctx.attribution
            .selection_sets
            .extend(flat_field.selection_set_path.clone());
        let bound_field = self.walk(&flat_field);
        let (expected_name, ty) = match bound_field.definition() {
            BoundAnyFieldDefinitionWalker::TypeName(_) => (None, ExpectedType::TypeName),
            BoundAnyFieldDefinitionWalker::Field(field) => {
                let expected_name = Some(if ctx.supports_aliases {
                    self.operation.response_keys[self.operation[flat_field.bound_field_id].bound_response_key]
                        .to_string()
                } else {
                    ctx.names.field(field.id).to_string()
                });
                let ctx = AttributionContext {
                    path: ctx.path.child(QueryPathSegment {
                        type_condition: flat_field.type_condition.clone(),
                        bound_response_key: bound_field.bound_response_key(),
                    }),
                    provideable: FieldSet::merge_opt(
                        ctx.provideable.get(field.id).map(|s| &s.selection_set),
                        field.provides(ctx.data_source_id),
                    ),
                    continuous: field.resolvers.is_empty(),
                    data_source_id: ctx.data_source_id,
                    supports_aliases: ctx.supports_aliases,
                    attribution: ctx.attribution,
                    names: ctx.names,
                };
                let ty = match field.ty().inner().id {
                    Definition::Object(object_id) => ExpectedType::Object(Box::new(self.expected_arbitrary_fields(
                        ctx,
                        SelectionSetRoot::Object(object_id),
                        vec![flat_field.bound_field_id],
                    ))),
                    Definition::Interface(interface_id) => {
                        ExpectedType::Object(Box::new(self.expected_arbitrary_fields(
                            ctx,
                            SelectionSetRoot::Interface(interface_id),
                            vec![flat_field.bound_field_id],
                        )))
                    }
                    Definition::Union(union_id) => ExpectedType::Object(Box::new(self.expected_arbitrary_fields(
                        ctx,
                        SelectionSetRoot::Union(union_id),
                        vec![flat_field.bound_field_id],
                    ))),
                    _ => ExpectedType::Scalar(
                        field
                            .ty()
                            .inner()
                            .data_type()
                            .expect("Only Scalar and Enum should be left"),
                    ),
                };
                (expected_name, ty)
            }
        };
        ExpectedUngroupedField {
            expected_name,
            type_condition: flat_field.type_condition,
            origin: *flat_field.selection_set_path.last().unwrap(),
            bound_field_id: flat_field.bound_field_id,
            ty,
        }
    }
}
