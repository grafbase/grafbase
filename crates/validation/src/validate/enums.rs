use super::*;

pub(crate) fn validate_enum_members<'a>(
    type_name: &str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    enm: &'a ast::EnumType,
    ctx: &mut Context<'a>,
) {
    validate_directives(&type_definition.node.directives, ast::DirectiveLocation::Enum, ctx);

    ctx.with_enum_values(type_name, &enm.values, |ctx, values| {
        let value_names = values.iter().map(|v| v.node.value.node.as_str());
        ctx.find_duplicates(value_names, |ctx, idx, _| {
            let value_name = enm.values[idx].node.value.node.as_str();
            ctx.push_error(miette::miette!(r#"Duplicate enum value "{type_name}.{value_name}""#));
        });

        for value in values {
            validate_directives(
                &value.node.directives,
                ast::DirectiveLocation::InputFieldDefinition,
                ctx,
            );
        }
    });
}

pub(crate) fn validate_enum_extension<'a>(
    type_name: &str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    ctx: &mut Context<'a>,
) {
    validate_directives(&type_definition.node.directives, ast::DirectiveLocation::Enum, ctx);

    if ctx.options.contains(Options::FORBID_EXTENDING_UNKNOWN_TYPES)
        && !matches!(
            ctx.definition_names.get(type_name).map(|t| &t.node.kind),
            Some(ast::TypeKind::Enum(_))
        )
    {
        ctx.push_error(miette::miette!("Cannot extend unknown enum {type_name}"));
    }
}
