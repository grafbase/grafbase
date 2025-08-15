//! This is a separate module because we want to use only the public API of [Subgraphs] and avoid
//! mixing GraphQL parser logic and types with our internals.

mod context;
mod directive_definitions;
mod directives;
mod enums;
mod fields;
mod nested_key_fields;
mod schema_definitions;

use self::{directives::*, nested_key_fields::ingest_nested_key_fields, schema_definitions::*};
use crate::{
    Subgraphs,
    subgraphs::{self, DefinitionId, DefinitionKind, DirectiveSiteId, SubgraphId},
};
use cynic_parser::{ConstValue, type_system as ast};

/// _Service is a special type exposed by subgraphs. It should not be composed.
const SERVICE_TYPE_NAME: &str = "_Service";

/// _Entity is a special union type exposed by subgraphs. It should not be composed.
const ENTITY_UNION_NAME: &str = "_Entity";

struct Context<'a> {
    document: &'a ast::TypeSystemDocument,
    subgraph_id: SubgraphId,
    subgraphs: &'a mut Subgraphs,
    root_type_matcher: RootTypeMatcher<'a>,
}

pub(crate) fn ingest_subgraph(
    document: &ast::TypeSystemDocument,
    name: &str,
    url: Option<&str>,
    subgraphs: &mut Subgraphs,
) {
    let subgraph_id = subgraphs.push_subgraph(name, url);

    let mut ctx = Context {
        document,
        subgraph_id,
        subgraphs,
        root_type_matcher: Default::default(),
    };

    ingest_schema_definitions(&mut ctx);

    ingest_top_level_definitions(&mut ctx);
    ingest_definition_bodies(&mut ctx);
    ingest_nested_key_fields(&mut ctx);
}

fn ingest_top_level_definitions(ctx: &mut Context<'_>) {
    let subgraph_id = ctx.subgraph_id;

    for definition in ctx.document.definitions() {
        match definition {
            ast::Definition::Type(type_definition) | ast::Definition::TypeExtension(type_definition) => {
                let type_name = type_definition.name();

                let description = type_definition
                    .description()
                    .map(|description| ctx.subgraphs.strings.intern(description.to_cow()));

                let directive_site_id = ctx.subgraphs.new_directive_site();

                // Order matters: we ingest enum values in the following block. We have to ingest the enum's directives first, so directives are ingested in order of directive site id.
                directives::ingest_directives(ctx, directive_site_id, type_definition.directives(), |_| {
                    type_name.to_owned()
                });

                let definition_id = match type_definition {
                    ast::TypeDefinition::Object(_) if type_name == SERVICE_TYPE_NAME => continue,
                    ast::TypeDefinition::Union(_) if type_name == ENTITY_UNION_NAME => continue,

                    ast::TypeDefinition::Object(_) => {
                        let definition_id = ctx.subgraphs.push_definition(
                            subgraph_id,
                            type_name,
                            DefinitionKind::Object,
                            description,
                            directive_site_id,
                        );

                        match ctx.root_type_matcher.match_name(type_name) {
                            RootTypeMatch::Query => {
                                ctx.subgraphs.set_query_type(subgraph_id, definition_id);
                            }
                            RootTypeMatch::Mutation => {
                                ctx.subgraphs.set_mutation_type(subgraph_id, definition_id);
                            }
                            RootTypeMatch::Subscription => {
                                ctx.subgraphs.set_subscription_type(subgraph_id, definition_id);
                            }
                            RootTypeMatch::NotRootButHasDefaultRootName => {
                                ctx.subgraphs.push_ingestion_diagnostic(subgraph_id, format!("The {type_name} type has the default name for a root but is itself not a root. This is not valid in a federation context."));
                            }
                            RootTypeMatch::NotRoot => (),
                        }

                        definition_id
                    }
                    ast::TypeDefinition::Interface(_interface_type) => ctx.subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Interface,
                        description,
                        directive_site_id,
                    ),
                    ast::TypeDefinition::Union(_) => ctx.subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Union,
                        description,
                        directive_site_id,
                    ),
                    ast::TypeDefinition::InputObject(_) => ctx.subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::InputObject,
                        description,
                        directive_site_id,
                    ),

                    ast::TypeDefinition::Scalar(_) => ctx.subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Scalar,
                        description,
                        directive_site_id,
                    ),

                    ast::TypeDefinition::Enum(enum_type) => {
                        let definition_id = ctx.subgraphs.push_definition(
                            subgraph_id,
                            type_name,
                            DefinitionKind::Enum,
                            description,
                            directive_site_id,
                        );
                        enums::ingest_enum(ctx, definition_id, enum_type);

                        definition_id
                    }
                };

                directives::ingest_keys(definition_id, type_definition.directives(), ctx);
            }
            ast::Definition::Directive(directive_definition) => {
                let definition_name_id = ctx.subgraphs.strings.intern(directive_definition.name());

                if !ctx.subgraphs.is_composed_directive(ctx.subgraph_id, definition_name_id) {
                    continue;
                }
                directive_definitions::ingest_directive_definition(ctx, directive_definition, definition_name_id);
            }
            ast::Definition::Schema(_) | ast::Definition::SchemaExtension(_) => (),
        }
    }
}

