use std::collections::{BTreeMap, HashMap};

use schema::{FieldId, ObjectId};

use crate::{
    request::{
        BoundFieldDefinitionId, BoundFieldId, BoundResponseKey, BoundSelection, BoundSelectionSetId,
        ResolvedTypeCondition, ResponseKey, SelectionSetRoot, TypeCondition,
    },
    response::{ResponseObject, ResponseValue},
};

use super::{FieldArgumentWalker, WalkerContext};

pub enum ExecutionObject<'a> {
    Concrete(ConcreteObject<'a>),
    Unknown(FieldsCollector<'a>),
}

impl<'a> ExecutionObject<'a> {
    pub fn expect_complete(self) -> ConcreteObject<'a> {
        match self {
            ExecutionObject::Concrete(group) => group,
            ExecutionObject::Unknown(_) => panic!("Expected complete grouped fields."),
        }
    }
}

pub struct ConcreteObject<'a> {
    ctx: WalkerContext<'a, ()>,
    object_id: ObjectId,
    fields: Vec<GroupForResponseKey>,
}

pub struct ExecutionField<'a> {
    bound_response_key: BoundResponseKey,
    bound_definition_id: BoundFieldDefinitionId,
    selection_set: Option<ExecutionObject<'a>>,
}

impl<'a> ConcreteObject<'a> {
    pub fn to_response_object_with(&self, f: impl Fn(ExecutionFieldWalker<'a>) -> ResponseValue) -> ResponseObject {
        ResponseObject {
            object_id: self.object_id,
            fields: self
                .fields
                .iter()
                .map(|group| {
                    let field = ExecutionFieldWalker {
                        ctx: self.ctx.walk(self.ctx.plan.operation[group.definition_id].field_id),
                        definition_id: group.definition_id,
                        selection_set: group.selection_set,
                    };
                    (group.first_bound_response_key, f(field))
                })
                .collect(),
        }
    }
}

/// Implements CollectFields from the GraphQL spec.
/// See https://spec.graphql.org/October2021/#sec-Field-Collection
pub struct FieldsCollector<'a> {
    ctx: WalkerContext<'a, ()>,
    fields: HashMap<ResponseKey, GroupForResponseKey>,
    conditional_fields: HashMap<ResolvedTypeCondition, Vec<BoundFieldId>>,
}

