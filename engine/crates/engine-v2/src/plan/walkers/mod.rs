use std::collections::HashMap;

use schema::{FieldId, ObjectId, Schema, SchemaWalker};

use crate::{
    execution::Variables,
    plan::{CollectedField, FieldType, RuntimeMergedConditionals},
    request::{
        BoundFieldId, FlatTypeCondition, Operation, OperationWalker, SelectionSetType, VariableDefinitionWalker,
    },
    response::{ResponseEdge, ResponseKey, ResponseKeys, ResponsePart, ResponsePath, SeedContext},
};

use super::{
    AnyCollectedSelectionSet, CollectedSelectionSetId, ConditionalSelectionSetId, OperationPlan, PlanId, PlanInput,
    PlanOutput, RuntimeCollectedSelectionSet,
};

mod argument;
mod collected;
mod field;
mod fragment_spread;
mod inline_fragment;
mod selection_set;

pub use argument::*;
pub use collected::*;
pub use field::*;
pub use fragment_spread::*;
pub use inline_fragment::*;
pub use selection_set::*;

#[derive(Clone, Copy)]
pub(crate) struct PlanWalker<'a, Item = (), SchemaItem = ()> {
    pub(super) schema_walker: SchemaWalker<'a, SchemaItem>,
    pub(super) operation_plan: &'a OperationPlan,
    pub(super) variables: Option<&'a Variables>,
    pub(super) plan_id: PlanId,
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

impl<'a> PlanWalker<'a> {
    pub fn schema(&self) -> SchemaWalker<'a> {
        self.schema_walker
    }

    pub fn response_keys(&self) -> &'a ResponseKeys {
        &self.operation_plan.response_keys
    }

    pub fn operation(&self) -> OperationWalker<'a> {
        self.operation_plan.operation.walker_with(self.schema_walker.walk(()))
    }

    pub fn selection_set(self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::RootFields(self)
    }

    pub fn id(&self) -> PlanId {
        self.plan_id
    }

    pub fn output(&self) -> &'a PlanOutput {
        &self.operation_plan.plan_outputs[usize::from(self.plan_id)]
    }

    pub fn input(&self) -> Option<&'a PlanInput> {
        self.operation_plan.plan_inputs[usize::from(self.plan_id)].as_ref()
    }

    pub fn collected_selection_set(&self) -> PlanWalker<'a, CollectedSelectionSetId, ()> {
        self.walk(self.output().collected_selection_set_id)
    }

    pub fn variable_definition(&self, name: &str) -> Option<VariableDefinitionWalker<'a>> {
        self.bound_walk_with((), ()).variable_definition(name)
    }

    pub fn new_seed<'out>(self, output: &'out mut ResponsePart) -> SeedContext<'out>
    where
        'a: 'out,
    {
        SeedContext::new(self, output)
    }

    pub fn root_error_path(&self, parent: &ResponsePath) -> ResponsePath {
        let mut fields = self.collected_selection_set().fields();
        if fields.len() == 1 {
            parent.child(fields.next().unwrap().as_bound_field().response_edge())
        } else {
            parent.clone()
        }
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
    pub fn walk<I2>(&self, item: I2) -> PlanWalker<'a, I2, SI>
    where
        SI: Copy,
    {
        PlanWalker {
            operation_plan: self.operation_plan,
            variables: self.variables,
            plan_id: self.plan_id,
            schema_walker: self.schema_walker,
            item,
        }
    }

    pub fn walk_with<I2, SI2>(&self, item: I2, schema_item: SI2) -> PlanWalker<'a, I2, SI2> {
        PlanWalker {
            operation_plan: self.operation_plan,
            variables: self.variables,
            plan_id: self.plan_id,
            schema_walker: self.schema_walker.walk(schema_item),
            item,
        }
    }

    pub fn bound_walk_with<I2, SI2: Copy>(&self, item: I2, schema_item: SI2) -> OperationWalker<'a, I2, SI2> {
        self.operation_plan
            .operation
            .walker_with(self.schema_walker.walk(schema_item))
            .walk(item)
    }
}

impl<'a> PlanWalker<'a> {
    pub fn collect_fields(
        &self,
        object_id: ObjectId,
        selection_sets: &[ConditionalSelectionSetId],
    ) -> RuntimeCollectedSelectionSet {
        let schema = self.schema();

        struct GroupForResponseKey {
            edge: ResponseEdge,
            bound_field_id: BoundFieldId,
            expected_key: ResponseKey,
            schema_field_id: FieldId,
            ty: FieldType<RuntimeMergedConditionals>,
        }

        let mut fields = HashMap::<ResponseKey, GroupForResponseKey>::default();
        let mut typename_fields = HashMap::<ResponseKey, ResponseEdge>::default();

        for selection_set_id in selection_sets {
            let selection_set = &self[*selection_set_id];
            for (type_condition, edge) in &selection_set.typename_fields {
                if !does_type_condition_apply(&schema, type_condition, object_id) {
                    continue;
                }
                typename_fields.entry(edge.as_response_key().unwrap()).or_insert(*edge);
            }
            for field in &self[selection_set.fields] {
                if !does_type_condition_apply(&schema, &field.type_condition, object_id) {
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
                            group.bound_field_id = field.bound_field_id;
                        }
                    })
                    .or_insert_with(|| GroupForResponseKey {
                        edge: field.edge,
                        bound_field_id: field.bound_field_id,
                        expected_key: field.expected_key,
                        schema_field_id: field.schema_field_id,
                        ty: match field.ty {
                            FieldType::Scalar(data_type) => FieldType::Scalar(data_type),
                            FieldType::SelectionSet(id) => FieldType::SelectionSet(RuntimeMergedConditionals {
                                ty: SelectionSetType::maybe_from(schema.walk(field.schema_field_id).ty().inner().id())
                                    .unwrap(),
                                selection_set_ids: vec![id],
                            }),
                        },
                    });
            }
        }
        let mut fields = fields
            .into_values()
            .map(
                |GroupForResponseKey {
                     edge,
                     bound_field_id,
                     expected_key,
                     schema_field_id,
                     ty,
                 }| {
                    let ty = match ty {
                        FieldType::Scalar(data_type) => FieldType::Scalar(data_type),
                        FieldType::SelectionSet(selection_set) => self.try_collect_merged_selection_sets(selection_set),
                    };
                    let wrapping = schema.walk(schema_field_id).ty().wrapping();
                    CollectedField {
                        edge,
                        expected_key,
                        ty,
                        bound_field_id,
                        schema_field_id,
                        wrapping,
                    }
                },
            )
            .collect::<Vec<_>>();
        let keys = self.response_keys();
        fields.sort_unstable_by(|a, b| keys[a.expected_key].cmp(&keys[b.expected_key]));
        RuntimeCollectedSelectionSet {
            object_id,
            boundary_ids: selection_sets
                .iter()
                .filter_map(|id| self[*id].maybe_boundary_id)
                .collect(),
            fields,
            typename_fields: typename_fields.into_values().collect(),
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

fn does_type_condition_apply(schema: &Schema, type_condition: &Option<FlatTypeCondition>, object_id: ObjectId) -> bool {
    type_condition
        .as_ref()
        .map(|cond| cond.matches(schema, object_id))
        .unwrap_or(true)
}
