//! This is a separate module because we want to use only the public API of [Subgraphs] and avoid
//! mixing GraphQL parser logic and types with our internals.

mod directive_definitions;
mod directives;
mod enums;
mod fields;
mod nested_key_fields;
mod schema_definitions;

use self::{directives::*, nested_key_fields::ingest_nested_key_fields, schema_definitions::*};
use crate::{
    subgraphs::{self, DefinitionId, DefinitionKind, DirectiveSiteId, SubgraphId},
    Subgraphs,
};
use cynic_parser::{type_system as ast, ConstValue};

/// _Service is a special type exposed by subgraphs. It should not be composed.
const SERVICE_TYPE_NAME: &str = "_Service";

/// _Entity is a special union type exposed by subgraphs. It should not be composed.
const ENTITY_UNION_NAME: &str = "_Entity";

pub(crate) fn ingest_subgraph(document: &ast::TypeSystemDocument, name: &str, url: &str, subgraphs: &mut Subgraphs) {
    let subgraph_id = subgraphs.push_subgraph(name, url);

    let root_type_matcher = ingest_schema_definition(document);

    let directive_matcher = ingest_directive_definitions(document, |error| {
        subgraphs.push_ingestion_diagnostic(subgraph_id, error);
    });

    ingest_top_level_definitions(subgraph_id, document, subgraphs, &directive_matcher, &root_type_matcher);
    ingest_definition_bodies(subgraph_id, document, subgraphs, &directive_matcher, &root_type_matcher);
    ingest_nested_key_fields(subgraph_id, subgraphs);

    for name in directive_matcher.iter_composed_directives() {
        subgraphs.insert_composed_directive(name)
    }
}

fn ingest_top_level_definitions(
    subgraph_id: SubgraphId,
    document: &ast::TypeSystemDocument,
    subgraphs: &mut Subgraphs,
    directive_matcher: &DirectiveMatcher<'_>,
    root_type_matcher: &RootTypeMatcher<'_>,
) {
    for definition in document.definitions() {
        match definition {
            ast::Definition::Type(type_definition) | ast::Definition::TypeExtension(type_definition) => {
                let type_name = type_definition.name();

                let description = type_definition
                    .description()
                    .map(|description| subgraphs.strings.intern(description.to_cow()));

                let directives = subgraphs.new_directive_site();

                let definition_id = match type_definition {
                    ast::TypeDefinition::Object(_) if type_name == SERVICE_TYPE_NAME => continue,
                    ast::TypeDefinition::Union(_) if type_name == ENTITY_UNION_NAME => continue,

                    ast::TypeDefinition::Object(_) => {
                        let definition_id = subgraphs.push_definition(
                            subgraph_id,
                            type_name,
                            DefinitionKind::Object,
                            description,
                            directives,
                        );

                        match root_type_matcher.match_name(type_name) {
                            RootTypeMatch::Query => {
                                subgraphs.set_query_type(subgraph_id, definition_id);
                            }
                            RootTypeMatch::Mutation => {
                                subgraphs.set_mutation_type(subgraph_id, definition_id);
                            }
                            RootTypeMatch::Subscription => {
                                subgraphs.set_subscription_type(subgraph_id, definition_id);
                            }
                            RootTypeMatch::NotRootButHasDefaultRootName => {
                                subgraphs.push_ingestion_diagnostic(subgraph_id, format!("The {type_name} type has the default name for a root but is itself not a root. This is not valid in a federation context."));
                            }
                            RootTypeMatch::NotRoot => (),
                        }

                        definition_id
                    }
                    ast::TypeDefinition::Interface(_interface_type) => subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Interface,
                        description,
                        directives,
                    ),
                    ast::TypeDefinition::Union(_) => subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Union,
                        description,
                        directives,
                    ),
                    ast::TypeDefinition::InputObject(_) => subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::InputObject,
                        description,
                        directives,
                    ),

                    ast::TypeDefinition::Scalar(_) => subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Scalar,
                        description,
                        directives,
                    ),

                    ast::TypeDefinition::Enum(enum_type) => {
                        let definition_id = subgraphs.push_definition(
                            subgraph_id,
                            type_name,
                            DefinitionKind::Enum,
                            description,
                            directives,
                        );
                        enums::ingest_enum(definition_id, enum_type, subgraphs, directive_matcher, subgraph_id);
                        definition_id
                    }
                };

                directives::ingest_directives(
                    directives,
                    type_definition.directives(),
                    subgraphs,
                    directive_matcher,
                    subgraph_id,
                    |_| type_name.to_owned(),
                );

                directives::ingest_keys(
                    definition_id,
                    type_definition.directives(),
                    subgraphs,
                    directive_matcher,
                );
            }
            ast::Definition::Directive(directive_definition) => {
                directive_definitions::ingest_directive_definition(directive_definition, subgraph_id, subgraphs);
            }
            ast::Definition::Schema(_) | ast::Definition::SchemaExtension(_) => (),
        }
    }
}

