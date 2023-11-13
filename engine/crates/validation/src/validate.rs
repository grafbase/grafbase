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
use crate::{diagnostics, Context, Options};
use async_graphql_parser::{types as ast, Positioned};

pub(crate) fn validate<'a>(parsed_ast: &'a ast::ServiceDocument, ctx: &mut Context<'a>) {
    for definition in &parsed_ast.definitions {
        match definition {
            ast::TypeSystemDefinition::Schema(def) => validate_schema_definition(def, ctx),
            ast::TypeSystemDefinition::Type(typedef) => validate_type_definition(typedef, ctx),
            ast::TypeSystemDefinition::Directive(def) => validate_directive_definition(def, ctx),
        }
    }

    validate_schema_definition_references(ctx);
    validate_root_types(ctx);
    validate_definitions_second_pass(parsed_ast, ctx);
}

fn extract_type_name(base: &ast::BaseType) -> &str {
    match base {
        ast::BaseType::Named(name) => name.as_str(),
        ast::BaseType::List(inner) => extract_type_name(&inner.base),
    }
}
