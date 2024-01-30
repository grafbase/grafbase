use fnv::FnvHashSet;
use itertools::Itertools;
use schema::Schema;

use crate::{
    plan::{
        CollectedSelectionSet, ConcreteField, ConcreteFieldId, ConcreteSelectionSet, ConcreteSelectionSetId,
        ConditionalField, ConditionalFieldId, ConditionalSelectionSet, ConditionalSelectionSetId, FieldType,
        OperationPlan, PlanBoundaryId, PlanId,
    },
    request::{
        BoundFieldId, BoundSelectionSetId, EntityType, FlatField, FlatTypeCondition, OperationWalker, SelectionSetType,
    },
    utils::IdRange,
};

use super::PlanningResult;

pub(super) struct Collector<'schema, 'op> {
    schema: &'schema Schema,
    operation: &'op mut OperationPlan,
    plan_id: PlanId,
    support_aliases: bool,
}

impl<'schema, 'a> Collector<'schema, 'a> {
    pub(super) fn new(schema: &'schema Schema, operation: &'a mut OperationPlan, plan_id: PlanId) -> Self {
        let support_aliases = schema
            .walk(operation.plans[usize::from(plan_id)].resolver_id)
            .supports_aliases();
        Collector {
            schema,
            operation,
            plan_id,
            support_aliases,
        }
    }

