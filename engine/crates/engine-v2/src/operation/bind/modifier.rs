use std::{collections::HashMap, ops::Range};

use id_newtypes::IdRange;
use schema::{Definition, FieldDefinitionWalker, ObjectId, TypeSystemDirective};

use crate::operation::{
    FieldArgumentId, FieldId, QueryModifier, QueryModifierId, QueryModifierRule, ResponseModifier, ResponseModifierId,
    ResponseModifierRule,
};

impl<'schema, 'p> super::Binder<'schema, 'p> {
    pub(super) fn generate_field_modifiers(
        &mut self,
        field_id: FieldId,
        argument_ids: IdRange<FieldArgumentId>,
        definition: FieldDefinitionWalker<'_>,
    ) {
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
                    match (directive.fields.is_some(), directive.node.is_some()) {
                        (true, true) => {
                            unreachable!("Authorized directive with both fields and node isn't supported yet");
                        }
                        (true, false) => {
                            self.register_field_impacted_by_response_modifier(
                                ResponseModifierRule::AuthorizedParentEdge {
                                    directive_id: *id,
                                    definition_id: definition.id(),
                                },
                                field_id,
                            );
                        }
                        (false, true) => {
                            self.register_field_impacted_by_response_modifier(
                                ResponseModifierRule::AuthorizedEdgeChild {
                                    directive_id: *id,
                                    definition_id: definition.id(),
                                },
                                field_id,
                            );
                        }
                        (false, false) => {
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

    fn register_field_impacted_by_response_modifier(&mut self, rule: ResponseModifierRule, field_id: FieldId) {
        let n = self.response_modifiers.len();
        self.response_modifiers
            .entry(rule)
            .or_insert((n.into(), Vec::new()))
            .1
            .push(field_id);
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
    finalize_modifiers(query_modifiers, |rule, ids_range| QueryModifier {
        rule,
        impacted_fields: IdRange::from(ids_range),
    })
}

pub(super) fn finalize_response_modifiers(
    response_modifiers: HashMap<ResponseModifierRule, (ResponseModifierId, Vec<FieldId>)>,
) -> (Vec<ResponseModifier>, Vec<FieldId>) {
    finalize_modifiers(response_modifiers, |rule, ids_range| ResponseModifier {
        rule,
        impacted_fields: IdRange::from(ids_range),
    })
}

fn finalize_modifiers<Rule, Id: Ord + Copy, Modifier>(
    modifiers: HashMap<Rule, (Id, Vec<FieldId>)>,
    build: impl Fn(Rule, Range<usize>) -> Modifier,
) -> (Vec<Modifier>, Vec<FieldId>) where {
    let mut query_modifiers = modifiers
        .into_iter()
        .map(|(rule, (id, fields))| (id, rule, fields))
        .collect::<Vec<_>>();
    query_modifiers.sort_unstable_by_key(|(id, _, _)| *id);

    let n = query_modifiers.len();
    query_modifiers.into_iter().fold(
        (Vec::with_capacity(n), Vec::with_capacity(n * 2)),
        |(mut modifiers, mut impact_field_ids), (_, rule, field_ids)| {
            let start = impact_field_ids.len();
            impact_field_ids.extend(field_ids);
            modifiers.push(build(rule, start..impact_field_ids.len()));
            (modifiers, impact_field_ids)
        },
    )
}
