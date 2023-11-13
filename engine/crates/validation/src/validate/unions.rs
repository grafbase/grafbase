use super::*;

pub(crate) fn validate_union_extension<'a>(
    type_name: &str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    ctx: &mut Context<'a>,
) {
    validate_directives(&type_definition.node.directives, ast::DirectiveLocation::Union, ctx);

    if !matches!(
        ctx.definition_names.get(type_name).map(|t| &t.node.kind),
        Some(ast::TypeKind::Union(_))
    ) {
        ctx.push_error(miette::miette!("Cannot extend unknown union {type_name}"));
    }
}

pub(crate) fn validate_union_members<'a>(
    type_name: &str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    union: &'a ast::UnionType,
    ctx: &mut Context<'a>,
) {
    validate_directives(&type_definition.node.directives, ast::DirectiveLocation::Union, ctx);

    ctx.with_union_members(type_name, &union.members, |ctx, members| {
        for member in members {
            let member_name = member.node.as_str();
            match ctx.definition_names.get(member_name) {
                Some(definition) => match definition.node.kind {
                    ast::TypeKind::Object(_) => (), // ok
                    _ => ctx.push_error(miette::miette!(
                        "Cannot add non-object type {member_name} as member of union type {type_name}"
                    )),
                },
                None => {
                    ctx.push_error(miette::miette!(
                        "Cannot add unknown type {member_name} as member of union type {type_name}"
                    ));
                }
            }
        }

        ctx.find_duplicates(members.iter().map(|name| name.node.as_str()), |ctx, first, _| {
            let name = &members[first].node;
            ctx.push_error(miette::miette!(
                r#"Union type "{type_name}" can only include type "{name}" once."#
            ));
        });
    });
}