    pub fn walker(&self) -> OperationWalker<'_> {
        self.operation.walker_with(self.schema.walker())
    }

    pub(super) fn collect(
        &mut self,
        root_selection_set_ids: Vec<BoundSelectionSetId>,
    ) -> PlanningResult<ConcreteSelectionSetId> {
        let ty = self.operation[root_selection_set_ids[0]].ty;
        let fields = self.find_root_fields(root_selection_set_ids);
        tracing::debug!(
            "Collecting output for plan {} from root fields: {}",
            self.plan_id,
            fields
                .iter()
                .map(|id| self.walker().walk(*id).response_key_str())
                .join(", ")
        );
        self.collect_concrete_selection_set(ty, fields, None)
    }

    fn find_root_fields(&self, root_selection_set_ids: Vec<BoundSelectionSetId>) -> Vec<BoundFieldId> {
        let walker = self.walker();
        root_selection_set_ids
            .into_iter()
            .flat_map(|id| walker.walk(id).fields())
            .filter_map(|field| {
                let field_plan_id = self.operation.field_attribution[usize::from(field.id())];
                if field_plan_id == self.plan_id {
                    Some(field.id())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    fn collect_selection_set(
        &mut self,
        selection_set_ids: Vec<BoundSelectionSetId>,
        concrete_parent: bool,
    ) -> PlanningResult<CollectedSelectionSet> {
        let selection_set = self.walker().flatten_selection_sets(selection_set_ids);

        let mut maybe_boundary_id = None;
        let mut plan_fields = Vec::new();
        for field in selection_set.fields {
            if !self.operation[field.bound_field_id].is_read() {
                continue;
            }

            let field_plan_id = self.operation.field_attribution[usize::from(field.bound_field_id)];
            if field_plan_id == self.plan_id {
                plan_fields.push(field);
            } else if maybe_boundary_id.is_none() {
                maybe_boundary_id = Some(
                    self.operation.plan_inputs[usize::from(field_plan_id)]
                        .as_ref()
                        .map(|input| input.boundary_id)
                        .expect("Children always have inputs"),
                );
            }
        }

        let mut conditions = FnvHashSet::<Option<EntityType>>::default();
        let mut too_complex = false;
        for field in &plan_fields {
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

        // Trying to simplify the attributed selection to a concrete one.
        // - if the parent is not concrete, there might be other selection sets that need to be merged
        //   at runtime with this one.
        // - the only concrete selection set we support right now is one without any conditions.
        //   If a single condition is left, we can only work with None. A selection set like
        //   `animal { ... on Dog { name } }` would have a single condition, but we may still see
        //   cat objects. A ConcreteSelectionSet would require `name`.
        if concrete_parent && !too_complex && conditions.len() == 1 && conditions.contains(&None) {
            self.collect_concrete_selection_set(
                selection_set.ty,
                plan_fields.into_iter().map(|field| field.bound_field_id).collect(),
                maybe_boundary_id,
            )
            .map(CollectedSelectionSet::Concrete)
        } else {
            self.create_conditional_selection_set(selection_set.ty, plan_fields, maybe_boundary_id)
                .map(CollectedSelectionSet::Conditional)
        }
    }

    fn collect_concrete_selection_set(
        &mut self,
        ty: SelectionSetType,
        fields: Vec<BoundFieldId>,
        maybe_boundary_id: Option<PlanBoundaryId>,
    ) -> PlanningResult<ConcreteSelectionSetId> {
        let grouped_by_response_key = self.walker().group_by_response_key(fields).into_values();

        let mut fields = vec![];
        let mut typename_fields = vec![];
        for group in grouped_by_response_key {
            let bound_field_id = group.final_bound_field_id;
            let bound_field = self.operation[bound_field_id].clone();
            if let Some(field_id) = bound_field.schema_field_id() {
                let schema_field = self.schema.walk(field_id);
                let expected_key = if self.support_aliases {
                    bound_field.response_key()
                } else {
                    self.operation
                        .bound_operation
                        .response_keys
                        .get_or_intern(schema_field.name())
                };
                let ty = match schema_field.ty().inner().data_type() {
                    Some(data_type) => FieldType::Scalar(data_type),
                    None => FieldType::SelectionSet(self.collect_selection_set(group.subselection_set_ids, true)?),
                };
                fields.push(ConcreteField {
                    expected_key,
                    edge: group.edge,
                    bound_field_id,
                    schema_field_id: field_id,
                    wrapping: schema_field.ty().wrapping().clone(),
                    ty,
                });
            } else {
                typename_fields.push(bound_field.response_edge());
            }
        }

        // Sorting by expected_key for deserialization
        let keys = &self.operation.response_keys;
        fields.sort_unstable_by(|a, b| keys[a.expected_key].cmp(&keys[b.expected_key]));
        let fields = self.push_concrete_fields(fields);
        Ok(self.push_concrete_selection_set(ConcreteSelectionSet {
            ty,
            maybe_boundary_id,
            fields,
            typename_fields,
        }))
    }

    fn create_conditional_selection_set(
        &mut self,
        ty: SelectionSetType,
        fields: Vec<FlatField>,
        maybe_boundary_id: Option<PlanBoundaryId>,
    ) -> PlanningResult<ConditionalSelectionSetId> {
        let mut typename_fields = Vec::new();
        let mut conditional_fields = Vec::new();
        for field in fields {
            let bound_field = self.operation[field.bound_field_id].clone();
            if let Some(field_id) = bound_field.schema_field_id() {
                let schema_field = self.schema.walker().walk(field_id);
                let expected_key = if self.support_aliases {
                    bound_field.response_key()
                } else {
                    self.operation
                        .bound_operation
                        .response_keys
                        .get_or_intern(schema_field.name())
                };
                let ty = match schema_field.ty().inner().data_type() {
                    Some(data_type) => FieldType::Scalar(data_type),
                    None => {
                        let selection_set_id =
                            self.collect_selection_set(bound_field.selection_set_id().into_iter().collect(), false)?;
                        let CollectedSelectionSet::Conditional(selection_set_id) = selection_set_id else {
                            unreachable!("undetermined selection set cannot produce concrete selecitons");
                        };
                        FieldType::SelectionSet(selection_set_id)
                    }
                };
                conditional_fields.push(ConditionalField {
                    type_condition: field.type_condition,
                    edge: bound_field.response_edge(),
                    expected_key,
                    schema_field_id: field_id,
                    bound_field_id: field.bound_field_id,
                    ty,
                });
            } else {
                let type_condition = field.type_condition;
                typename_fields.push((type_condition, bound_field.response_edge()));
            }
        }

        let fields = self.push_provisional_fields(conditional_fields);
        Ok(self.push_provisional_selection_set(ConditionalSelectionSet {
            ty,
            maybe_boundary_id,
            fields,
            typename_fields,
        }))
    }

    fn push_provisional_selection_set(&mut self, selection_set: ConditionalSelectionSet) -> ConditionalSelectionSetId {
        let id = ConditionalSelectionSetId::from(self.operation.collected_conditional_selection_sets.len());
        self.operation.collected_conditional_selection_sets.push(selection_set);
        id
    }

    fn push_provisional_fields(&mut self, fields: Vec<ConditionalField>) -> IdRange<ConditionalFieldId> {
        // Can be empty when only __typename fields are present.
        if fields.is_empty() {
            return IdRange::empty();
        }
        let start = ConditionalFieldId::from(self.operation.collected_conditional_fields.len());
        self.operation.collected_conditional_fields.extend(fields);
        IdRange {
            start,
            end: ConditionalFieldId::from(self.operation.collected_conditional_fields.len()),
        }
    }

    fn push_concrete_selection_set(&mut self, selection_set: ConcreteSelectionSet) -> ConcreteSelectionSetId {
        let id = ConcreteSelectionSetId::from(self.operation.collected_concrete_selection_sets.len());
        self.operation.collected_concrete_selection_sets.push(selection_set);
        id
    }

    fn push_concrete_fields(&mut self, fields: Vec<ConcreteField>) -> IdRange<ConcreteFieldId> {
        // Can be empty when only __typename fields are present.
        if fields.is_empty() {
            return IdRange::empty();
        }
        let start = ConcreteFieldId::from(self.operation.collected_concrete_fields.len());
        self.operation.collected_concrete_fields.extend(fields);
        IdRange {
            start,
            end: ConcreteFieldId::from(self.operation.collected_concrete_fields.len()),
        }
    }
}
