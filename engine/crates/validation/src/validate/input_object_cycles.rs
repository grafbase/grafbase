use crate::context::Context;
use async_graphql_parser::{types as ast, Positioned};

/// http://spec.graphql.org/draft/#sec-Input-Objects.Type-Validation
pub(crate) fn input_object_cycles<'a>(
    input_object_name: &'a str,
    input_object: &'a ast::InputObjectType,
    ctx: &mut Context<'a>,
) {
    if let Some(mut chain) = references_input_object_rec(input_object_name, &input_object.fields, ctx) {
        chain.reverse();
        ctx.push_error(miette::miette!(r#"Cannot reference Input Object {input_object_name} within itself through a series of non-null fields: "{}""#, chain.join(".")));
    }
}

fn references_input_object_rec<'a>(
    name: &'a str,
    fields: &'a [Positioned<ast::InputValueDefinition>],
    ctx: &mut Context<'a>,
) -> Option<Vec<&'a str>> {
    for field in fields {
        let field = &field.node;

        if field.ty.node.nullable || matches!(field.ty.node.base, ast::BaseType::List(_)) {
            continue;
        }

        let field_type_name = super::extract_type_name(&field.ty.node.base);
        if field_type_name == name {
            return Some(vec![field.name.node.as_str()]);
        }

        if let Some(ast::TypeKind::InputObject(input_object)) =
            ctx.definition_names.get(field_type_name).map(|ty| &ty.node.kind)
        {
            if let Some(mut chain) = references_input_object_rec(name, &input_object.fields, ctx) {
                chain.push(field.name.node.as_str());
                return Some(chain);
            }
        }
    }

    None
}
