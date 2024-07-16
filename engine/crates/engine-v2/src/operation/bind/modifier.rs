use std::collections::HashMap;

use id_newtypes::IdRange;
use schema::{Definition, FieldDefinitionWalker, ObjectId, TypeSystemDirective};

use crate::operation::{FieldId, QueryModifier, QueryModifierCondition, QueryModifierId};

impl<'schema, 'p> super::Binder<'schema, 'p> {
    pub(super) fn generate_field_modifiers(&mut self, field_id: FieldId, definition: FieldDefinitionWalker<'_>) {
        for directive in definition.directives().as_ref().iter() {
            match directive {
                TypeSystemDirective::Authenticated => {
                    self.push_field_modifier(QueryModifierCondition::Authenticated, field_id);
                }
                TypeSystemDirective::RequiresScopes(id) => {
                    self.push_field_modifier(QueryModifierCondition::RequiresScopes(*id), field_id);
                }
                TypeSystemDirective::Authorized(id) => {
                    self.push_field_modifier(
                        QueryModifierCondition::AuthorizedField {
                            directive_id: *id,
                            definition_id: definition.id(),
                            argument_ids: if !self.schema[*id].arguments.is_empty() {
                                self[field_id].argument_ids()
                            } else {
                                Default::default()
                            },
                        },
                        field_id,
                    );
                }
                _ => {}
            }
        }

        let output_definition = definition.ty().inner();
        for directive in output_definition.directives().as_ref() {
            match directive {
                TypeSystemDirective::Authenticated => {
                    self.push_field_modifier(QueryModifierCondition::Authenticated, field_id);
                }
                TypeSystemDirective::RequiresScopes(id) => {
                    self.push_field_modifier(QueryModifierCondition::RequiresScopes(*id), field_id);
                }
                TypeSystemDirective::Authorized(id) => {
                    self.push_field_modifier(
                        QueryModifierCondition::AuthorizedDefinition {
                            directive_id: *id,
                            definition: output_definition.id(),
                        },
                        field_id,
                    );
                }
                _ => {}
            }
        }
    }

    pub(super) fn generate_modifiers_for_root_object(&mut self, root_object_id: ObjectId) -> Vec<QueryModifierId> {
        let mut modifiers = Vec::new();
        for directive in self.schema.walk(root_object_id).directives().as_ref() {
            match directive {
                TypeSystemDirective::Authenticated => {
                    modifiers.push(self.push_root_object_modifier(QueryModifierCondition::Authenticated));
                }
                TypeSystemDirective::RequiresScopes(id) => {
                    modifiers.push(self.push_root_object_modifier(QueryModifierCondition::RequiresScopes(*id)));
                }
                TypeSystemDirective::Authorized(id) => {
                    modifiers.push(
                        self.push_root_object_modifier(QueryModifierCondition::AuthorizedDefinition {
                            directive_id: *id,
                            definition: Definition::Object(root_object_id),
                        }),
                    );
                }
                _ => {}
            }
        }
        modifiers.sort_unstable();
        modifiers
    }

    fn push_field_modifier(&mut self, modifier: QueryModifierCondition, field_id: FieldId) {
        let n = self.query_modifiers.len();
        self.query_modifiers
            .entry(modifier)
            .or_insert((n.into(), Vec::new()))
            .1
            .push(field_id);
    }

    fn push_root_object_modifier(&mut self, modifier: QueryModifierCondition) -> QueryModifierId {
        let n = self.query_modifiers.len();
        self.query_modifiers.entry(modifier).or_insert((n.into(), Vec::new())).0
    }
}

pub(super) fn finalize_query_modifiers(
    query_modifiers: HashMap<QueryModifierCondition, (QueryModifierId, Vec<FieldId>)>,
) -> (Vec<QueryModifier>, Vec<FieldId>) {
    let mut query_modifiers = query_modifiers
        .into_iter()
        .map(|(condition, (id, fields))| (id, condition, fields))
        .collect::<Vec<_>>();
    query_modifiers.sort_unstable_by_key(|(id, _, _)| *id);

    let n = query_modifiers.len();
    query_modifiers.into_iter().fold(
        (Vec::with_capacity(n), Vec::with_capacity(n * 2)),
        |(mut modifiers, mut impact_field_ids), (_, modifier, field_ids)| {
            let start = impact_field_ids.len();
            impact_field_ids.extend(field_ids);
            modifiers.push(QueryModifier {
                condition: modifier,
                impacted_fields: IdRange::from(start..impact_field_ids.len()),
            });
            (modifiers, impact_field_ids)
        },
    )
}
