mod arguments;
mod directive_definitions;
mod enums;
mod input_object_cycles;
mod input_objects;
mod input_types;
mod interface_implementers;
mod interfaces;
mod object_field;
mod objects;
mod output_types;
mod scalars;
mod schema_definition;
mod type_definition;
mod unions;

use self::{
    directive_definitions::*, enums::*, input_objects::*, input_types::*, interfaces::*, objects::*, scalars::*,
    schema_definition::*, type_definition::*, unions::*,
};
use crate::{diagnostics, Context, Options, SchemaDefinition};
use async_graphql_parser::{types as ast, Positioned};

pub(crate) fn validate<'a>(parsed_ast: &'a ast::ServiceDocument, ctx: &mut Context<'a>) {
    let mut schema_definitions = Vec::new();

    for definition in &parsed_ast.definitions {
        match definition {
            ast::TypeSystemDefinition::Schema(def) => {
                schema_definitions.push(SchemaDefinition {
                    pos: def.pos,
                    directives: &def.node.directives,
                    query: def.node.query.as_ref().map(|node| node.node.as_str()),
                    mutation: def.node.mutation.as_ref().map(|node| node.node.as_str()),
                    subscription: def.node.subscription.as_ref().map(|node| node.node.as_str()),
                    is_extension: def.node.extend,
                });
            }
            ast::TypeSystemDefinition::Type(typedef) => validate_type_definition(typedef, ctx),
            ast::TypeSystemDefinition::Directive(def) => validate_directive_definition(def, ctx),
        }
    }

    validate_schema_definitions(&schema_definitions, ctx);

    if schema_definitions.is_empty() {
        validate_root_types(ctx);
    }

    validate_definitions_second_pass(parsed_ast, ctx);
}

fn extract_type_name(base: &ast::BaseType) -> &str {
    match base {
        ast::BaseType::Named(name) => name.as_str(),
        ast::BaseType::List(inner) => extract_type_name(&inner.base),
    }
}
