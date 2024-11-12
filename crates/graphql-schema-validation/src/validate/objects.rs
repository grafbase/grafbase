use super::*;

pub(crate) fn validate_object<'a>(
    parent_type_name: &'a str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    obj: &'a ast::ObjectType,
    ctx: &mut Context<'a>,
) {
    validate_directives(&type_definition.node.directives, ast::DirectiveLocation::Object, ctx);

    ctx.with_implements(parent_type_name, &obj.implements, |ctx, implements| {
        interface_implementers::validate_implements_list(parent_type_name, implements, &obj.fields, ctx);
    });

    ctx.with_fields(parent_type_name, &obj.fields, |ctx, fields| {
        if fields.is_empty() {
            diagnostics::empty_object(parent_type_name, ctx)
        }

        for field in fields {
            object_field::validate_object_field(parent_type_name, field, ctx);
            let type_name = extract_type_name(&field.node.ty.node.base);
            let field_name = &field.node.name.node;
            let location = || format!("{parent_type_name}.{field_name}");
            match output_types::validate_output_type(type_name, field.node.ty.pos, ctx) {
                output_types::ValidateOutputTypeResult::Ok => (),
                output_types::ValidateOutputTypeResult::UnknownType => {
                    diagnostics::unknown_type(type_name, &location(), ctx);
                }
                output_types::ValidateOutputTypeResult::InputObject => {
                    diagnostics::input_object_in_output_position(type_name, &location(), ctx);
                }
            }
        }

        let field_names = fields.iter().map(|f| f.node.name.node.as_str());
        ctx.find_duplicates(field_names, |ctx, _, idx| {
            let field_name = fields[idx].node.name.node.as_str();
            ctx.push_error(miette::miette!(
                "Duplicate field {field_name} already exists on {parent_type_name}"
            ));
        });
    });
}

pub(crate) fn validate_object_extension<'a>(
    type_name: &'a str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    _obj: &'a ast::ObjectType,
    ctx: &mut Context<'a>,
) {
    validate_directives(&type_definition.node.directives, ast::DirectiveLocation::Object, ctx);

    if ctx.options.contains(Options::FORBID_EXTENDING_UNKNOWN_TYPES)
        && !matches!(
            ctx.definition_names.get(type_name).map(|t| &t.node.kind),
            Some(ast::TypeKind::Object(_))
        )
    {
        ctx.push_error(miette::miette!("Cannot extend unknown object {type_name}"));
    }
}
