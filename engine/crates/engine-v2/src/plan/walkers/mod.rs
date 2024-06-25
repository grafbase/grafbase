use std::collections::HashMap;

use schema::{EntityId, FieldDefinitionId, ObjectId, Schema, SchemaWalker};

use crate::{
    operation::{
        FieldId, Operation, OperationWalker, QueryInputValueId, QueryInputValueWalker, SelectionSetType, Variables,
    },
    plan::{CollectedField, FieldError, FieldType, RuntimeMergedConditionals},
    response::{ResponseEdge, ResponseKey, ResponseKeys, SafeResponseKey},
};

use super::{
    AnyCollectedSelectionSet, CollectedSelectionSetId, ConditionalSelectionSetId, ExecutionPlanId, OperationPlan,
    PlanInput, PlanOutput, RuntimeCollectedSelectionSet,
};

mod collected;
mod field;
mod fragment_spread;
mod inline_fragment;
mod selection_set;

pub use collected::*;
pub use field::*;
pub use fragment_spread::*;
pub use inline_fragment::*;
pub use selection_set::*;

#[derive(Clone, Copy)]
pub(crate) struct PlanWalker<'a, Item = (), SchemaItem = ()> {
    pub(super) schema_walker: SchemaWalker<'a, SchemaItem>,
    pub(super) operation_plan: &'a OperationPlan,
    pub(super) variables: &'a Variables,
    pub(super) execution_plan_id: ExecutionPlanId,
    pub(super) item: Item,
}

impl<'a> std::fmt::Debug for PlanWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanWalker").finish_non_exhaustive()
    }
}

impl<'a, I: Copy, SI> PlanWalker<'a, I, SI>
where
    Operation: std::ops::Index<I>,
{
    pub fn as_ref(&self) -> &'a <Operation as std::ops::Index<I>>::Output {
        &self.operation_plan[self.item]
    }

    #[allow(dead_code)]
    pub fn id(&self) -> I {
        self.item
    }
}

impl<'a> PlanWalker<'a, (), ()> {
    pub fn schema(&self) -> SchemaWalker<'a, ()> {
        self.schema_walker
    }

    pub fn response_keys(&self) -> &'a ResponseKeys {
        &self.operation_plan.response_keys
    }

    pub fn selection_set(self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::RootFields(self)
    }

    pub fn output(&self) -> &'a PlanOutput {
        &self.operation_plan[self.execution_plan_id].output
    }

    pub fn input(&self) -> &'a PlanInput {
        &self.operation_plan[self.execution_plan_id].input
    }

    pub fn collected_selection_set(&self) -> PlanWalker<'a, CollectedSelectionSetId, ()> {
        self.walk(self.output().collected_selection_set_id)
    }
}

impl<'a, Id> std::ops::Index<Id> for PlanWalker<'a>
where
    OperationPlan: std::ops::Index<Id>,
{
    type Output = <OperationPlan as std::ops::Index<Id>>::Output;
    fn index(&self, index: Id) -> &Self::Output {
        &self.operation_plan[index]
    }
}

impl<'a, I, SI> PlanWalker<'a, I, SI> {
    fn walk<I2>(&self, item: I2) -> PlanWalker<'a, I2, SI>
    where
        SI: Copy,
    {
        PlanWalker {
            operation_plan: self.operation_plan,
            variables: self.variables,
            execution_plan_id: self.execution_plan_id,
            schema_walker: self.schema_walker,
            item,
        }
    }

    fn walk_with<I2, SI2>(&self, item: I2, schema_item: SI2) -> PlanWalker<'a, I2, SI2> {
        PlanWalker {
            operation_plan: self.operation_plan,
            variables: self.variables,
            execution_plan_id: self.execution_plan_id,
            schema_walker: self.schema_walker.walk(schema_item),
            item,
        }
    }

    fn bound_walk_with<I2, SI2: Copy>(&self, item: I2, schema_item: SI2) -> OperationWalker<'a, I2, SI2> {
        self.operation_plan
            .operation
            .walker_with(self.schema_walker.walk(schema_item), self.variables)
            .walk(item)
    }
}

