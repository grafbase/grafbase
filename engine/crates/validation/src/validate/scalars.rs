use super::*;

pub(crate) fn validate_scalar_extension<'a>(
    type_name: &str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    ctx: &mut Context<'a>,
) {
    validate_directives(&type_definition.node.directives, ast::DirectiveLocation::Scalar, ctx);

    if !is_builtin_scalar(type_name)
        && !matches!(
            ctx.definition_names.get(type_name).map(|t| &t.node.kind),
            Some(ast::TypeKind::Scalar)
        )
    {
        ctx.push_error(miette::miette!("Cannot extend unknown scalar {type_name}"));
    }
}

pub(crate) fn is_builtin_scalar(name: &str) -> bool {
    ["String", "ID", "Float", "Boolean", "Int"].contains(&name)
}
