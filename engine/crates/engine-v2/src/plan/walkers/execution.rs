use std::collections::HashMap;

use schema::{FieldId, ObjectId, SchemaWalker};

use crate::{
    execution::StrId,
    plan::{OperationPlan, PlanId},
    request::{
        BoundFieldDefinitionId, BoundSelection, BoundSelectionSetId, OperationFieldArgumentWalker, TypeCondition,
    },
    response::{ResponseObject, ResponseValue},
};

/// Implements CollectFields from the GraphQL spec.
/// See https://spec.graphql.org/October2021/#sec-Field-Collection
pub struct GroupedFieldSet<'a> {
    schema: SchemaWalker<'a, ()>,
    plan: &'a OperationPlan,
    plan_id: PlanId,
    concrete_object_id: ObjectId,
    grouped_fields: HashMap<StrId, GroupForResponseKey>,
}

impl<'a> GroupedFieldSet<'a> {
    pub(super) fn new(
        schema: SchemaWalker<'a, ()>,
        plan: &'a OperationPlan,
        plan_id: PlanId,
        concrete_object_id: ObjectId,
    ) -> Self {
        Self {
            schema,
            plan,
            plan_id,
            concrete_object_id,
            grouped_fields: HashMap::new(),
        }
    }

    pub fn to_response_object_with(&self, f: impl Fn(ResolvedField<'a>) -> ResponseValue) -> ResponseObject {
        let mut fields = HashMap::with_capacity(self.grouped_fields.len());
        let object_id = Some(self.concrete_object_id);
        for (key, group) in &self.grouped_fields {
            let field = ResolvedField {
                schema: self.schema,
                plan: self.plan,
                plan_id: self.plan_id,
                definition_id: group.definition_id,
                selection_set: group.selection_set.clone(),
            };
            fields.insert(*key, f(field));
        }
        ResponseObject { object_id, fields }
    }

    pub(super) fn collect_fields(&mut self, selection_set_id: BoundSelectionSetId) {
        for selection in &self.plan.operation[selection_set_id].items {
            match selection {
                BoundSelection::Field(id) => {
                    if self.plan.attribution[*id] == self.plan_id {
                        let field = &self.plan.operation[*id];
                        let definition = &self.plan.operation[field.definition_id];
                        self.grouped_fields
                            .entry(definition.name)
                            .or_insert_with(|| GroupForResponseKey {
                                definition_id: field.definition_id,
                                selection_set: vec![],
                            })
                            .selection_set
                            .push(field.selection_set_id);
                    }
                }

                BoundSelection::FragmentSpread(spread) => {
                    if self.plan.attribution[spread.selection_set_id].contains(&self.plan_id)
                        && self.does_fragment_type_apply(self.plan.operation[spread.fragment_id].type_condition)
                    {
                        self.collect_fields(spread.selection_set_id);
                    }
                }
                BoundSelection::InlineFragment(fragment) => {
                    if self.plan.attribution[fragment.selection_set_id].contains(&self.plan_id)
                        && fragment
                            .type_condition
                            .map(|cond| self.does_fragment_type_apply(cond))
                            .unwrap_or(true)
                    {
                        self.collect_fields(fragment.selection_set_id);
                    }
                }
            }
        }
    }

    fn does_fragment_type_apply(&self, type_condition: TypeCondition) -> bool {
        match type_condition {
            TypeCondition::Interface(interface_id) => self.schema[interface_id]
                .possible_types
                .contains(&self.concrete_object_id),
            TypeCondition::Object(object_id) => self.concrete_object_id == object_id,
            TypeCondition::Union(union_id) => self.schema[union_id].possible_types.contains(&self.concrete_object_id),
        }
    }
}

pub struct GroupForResponseKey {
    definition_id: BoundFieldDefinitionId,
    selection_set: Vec<BoundSelectionSetId>,
}

pub struct ResolvedField<'a> {
    schema: SchemaWalker<'a, ()>,
    plan: &'a OperationPlan,
    plan_id: PlanId,
    definition_id: BoundFieldDefinitionId,
    selection_set: Vec<BoundSelectionSetId>,
}

impl<'a> ResolvedField<'a> {
    pub fn id(&self) -> FieldId {
        self.plan.operation[self.definition_id].field_id
    }

    pub fn bound_arguments(&self) -> impl ExactSizeIterator<Item = OperationFieldArgumentWalker<'a>> + 'a {
        let walker = self.schema;
        self.plan.operation[self.definition_id]
            .arguments
            .iter()
            .map(move |argument| OperationFieldArgumentWalker::new(walker.walk(argument.input_value_id), argument))
    }

    pub fn collect_fields(&self, concrete_object_id: ObjectId) -> GroupedFieldSet<'a> {
        let mut group = GroupedFieldSet::new(self.schema, self.plan, self.plan_id, concrete_object_id);
        for selection_set_id in &self.selection_set {
            group.collect_fields(*selection_set_id);
        }
        group
    }
}

impl<'a> IntoIterator for GroupedFieldSet<'a> {
    type Item = <GroupedFieldSetIterator<'a> as Iterator>::Item;

    type IntoIter = GroupedFieldSetIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        GroupedFieldSetIterator {
            schema: self.schema,
            plan: self.plan,
            plan_id: self.plan_id,
            grouped_fields: self.grouped_fields.into_iter(),
        }
    }
}

pub struct GroupedFieldSetIterator<'a> {
    schema: SchemaWalker<'a, ()>,
    plan: &'a OperationPlan,
    plan_id: PlanId,
    grouped_fields: <HashMap<StrId, GroupForResponseKey> as IntoIterator>::IntoIter,
}

impl<'a> Iterator for GroupedFieldSetIterator<'a> {
    type Item = (StrId, ResolvedField<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, group) = self.grouped_fields.next()?;
        let field = ResolvedField {
            schema: self.schema,
            plan: self.plan,
            plan_id: self.plan_id,
            definition_id: group.definition_id,
            selection_set: group.selection_set,
        };
        Some((key, field))
    }
}