impl<'a> PlanWalker<'a, (), ()> {
    pub fn walk_input_value(&self, input_value_id: QueryInputValueId) -> QueryInputValueWalker<'a> {
        self.bound_walk_with(&self.operation_plan[input_value_id], ())
    }

    pub fn collect_fields(
        &self,
        object_id: ObjectId,
        selection_sets: &[ConditionalSelectionSetId],
    ) -> RuntimeCollectedSelectionSet {
        let schema = self.schema();

        struct GroupForResponseKey {
            edge: ResponseEdge,
            field_id: FieldId,
            expected_key: SafeResponseKey,
            definition_id: FieldDefinitionId,
            ty: FieldType<RuntimeMergedConditionals>,
        }

        let mut fields = HashMap::<ResponseKey, GroupForResponseKey>::default();
        let mut typename_fields = HashMap::<ResponseKey, ResponseEdge>::default();
        let mut field_errors = HashMap::<ResponseKey, FieldError>::new();

        for selection_set_id in selection_sets {
            let selection_set = &self.operation_plan[*selection_set_id];
            for (type_condition, edge) in &selection_set.typename_fields {
                if type_condition
                    .map(|entity_id| !does_type_condition_apply(&schema, entity_id, object_id))
                    .unwrap_or_default()
                {
                    continue;
                }
                typename_fields.entry(edge.as_response_key().unwrap()).or_insert(*edge);
            }
            for field in &self.operation_plan[selection_set.field_ids] {
                if !does_type_condition_apply(&schema, field.entity_id, object_id) {
                    continue;
                }
                fields
                    .entry(field.edge.as_response_key().unwrap())
                    .and_modify(|group| {
                        if let (FieldType::SelectionSet(selection_set), FieldType::SelectionSet(id)) =
                            (&mut group.ty, &field.ty)
                        {
                            selection_set.selection_set_ids.push(*id);
                        }
                        // Equivalent to comparing their query position. We want to keep the one
                        // with the lowest query position.
                        if field.edge < group.edge {
                            group.edge = field.edge;
                            group.field_id = field.id;
                        }
                    })
                    .or_insert_with(|| GroupForResponseKey {
                        edge: field.edge,
                        field_id: field.id,
                        expected_key: field.expected_key,
                        definition_id: field.definition_id,
                        ty: match field.ty {
                            FieldType::Scalar(scalar_type) => FieldType::Scalar(scalar_type),
                            FieldType::SelectionSet(id) => FieldType::SelectionSet(RuntimeMergedConditionals {
                                ty: SelectionSetType::maybe_from(schema.walk(field.definition_id).ty().inner().id())
                                    .unwrap(),
                                selection_set_ids: vec![id],
                            }),
                        },
                    });
            }
            for (entity_id, field_error) in &selection_set.field_errors {
                if !does_type_condition_apply(&schema, *entity_id, object_id) {
                    continue;
                }
                field_errors
                    .entry(field_error.edge.as_response_key().unwrap())
                    .and_modify(|FieldError { ref mut errors, .. }| {
                        errors.extend_from_slice(&field_error.errors);
                    })
                    .or_insert_with(|| field_error.clone());
            }
        }
        let mut fields = fields
            .into_values()
            .map(
                |GroupForResponseKey {
                     edge,
                     field_id,
                     expected_key,
                     definition_id,
                     ty,
                 }| {
                    let ty = match ty {
                        FieldType::Scalar(scalar_type) => FieldType::Scalar(scalar_type),
                        FieldType::SelectionSet(selection_set) => self.try_collect_merged_selection_sets(selection_set),
                    };
                    let wrapping = schema.walk(definition_id).ty().wrapping();
                    CollectedField {
                        edge,
                        expected_key,
                        ty,
                        id: field_id,
                        definition_id,
                        wrapping,
                    }
                },
            )
            .collect::<Vec<_>>();
        let keys = self.response_keys();
        fields.sort_unstable_by(|a, b| keys[a.expected_key].cmp(&keys[b.expected_key]));
        RuntimeCollectedSelectionSet {
            object_id,
            tracked_entity_locations: selection_sets
                .iter()
                .filter_map(|id| self.operation_plan[*id].maybe_tracked_entity_location)
                .collect(),
            fields,
            typename_fields: typename_fields.into_values().collect(),
            field_errors: field_errors.into_values().collect(),
        }
    }

    fn try_collect_merged_selection_sets(&self, selection_set: RuntimeMergedConditionals) -> FieldType {
        if let SelectionSetType::Object(object_id) = selection_set.ty {
            FieldType::SelectionSet(AnyCollectedSelectionSet::RuntimeCollected(Box::new(
                self.collect_fields(object_id, &selection_set.selection_set_ids),
            )))
        } else {
            FieldType::SelectionSet(AnyCollectedSelectionSet::RuntimeMergedConditionals(selection_set))
        }
    }
}

fn does_type_condition_apply(schema: &Schema, type_condition: EntityId, object_id: ObjectId) -> bool {
    match type_condition {
        EntityId::Object(id) => id == object_id,
        EntityId::Interface(id) => schema[id].possible_types.contains(&object_id),
    }
}