fn ingest_definition_bodies(
    subgraph_id: SubgraphId,
    document: &ast::TypeSystemDocument,
    subgraphs: &mut Subgraphs,
    federation_directives_matcher: &DirectiveMatcher<'_>,
    root_type_matcher: &RootTypeMatcher<'_>,
) {
    let type_definitions = document.definitions().filter_map(|def| match def {
        ast::Definition::Type(ty) | ast::Definition::TypeExtension(ty) => Some(ty),
        _ => None,
    });

    for definition in type_definitions {
        match definition {
            ast::TypeDefinition::Union(_) if definition.name() == ENTITY_UNION_NAME => continue,
            ast::TypeDefinition::Union(union) => {
                let union_id = subgraphs.definition_by_name(definition.name(), subgraph_id);

                for member in union.members() {
                    let member_id = subgraphs.definition_by_name(member.name(), subgraph_id);
                    subgraphs.push_union_member(union_id, member_id);
                }
            }
            ast::TypeDefinition::InputObject(input_object) => {
                let definition_id = subgraphs.definition_by_name(definition.name(), subgraph_id);
                fields::ingest_input_fields(
                    definition_id,
                    input_object.fields(),
                    federation_directives_matcher,
                    subgraphs,
                    subgraph_id,
                );
            }
            ast::TypeDefinition::Interface(interface) => {
                let definition_id = subgraphs.definition_by_name(interface.name(), subgraph_id);

                for implemented_interface in interface.implements_interfaces() {
                    let implemented_interface = subgraphs.definition_by_name(implemented_interface, subgraph_id);

                    subgraphs.push_interface_impl(definition_id, implemented_interface);
                }

                let is_query_root_type = false; // interfaces can't be at the root

                fields::ingest_fields(
                    definition_id,
                    interface.fields(),
                    federation_directives_matcher,
                    is_query_root_type,
                    subgraph_id,
                    subgraphs,
                );
            }
            ast::TypeDefinition::Object(_) if definition.name() == SERVICE_TYPE_NAME => continue,
            ast::TypeDefinition::Object(object_type) => {
                let definition_id = subgraphs.definition_by_name(definition.name(), subgraph_id);

                for implemented_interface in object_type.implements_interfaces() {
                    let implemented_interface = subgraphs.definition_by_name(implemented_interface, subgraph_id);

                    subgraphs.push_interface_impl(definition_id, implemented_interface);
                }

                let is_query_root_type = root_type_matcher.is_query(definition.name());

                fields::ingest_fields(
                    definition_id,
                    object_type.fields(),
                    federation_directives_matcher,
                    is_query_root_type,
                    subgraph_id,
                    subgraphs,
                );
            }
            _ => (),
        }
    }
}

pub(super) fn ast_value_to_subgraph_value(value: ConstValue<'_>, subgraphs: &mut Subgraphs) -> subgraphs::Value {
    match &value {
        ConstValue::Null(_) => subgraphs::Value::Null,
        ConstValue::Int(n) => subgraphs::Value::Int(n.as_i64()),
        ConstValue::Float(n) => subgraphs::Value::Float(n.as_f64()),
        ConstValue::String(s) => subgraphs::Value::String(subgraphs.strings.intern(s.as_str())),
        ConstValue::Boolean(b) => subgraphs::Value::Boolean(b.value()),
        ConstValue::Enum(e) => subgraphs::Value::Enum(subgraphs.strings.intern(e.name())),
        ConstValue::List(l) => {
            subgraphs::Value::List(l.items().map(|v| ast_value_to_subgraph_value(v, subgraphs)).collect())
        }
        ConstValue::Object(o) => subgraphs::Value::Object(
            o.fields()
                .map(|field| {
                    (
                        subgraphs.strings.intern(field.name()),
                        ast_value_to_subgraph_value(field.value(), subgraphs),
                    )
                })
                .collect(),
        ),
    }
}