impl<'a> FieldsCollector<'a> {
    pub fn collection_selection_set(
        ctx: WalkerContext<'a, ()>,
        selection_set_id: BoundSelectionSetId,
    ) -> ExecutionObject<'a> {
        let root = ctx.plan.operation[selection_set_id].root;
        Self::collect(ctx, root, vec![selection_set_id])
    }

    fn collect(
        ctx: WalkerContext<'a, ()>,
        root: SelectionSetRoot,
        selection_sets: Vec<BoundSelectionSetId>,
    ) -> ExecutionObject<'a> {
        let mut collector = Self {
            ctx,
            fields: HashMap::new(),
            conditional_fields: HashMap::new(),
        };
        for id in selection_sets {
            collector.collect_fields(None, root, id);
        }

        if let SelectionSetRoot::Object(object_id) = root {
            assert!(collector.conditional_fields.is_empty());
            ExecutionObject::Concrete(ConcreteObject {
                ctx,
                object_id,
                fields: collector.fields.into_values().collect(),
            })
        } else {
            ExecutionObject::Unknown(collector)
        }
    }

    fn collect_fields(
        &mut self,
        resolved_type_condition: Option<ResolvedTypeCondition>,
        root: SelectionSetRoot,
        selection_set_id: BoundSelectionSetId,
    ) {
        for selection in &self.ctx.plan.operation[selection_set_id].items {
            match selection {
                BoundSelection::Field(id) => {
                    if self.ctx.plan.attribution[*id] == self.ctx.plan_id {
                        if let Some(resolved_type_condition) = resolved_type_condition {
                            self.conditional_fields
                                .entry(resolved_type_condition)
                                .or_default()
                                .push(*id);
                        } else {
                            let field = &self.ctx.plan.operation[*id];
                            let definition = &self.ctx.plan.operation[field.definition_id];
                            let group =
                                self.fields
                                    .entry(definition.response_key)
                                    .or_insert_with(|| GroupForResponseKey {
                                        first_bound_response_key: field.bound_response_key,
                                        definition_id: field.definition_id,
                                        selection_set: vec![],
                                    });
                            if let Some(id) = field.selection_set_id {
                                group.selection_set.push(id)
                            }
                        }
                    }
                }

                BoundSelection::FragmentSpread(spread) => {
                    if self.ctx.plan.attribution[spread.selection_set_id].contains(&self.ctx.plan_id) {
                        let type_condition = self.ctx.plan.operation[spread.fragment_id].type_condition;
                        self.collection_conditional_fields(
                            resolved_type_condition,
                            root,
                            type_condition,
                            selection_set_id,
                        )
                    }
                }
                BoundSelection::InlineFragment(fragment) => {
                    if self.ctx.plan.attribution[fragment.selection_set_id].contains(&self.ctx.plan_id) {
                        if let Some(type_condition) = fragment.type_condition {
                            self.collection_conditional_fields(
                                resolved_type_condition,
                                root,
                                type_condition,
                                selection_set_id,
                            )
                        } else {
                            self.collect_fields(resolved_type_condition, root, fragment.selection_set_id);
                        }
                    }
                }
            }
        }
    }

    fn collection_conditional_fields(
        &mut self,
        resolved_type_condition: Option<ResolvedTypeCondition>,
        root: SelectionSetRoot,
        type_condition: TypeCondition,
        selection_set_id: BoundSelectionSetId,
    ) {
        // If we know from the schema what the object type is already, we can resolve eagerly.
        if let SelectionSetRoot::Object(object_id) = root {
            match type_condition {
                TypeCondition::Interface(id) if self.ctx.schema_walker[object_id].interfaces.contains(&id) => {
                    self.collect_fields(resolved_type_condition, root, selection_set_id);
                }
                TypeCondition::Object(id) if id == object_id => {
                    self.collect_fields(resolved_type_condition, root, selection_set_id);
                }
                TypeCondition::Union(id) if self.ctx.schema_walker[id].possible_types.contains(&object_id) => {
                    self.collect_fields(resolved_type_condition, root, selection_set_id);
                }
                // nothing to collect, type condition does not apply
                _ => (),
            }
        } else {
            let resolved_type_condition = ResolvedTypeCondition::merge(
                resolved_type_condition,
                Some(type_condition.resolve(&self.ctx.schema_walker)),
            );
            let root = self.ctx.plan.operation[selection_set_id].root;
            self.collect_fields(resolved_type_condition, root, selection_set_id);
        }
    }
}

struct GroupForResponseKey {
    first_bound_response_key: BoundResponseKey,
    definition_id: BoundFieldDefinitionId,
    selection_set: Vec<BoundSelectionSetId>,
}

pub struct ExecutionFieldWalker<'a> {
    ctx: WalkerContext<'a, FieldId>,
    definition_id: BoundFieldDefinitionId,
    nested: &'a ExecutionOject<'a>,
}

impl<'a> ExecutionFieldWalker<'a> {
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

    pub fn collect_fields(&self, concrete_object_id: ObjectId) -> Option<ExecutionObject<'a>> {
        self
        let root = self.ctx.plan.operation[self.selection_set_id].root;
        FieldsCollector::collect(self.ctx.walk(()), root, self.selection_set)
    }
}

// impl<'a> IntoIterator for FieldsCollector<'a> {
//     type Item = <GroupedFieldSetIterator<'a> as Iterator>::Item;
//
//     type IntoIter = GroupedFieldSetIterator<'a>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         GroupedFieldSetIterator {
//             ctx: self.ctx,
//             grouped_fields: self.grouped_fields.into_iter(),
//         }
//     }
// }
//
// pub struct GroupedFieldSetIterator<'a> {
//     ctx: WalkerContext<'a, ()>,
//     grouped_fields: <HashMap<ResponseKey, GroupForResponseKey> as IntoIterator>::IntoIter,
// }
//
// impl<'a> Iterator for GroupedFieldSetIterator<'a> {
//     type Item = (ResponseKey, ResolvedField<'a>);
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let (key, group) = self.grouped_fields.next()?;
//         let field = ResolvedField {
//             ctx: self.ctx.walk(self.ctx.plan.operation[group.definition_id].field_id),
//             definition_id: group.definition_id,
//             selection_set: group.selection_set,
//         };
//         Some((key, field))
//     }
// }
//
//
// impl<'a> std::ops::Deref for ResolvedField<'a> {
//     type Target = schema::FieldWalker<'a>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.ctx.schema_walker
//     }
// }
//
// impl<'a> std::fmt::Debug for ResolvedField<'a> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("FieldWalker")
//             .field("name", &self.name())
//             .field("arguments", &self.bound_arguments().collect::<Vec<_>>())
//             .finish()
//     }
// }
