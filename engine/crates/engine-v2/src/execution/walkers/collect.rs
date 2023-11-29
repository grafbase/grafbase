use std::collections::HashMap;

use schema::{FieldId, ObjectId};

use super::{FieldArgumentWalker, WalkerContext};
use crate::{
    execution::StrId,
    request::{BoundFieldDefinitionId, BoundSelection, BoundSelectionSetId, TypeCondition},
    response::{ResponseObject, ResponseValue},
};

/// Implements CollectFields from the GraphQL spec.
/// See https://spec.graphql.org/October2021/#sec-Field-Collection
pub struct GroupedFieldSet<'a> {
    ctx: WalkerContext<'a, ()>,
    concrete_object_id: ObjectId,
    grouped_fields: HashMap<StrId, GroupForResponseKey>,
}

impl<'a> GroupedFieldSet<'a> {
    pub(super) fn new(ctx: WalkerContext<'a, ()>, concrete_object_id: ObjectId) -> Self {
        Self {
            ctx,
            concrete_object_id,
            grouped_fields: HashMap::new(),
        }
    }

    pub fn to_response_object_with(&self, f: impl Fn(ResolvedField<'a>) -> ResponseValue) -> ResponseObject {
        let mut fields = HashMap::with_capacity(self.grouped_fields.len());
        let object_id = Some(self.concrete_object_id);
        for (key, group) in &self.grouped_fields {
            let field = ResolvedField {
                ctx: self.ctx.walk(self.ctx.plan.operation[group.definition_id].field_id),
                definition_id: group.definition_id,
                selection_set: group.selection_set.clone(),
            };
            fields.insert(*key, f(field));
        }
        ResponseObject { object_id, fields }
    }

    pub(super) fn collect_fields(&mut self, selection_set_id: BoundSelectionSetId) {
        for selection in &self.ctx.plan.operation[selection_set_id].items {
            match selection {
                BoundSelection::Field(id) => {
                    if self.ctx.plan.attribution[*id] == self.ctx.plan_id {
                        let field = &self.ctx.plan.operation[*id];
                        let definition = &self.ctx.plan.operation[field.definition_id];
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
                    if self.ctx.plan.attribution[spread.selection_set_id].contains(&self.ctx.plan_id)
                        && self.does_fragment_type_apply(self.ctx.plan.operation[spread.fragment_id].type_condition)
                    {
                        self.collect_fields(spread.selection_set_id);
                    }
                }
                BoundSelection::InlineFragment(fragment) => {
                    if self.ctx.plan.attribution[fragment.selection_set_id].contains(&self.ctx.plan_id)
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
            TypeCondition::Interface(interface_id) => self.ctx.schema_walker[interface_id]
                .possible_types
                .contains(&self.concrete_object_id),
            TypeCondition::Object(object_id) => self.concrete_object_id == object_id,
            TypeCondition::Union(union_id) => self.ctx.schema_walker[union_id]
                .possible_types
                .contains(&self.concrete_object_id),
        }
    }
}

pub struct GroupForResponseKey {
    definition_id: BoundFieldDefinitionId,
    selection_set: Vec<BoundSelectionSetId>,
}

impl<'a> IntoIterator for GroupedFieldSet<'a> {
    type Item = <GroupedFieldSetIterator<'a> as Iterator>::Item;

    type IntoIter = GroupedFieldSetIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        GroupedFieldSetIterator {
            ctx: self.ctx,
            grouped_fields: self.grouped_fields.into_iter(),
        }
    }
}

pub struct GroupedFieldSetIterator<'a> {
    ctx: WalkerContext<'a, ()>,
    grouped_fields: <HashMap<StrId, GroupForResponseKey> as IntoIterator>::IntoIter,
}

impl<'a> Iterator for GroupedFieldSetIterator<'a> {
    type Item = (StrId, ResolvedField<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, group) = self.grouped_fields.next()?;
        let field = ResolvedField {
            ctx: self.ctx.walk(self.ctx.plan.operation[group.definition_id].field_id),
            definition_id: group.definition_id,
            selection_set: group.selection_set,
        };
        Some((key, field))
    }
}

pub struct ResolvedField<'a> {
    ctx: WalkerContext<'a, FieldId>,
    definition_id: BoundFieldDefinitionId,
    selection_set: Vec<BoundSelectionSetId>,
}

impl<'a> ResolvedField<'a> {
    pub fn bound_arguments(&self) -> impl ExactSizeIterator<Item = FieldArgumentWalker<'a>> + 'a {
        let ctx = self.ctx;
        self.ctx.plan.operation[self.definition_id]
            .arguments
            .iter()
            .map(move |argument| FieldArgumentWalker {
                ctx: ctx.walk(argument.input_value_id),
                argument,
            })
    }

    pub fn collect_fields(&self, concrete_object_id: ObjectId) -> GroupedFieldSet<'a> {
        let mut group = GroupedFieldSet::new(self.ctx.walk(()), concrete_object_id);
        for selection_set_id in &self.selection_set {
            group.collect_fields(*selection_set_id);
        }
        group
    }
}

impl<'a> std::ops::Deref for ResolvedField<'a> {
    type Target = schema::FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ctx.schema_walker
    }
}

impl<'a> std::fmt::Debug for ResolvedField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldWalker")
            .field("name", &self.name())
            .field("arguments", &self.bound_arguments().collect::<Vec<_>>())
            .finish()
    }
}
