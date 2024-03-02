use id_newtypes::IdRange;
use itertools::Itertools;
use schema::Schema;
use std::collections::HashSet;

use crate::{
    plan::{
        flatten_selection_sets, AnyCollectedSelectionSet, AnyCollectedSelectionSetId, CollectedField, CollectedFieldId,
        CollectedSelectionSet, CollectedSelectionSetId, ConditionalField, ConditionalFieldId, ConditionalSelectionSet,
        ConditionalSelectionSetId, EntityType, FieldType, FlatField, FlatTypeCondition, OperationPlan, PlanBoundaryId,
        PlanId,
    },
    request::{BoundFieldId, BoundSelectionSetId, OperationWalker, SelectionSetType},
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
            .walk(operation.planned_resolvers[usize::from(plan_id)].resolver_id)
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
    ) -> PlanningResult<CollectedSelectionSetId> {
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
        self.collect_fields(ty, fields, None)
    }

    fn find_root_fields(&self, root_selection_set_ids: Vec<BoundSelectionSetId>) -> Vec<BoundFieldId> {
        let walker = self.walker();
        root_selection_set_ids
            .into_iter()
            .flat_map(|id| walker.walk(id).fields())
            .filter_map(|field| {
                let field_plan_id = self.operation.bound_field_to_plan_id[usize::from(field.id())];
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
    ) -> PlanningResult<AnyCollectedSelectionSet> {
        let selection_set = flatten_selection_sets(self.schema, self.operation, selection_set_ids);

        let mut maybe_boundary_id = None;
        let mut plan_fields = Vec::new();
        for field in selection_set.fields {
            if !self.operation[field.bound_field_id].is_read() {
                continue;
            }

            let field_plan_id = self.operation.bound_field_to_plan_id[usize::from(field.bound_field_id)];
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

        let mut conditions = HashSet::<Option<EntityType>>::default();
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
        let id = if concrete_parent && !too_complex && conditions.len() == 1 && conditions.contains(&None) {
            self.collect_fields(
                selection_set.ty,
                plan_fields.into_iter().map(|field| field.bound_field_id).collect(),
                maybe_boundary_id,
            )
            .map(AnyCollectedSelectionSetId::Collected)?
        } else {
            self.collected_conditional_fields(selection_set.ty, plan_fields, maybe_boundary_id)
                .map(AnyCollectedSelectionSetId::Conditional)?
        };

        // We keep track of which collected selection set matches which bound selection sets.
        // This allows us to know whether `__typename` is necessary in the generated subgraph query.
        for root_id in selection_set.root_selection_set_ids {
            self.operation.bound_to_collected_selection_set[usize::from(root_id)] = Some(id);
        }
        Ok(match id {
            AnyCollectedSelectionSetId::Collected(id) => AnyCollectedSelectionSet::Collected(id),
            AnyCollectedSelectionSetId::Conditional(id) => AnyCollectedSelectionSet::Conditional(id),
        })
    }

    fn collect_fields(
        &mut self,
        ty: SelectionSetType,
        fields: Vec<BoundFieldId>,
        maybe_boundary_id: Option<PlanBoundaryId>,
    ) -> PlanningResult<CollectedSelectionSetId> {
        let grouped_by_response_key = self
            .walker()
            .group_by_response_key_sorted_by_query_position(fields)
            .into_values();

        let mut fields = vec![];
        let mut typename_fields = vec![];
        for bound_field_ids in grouped_by_response_key {
            let bound_field_id: BoundFieldId = bound_field_ids[0];
            let bound_field = self.operation[bound_field_id].clone();
            if let Some(schema_field_id) = bound_field.schema_field_id() {
                let schema_field = self.schema.walk(schema_field_id);
                let expected_key = if self.support_aliases {
                    self.operation.response_keys.ensure_safety(bound_field.response_key())
                } else {
                    self.operation.response_keys.get_or_intern(schema_field.name())
                };
                let ty = match schema_field.ty().inner().scalar_type() {
                    Some(scalar_type) => FieldType::Scalar(scalar_type),
                    None => {
                        let subselection_set_ids = bound_field_ids
                            .into_iter()
                            .filter_map(|id| self.operation[id].selection_set_id())
                            .collect();
                        FieldType::SelectionSet(self.collect_selection_set(subselection_set_ids, true)?)
                    }
                };
                fields.push(CollectedField {
                    expected_key,
                    edge: bound_field.response_edge(),
                    bound_field_id,
                    schema_field_id,
                    wrapping: schema_field.ty().wrapping(),
                    ty,
                });
            } else {
                typename_fields.push(bound_field.response_edge());
            }
        }

        // Sorting by expected_key for deserialization
        let keys = &self.operation.response_keys;
        fields.sort_unstable_by(|a, b| keys[a.expected_key].cmp(&keys[b.expected_key]));
        let field_ids = self.push_collecteded_fields(fields);
        Ok(self.push_collected_selection_set(CollectedSelectionSet {
            ty,
            maybe_boundary_id,
            field_ids,
            typename_fields,
        }))
    }

    fn collected_conditional_fields(
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
                    self.operation.response_keys.ensure_safety(bound_field.response_key())
                } else {
                    self.operation.response_keys.get_or_intern(schema_field.name())
                };
                let ty = match schema_field.ty().inner().scalar_type() {
                    Some(data_type) => FieldType::Scalar(data_type),
                    None => {
                        let selection_set_id =
                            self.collect_selection_set(bound_field.selection_set_id().into_iter().collect(), false)?;
                        let AnyCollectedSelectionSet::Conditional(selection_set_id) = selection_set_id else {
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

        let field_ids = self.push_conditional_fields(conditional_fields);
        Ok(self.push_conditional_selection_set(ConditionalSelectionSet {
            ty,
            maybe_boundary_id,
            field_ids,
            typename_fields,
        }))
    }

    fn push_conditional_selection_set(&mut self, selection_set: ConditionalSelectionSet) -> ConditionalSelectionSetId {
        let id = ConditionalSelectionSetId::from(self.operation.conditional_selection_sets.len());
        self.operation.conditional_selection_sets.push(selection_set);
        id
    }

    fn push_conditional_fields(&mut self, fields: Vec<ConditionalField>) -> IdRange<ConditionalFieldId> {
        // Can be empty when only __typename fields are present.
        if fields.is_empty() {
            return IdRange::empty();
        }
        let start = ConditionalFieldId::from(self.operation.conditional_fields.len());
        self.operation.conditional_fields.extend(fields);
        IdRange {
            start,
            end: ConditionalFieldId::from(self.operation.conditional_fields.len()),
        }
    }

    fn push_collected_selection_set(&mut self, selection_set: CollectedSelectionSet) -> CollectedSelectionSetId {
        let id = CollectedSelectionSetId::from(self.operation.collected_selection_sets.len());
        self.operation.collected_selection_sets.push(selection_set);
        id
    }

    fn push_collecteded_fields(&mut self, fields: Vec<CollectedField>) -> IdRange<CollectedFieldId> {
        // Can be empty when only __typename fields are present.
        if fields.is_empty() {
            return IdRange::empty();
        }
        let start = CollectedFieldId::from(self.operation.collected_fields.len());
        self.operation.collected_fields.extend(fields);
        IdRange {
            start,
            end: CollectedFieldId::from(self.operation.collected_fields.len()),
        }
    }
}