fn ingest_definition_bodies(ctx: &mut Context<'_>) {
    let document = ctx.document;
    let subgraph_id = ctx.subgraph_id;

    let type_definitions = document.definitions().filter_map(|def| match def {
        ast::Definition::Type(ty) | ast::Definition::TypeExtension(ty) => Some(ty),
        _ => None,
    });

    for definition in type_definitions {
        match definition {
            ast::TypeDefinition::Union(_) if definition.name() == ENTITY_UNION_NAME => continue,
            ast::TypeDefinition::Union(union) => {
                let Some(union_id) = ctx.subgraphs.definition_by_name(definition.name(), subgraph_id) else {
                    ctx.subgraphs.push_ingestion_diagnostic(
                        subgraph_id,
                        format!(
                            "Union type `{}` is used but not defined in the subgraph.",
                            definition.name()
                        ),
                    );
                    continue;
                };

                for member in union.members() {
                    let Some(member_id) = ctx.subgraphs.definition_by_name(member.name(), subgraph_id) else {
                        ctx.subgraphs.push_ingestion_diagnostic(
                            subgraph_id,
                            format!(
                                "Union member `{}` is used but not defined in the subgraph.",
                                member.name()
                            ),
                        );
                        continue;
                    };
                    ctx.subgraphs.push_union_member(union_id, member_id);
                }
            }
            ast::TypeDefinition::InputObject(input_object) => {
                let Some(definition_id) = ctx.subgraphs.definition_by_name(definition.name(), subgraph_id) else {
                    ctx.subgraphs.push_ingestion_diagnostic(
                        subgraph_id,
                        format!(
                            "Input object type `{}` is used but not defined in the subgraph.",
                            definition.name()
                        ),
                    );
                    continue;
                };
                fields::ingest_input_fields(ctx, definition_id, input_object.fields());
            }
            ast::TypeDefinition::Interface(interface) => {
                let Some(definition_id) = ctx.subgraphs.definition_by_name(interface.name(), subgraph_id) else {
                    ctx.subgraphs.push_ingestion_diagnostic(
                        subgraph_id,
                        format!(
                            "Interface type `{}` is used but not defined in the subgraph.",
                            interface.name()
                        ),
                    );
                    continue;
                };

                for implemented_interface in interface.implements_interfaces() {
                    let Some(implemented_interface) =
                        ctx.subgraphs.definition_by_name(implemented_interface, subgraph_id)
                    else {
                        ctx.subgraphs.push_ingestion_diagnostic(
                            subgraph_id,
                            format!(
                                "Interface `{}` implementeds `{}`, but `{}` is not defined in the subgraph.",
                                interface.name(),
                                implemented_interface,
                                implemented_interface
                            ),
                        );
                        continue;
                    };

                    ctx.subgraphs.push_interface_impl(definition_id, implemented_interface);
                }

                let is_query_root_type = false; // interfaces can't be at the root

                fields::ingest_fields(ctx, definition_id, interface.fields(), is_query_root_type);
            }
            ast::TypeDefinition::Object(_) if definition.name() == SERVICE_TYPE_NAME => continue,
            ast::TypeDefinition::Object(object_type) => {
                let Some(definition_id) = ctx.subgraphs.definition_by_name(definition.name(), subgraph_id) else {
                    ctx.subgraphs.push_ingestion_diagnostic(
                        subgraph_id,
                        format!(
                            "Object type `{}` is used but not defined in the subgraph.",
                            definition.name()
                        ),
                    );
                    continue;
                };

                for implemented_interface in object_type.implements_interfaces() {
                    let Some(implemented_interface) =
                        ctx.subgraphs.definition_by_name(implemented_interface, subgraph_id)
                    else {
                        ctx.subgraphs.push_ingestion_diagnostic(
                            subgraph_id,
                            format!(
                                "Object type `{}` implements `{}`, but `{}` is not defined in the subgraph.",
                                object_type.name(),
                                implemented_interface,
                                implemented_interface
                            ),
                        );
                        continue;
                    };

                    ctx.subgraphs.push_interface_impl(definition_id, implemented_interface);
                }

                let is_query_root_type = ctx.root_type_matcher.is_query(definition.name());

                fields::ingest_fields(ctx, definition_id, object_type.fields(), is_query_root_type);
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
