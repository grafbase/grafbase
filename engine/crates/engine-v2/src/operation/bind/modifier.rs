use std::{collections::HashMap, ops::Range};

use id_newtypes::IdRange;
use schema::{DefinitionId, FieldDefinition, ObjectDefinitionId, TypeSystemDirective};

use crate::operation::{
    FieldArgumentId, FieldId, QueryModifier, QueryModifierId, QueryModifierRule, ResponseModifier, ResponseModifierId,
    ResponseModifierRule,
};

impl<'schema, 'p> super::Binder<'schema, 'p> {
    /// Generates query modifiers for a specific field based on its directives.
    ///
    /// This function examines the directives associated with the provided field definition
    /// and registers corresponding query modifiers. Different directives may indicate
    /// requirements for authentication, scope, or authorization, which are processed
    /// accordingly.
    ///
    /// # Parameters
    ///
    /// - `field_id`: The identifier for the field being processed.
    /// - `argument_ids`: A range of field argument identifiers associated with the field.
    /// - `field_definition`: The definition of the field, encapsulating its properties and directives.
    pub(super) fn generate_field_modifiers(
        &mut self,
        field_id: FieldId,
        argument_ids: IdRange<FieldArgumentId>,
        field_definition: FieldDefinition<'_>,
    ) {
        for directive in field_definition.directives() {
            match directive {
                TypeSystemDirective::Authenticated => {
                    self.register_field_impacted_by_query_modifier(QueryModifierRule::Authenticated, field_id);
                }
                TypeSystemDirective::RequiresScopes(directive) => {
                    self.register_field_impacted_by_query_modifier(
                        QueryModifierRule::RequiresScopes(directive.id()),
                        field_id,
                    );
                }
                TypeSystemDirective::Authorized(directive) => {
                    match (directive.fields().is_some(), directive.node().is_some()) {
                        (true, true) => {
                            unreachable!("Authorized directive with both fields and node isn't supported yet");
                        }
                        (true, false) => {
                            self.register_field_impacted_by_response_modifier(
                                ResponseModifierRule::AuthorizedParentEdge {
                                    directive_id: directive.id(),
                                    definition_id: field_definition.id(),
                                },
                                field_id,
                            );
                        }
                        (false, true) => {
                            self.register_field_impacted_by_response_modifier(
                                ResponseModifierRule::AuthorizedEdgeChild {
                                    directive_id: directive.id(),
                                    definition_id: field_definition.id(),
                                },
                                field_id,
                            );
                        }
                        (false, false) => {
                            self.register_field_impacted_by_query_modifier(
                                QueryModifierRule::AuthorizedField {
                                    directive_id: directive.id(),
                                    definition_id: field_definition.id(),
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

        for directive in field_definition.ty().definition().directives() {
            match directive {
                TypeSystemDirective::Authenticated => {
                    self.register_field_impacted_by_query_modifier(QueryModifierRule::Authenticated, field_id);
                }
                TypeSystemDirective::RequiresScopes(directive) => {
                    self.register_field_impacted_by_query_modifier(
                        QueryModifierRule::RequiresScopes(directive.id()),
                        field_id,
                    );
                }
                TypeSystemDirective::Authorized(directive) => {
                    self.register_field_impacted_by_query_modifier(
                        QueryModifierRule::AuthorizedDefinition {
                            directive_id: directive.id(),
                            definition_id: field_definition.ty().as_ref().definition_id,
                        },
                        field_id,
                    );
                }
                _ => {}
            }
        }
    }

    /// Generates query modifiers for the root object based on its directives.
    ///
    /// This function examines the directives associated with the given root object
    /// definition and registers corresponding query modifiers. Each directive may
    /// indicate different requirements for authentication, scope, or authorization,
    /// which are processed and returned as a vector of `QueryModifierId`.
    pub(super) fn generate_modifiers_for_root_object(
        &mut self,
        root_object_id: ObjectDefinitionId,
    ) -> Vec<QueryModifierId> {
        let mut modifiers = Vec::new();
        for directive in self.schema.walk(root_object_id).directives() {
            match directive {
                TypeSystemDirective::Authenticated => {
                    modifiers.push(self.push_root_object_query_modifier(QueryModifierRule::Authenticated));
                }
                TypeSystemDirective::RequiresScopes(directive) => {
                    modifiers
                        .push(self.push_root_object_query_modifier(QueryModifierRule::RequiresScopes(directive.id())));
                }
                TypeSystemDirective::Authorized(directive) => {
                    modifiers.push(
                        self.push_root_object_query_modifier(QueryModifierRule::AuthorizedDefinition {
                            directive_id: directive.id(),
                            definition_id: DefinitionId::Object(root_object_id),
                        }),
                    );
                }
                _ => {}
            }
        }
        modifiers.sort_unstable();
        modifiers
    }

    /// Registers a field impacted by a specific response modifier.
    ///
    /// This function associates a field with a response modifier rule. It inserts a new entry
    /// if the rule is not already present, initializing it with the current number of response
    /// modifiers and an empty list of impacted fields. The specified field ID is then added to
    /// the list of fields impacted by the given response modifier.
    ///
    /// # Parameters
    ///
    /// - `rule`: The response modifier rule indicating the association for the field.
    /// - `field_id`: The identifier for the field impacted by the response modifier.
    fn register_field_impacted_by_response_modifier(&mut self, rule: ResponseModifierRule, field_id: FieldId) {
        let n = self.response_modifiers.len();

        self.response_modifiers
            .entry(rule)
            .or_insert((n.into(), Vec::new()))
            .1
            .push(field_id);
    }

    /// Registers a field impacted by a specific query modifier.
    ///
    /// This function associates a field with a query modifier rule. It inserts a new entry
    /// if the rule is not already present, initializing it with the current number of query
    /// modifiers and an empty list of impacted fields. The specified field ID is then added to
    /// the list of fields impacted by the given query modifier.
    ///
    /// # Parameters
    ///
    /// - `rule`: The query modifier rule indicating the association for the field.
    /// - `field_id`: The identifier for the field impacted by the query modifier.
    fn register_field_impacted_by_query_modifier(&mut self, rule: QueryModifierRule, field_id: FieldId) {
        let n = self.query_modifiers.len();

        self.query_modifiers
            .entry(rule)
            .or_insert((n.into(), Vec::new()))
            .1
            .push(field_id);
    }

    /// Pushes a query modifier for the root object based on the specified rule.
    ///
    /// This function registers a query modifier rule for the root object in the internal storage.
    ///
    /// # Parameters
    ///
    /// - `rule`: The query modifier rule to be registered for the root object.
    ///
    /// # Returns
    ///
    /// Returns the identifier for the query modifier associated with the specified rule.
    fn push_root_object_query_modifier(&mut self, rule: QueryModifierRule) -> QueryModifierId {
        let n = self.query_modifiers.len();
        self.query_modifiers.entry(rule).or_insert((n.into(), Vec::new())).0
    }
}

/// Finalizes the query modifiers by transforming the given `query_modifiers`
/// into a vector of `QueryModifier` and a vector of impacted field identifiers.
///
/// This function takes a hashmap mapping query modifier rules to their IDs and
/// the fields they impact, constructs `QueryModifier` instances for each rule,
/// and collects the impacted field IDs in the process.
///
/// # Parameters
///
/// - `query_modifiers`: A hashmap where keys are `QueryModifierRule` and values
///   are tuples containing the associated `QueryModifierId` and a vector of
///   impacted `FieldId`s.
///
/// # Returns
///
/// A tuple containing two elements:
///
/// - A vector of finalized `QueryModifier` instances.
/// - A vector of `FieldId`s that were impacted by the query modifiers.
pub(super) fn finalize_query_modifiers(
    query_modifiers: HashMap<QueryModifierRule, (QueryModifierId, Vec<FieldId>)>,
) -> (Vec<QueryModifier>, Vec<FieldId>) {
    finalize_modifiers(query_modifiers, |rule, ids_range| QueryModifier {
        rule,
        impacted_fields: IdRange::from(ids_range),
    })
}

/// Finalizes the response modifiers by transforming the given `response_modifiers`
/// into a vector of `ResponseModifier` and a vector of impacted field identifiers.
///
/// This function takes a hashmap mapping response modifier rules to their IDs and
/// the fields they impact, constructs `ResponseModifier` instances for each rule,
/// and collects the impacted field IDs in the process.
///
/// # Parameters
///
/// - `response_modifiers`: A hashmap where keys are `ResponseModifierRule` and values
///   are tuples containing the associated `ResponseModifierId` and a vector of
///   impacted `FieldId`s.
///
/// # Returns
///
/// A tuple containing two elements:
///
/// - A vector of finalized `ResponseModifier` instances.
/// - A vector of `FieldId`s that were impacted by the response modifiers.
pub(super) fn finalize_response_modifiers(
    response_modifiers: HashMap<ResponseModifierRule, (ResponseModifierId, Vec<FieldId>)>,
) -> (Vec<ResponseModifier>, Vec<FieldId>) {
    finalize_modifiers(response_modifiers, |rule, ids_range| ResponseModifier {
        rule,
        impacted_fields: IdRange::from(ids_range),
    })
}

/// Finalizes the modifiers by transforming the given `modifiers`
/// into a vector of `Modifier` instances and a vector of impacted field identifiers.
///
/// This function takes a hashmap mapping rules to their IDs and
/// the fields they impact, constructs `Modifier` instances for each rule,
/// and collects the impacted field IDs in the process.
///
/// # Parameters
///
/// - `modifiers`: A hashmap where keys are `Rule` and values
///   are tuples containing the associated `Id` and a vector of
///   impacted `FieldId`s.
///
/// # Returns
///
/// A tuple containing two elements:
///
/// - A vector of finalized `Modifier` instances.
/// - A vector of `FieldId`s that were impacted by the modifiers.
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
