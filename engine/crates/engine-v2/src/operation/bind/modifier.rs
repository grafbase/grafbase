use std::collections::HashMap;

use id_newtypes::IdRange;
use schema::{Definition, FieldDefinitionWalker, ObjectId, TypeSystemDirective};

use crate::operation::{
    FieldArgumentId, FieldId, QueryModifier, QueryModifierId, QueryModifierRule, ResponseModifierRule,
    SubjectToResponseModifierRuleId,
};

impl<'schema, 'p> super::Binder<'schema, 'p> {
    pub(super) fn generate_field_modifiers(
        &mut self,
        field_id: FieldId,
        argument_ids: IdRange<FieldArgumentId>,
        definition: FieldDefinitionWalker<'_>,
    ) -> IdRange<SubjectToResponseModifierRuleId> {
        let response_modifiers_start = self.response_modifier_rules.len();

        for directive in definition.directives().as_ref().iter() {
            match directive {
                TypeSystemDirective::Authenticated => {
                    self.register_field_impacted_by_query_modifier(QueryModifierRule::Authenticated, field_id);
                }
                TypeSystemDirective::RequiresScopes(id) => {
                    self.register_field_impacted_by_query_modifier(QueryModifierRule::RequiresScopes(*id), field_id);
                }
                TypeSystemDirective::Authorized(id) => {
                    let directive = &self.schema[*id];
                    if directive.fields.is_some() {
                        self.register_field_subject_to_response_modifier_rule(ResponseModifierRule::AuthorizedField {
                            directive_id: *id,
                            definition_id: definition.id(),
                        });
                    } else {
                        self.register_field_impacted_by_query_modifier(
                            QueryModifierRule::AuthorizedField {
                                directive_id: *id,
                                definition_id: definition.id(),
                                argument_ids,
                            },
                            field_id,
                        );
                    }
                }
                _ => {}
            }
        }

        let output_definition = definition.ty().inner();
        for directive in output_definition.directives().as_ref() {
            match directive {
                TypeSystemDirective::Authenticated => {
                    self.register_field_impacted_by_query_modifier(QueryModifierRule::Authenticated, field_id);
                }
                TypeSystemDirective::RequiresScopes(id) => {
                    self.register_field_impacted_by_query_modifier(QueryModifierRule::RequiresScopes(*id), field_id);
                }
                TypeSystemDirective::Authorized(id) => {
                    self.register_field_impacted_by_query_modifier(
                        QueryModifierRule::AuthorizedDefinition {
                            directive_id: *id,
                            definition: output_definition.id(),
                        },
                        field_id,
                    );
                }
                _ => {}
            }
        }

        let response_modifiers_end = self.response_modifier_rules.len();
        IdRange::from(response_modifiers_start..response_modifiers_end)
    }

    pub(super) fn generate_modifiers_for_root_object(&mut self, root_object_id: ObjectId) -> Vec<QueryModifierId> {
        let mut modifiers = Vec::new();
        for directive in self.schema.walk(root_object_id).directives().as_ref() {
            match directive {
                TypeSystemDirective::Authenticated => {
                    modifiers.push(self.push_root_object_query_modifier(QueryModifierRule::Authenticated));
                }
                TypeSystemDirective::RequiresScopes(id) => {
                    modifiers.push(self.push_root_object_query_modifier(QueryModifierRule::RequiresScopes(*id)));
                }
                TypeSystemDirective::Authorized(id) => {
                    modifiers.push(
                        self.push_root_object_query_modifier(QueryModifierRule::AuthorizedDefinition {
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

    fn register_field_subject_to_response_modifier_rule(&mut self, rule: ResponseModifierRule) {
        let n = self.response_modifier_rules.len();
        let id = *self.response_modifier_rules.entry(rule).or_insert(n.into());
        self.fields_subject_to_response_modifier_rules.push(id);
    }

    fn register_field_impacted_by_query_modifier(&mut self, rule: QueryModifierRule, field_id: FieldId) {
        let n = self.query_modifiers.len();
        self.query_modifiers
            .entry(rule)
            .or_insert((n.into(), Vec::new()))
            .1
            .push(field_id);
    }

    fn push_root_object_query_modifier(&mut self, rule: QueryModifierRule) -> QueryModifierId {
        let n = self.query_modifiers.len();
        self.query_modifiers.entry(rule).or_insert((n.into(), Vec::new())).0
    }
}

pub(super) fn finalize_query_modifiers(
    query_modifiers: HashMap<QueryModifierRule, (QueryModifierId, Vec<FieldId>)>,
) -> (Vec<QueryModifier>, Vec<FieldId>) {
    let mut query_modifiers = query_modifiers
        .into_iter()
        .map(|(rule, (id, fields))| (id, rule, fields))
        .collect::<Vec<_>>();
    query_modifiers.sort_unstable_by_key(|(id, _, _)| *id);

    let n = query_modifiers.len();
    query_modifiers.into_iter().fold(
        (Vec::with_capacity(n), Vec::with_capacity(n * 2)),
        |(mut modifiers, mut impact_field_ids), (_, modifier, field_ids)| {
            let start = impact_field_ids.len();
            impact_field_ids.extend(field_ids);
            modifiers.push(QueryModifier {
                rule: modifier,
                impacted_fields: IdRange::from(start..impact_field_ids.len()),
            });
            (modifiers, impact_field_ids)
        },
    )
}
