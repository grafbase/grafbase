use super::*;

pub(crate) fn validate_input_object<'a>(
    parent_type_name: &'a str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    input_object: &'a ast::InputObjectType,
    ctx: &mut Context<'a>,
) {
    validate_directives(
        &type_definition.node.directives,
        ast::DirectiveLocation::InputObject,
        ctx,
    );

    for field in &input_object.fields {
        validate_directives(
            &field.node.directives,
            ast::DirectiveLocation::InputFieldDefinition,
            ctx,
        );
        let field_name = &field.node.name.node;
        let type_name = extract_type_name(&field.node.ty.node.base);
        let location = || format!("{parent_type_name}.{field_name}");
        match input_types::validate_input_type(type_name, field.node.ty.pos, ctx) {
            ValidateInputTypeResult::Ok => (),
            ValidateInputTypeResult::UnknownType => diagnostics::unknown_type(type_name, &location(), ctx),
            ValidateInputTypeResult::NotAnInputType => {
                diagnostics::output_type_in_input_position(type_name, &location(), ctx);
            }
        }
    }

    input_object_cycles::input_object_cycles(parent_type_name, input_object, ctx);
}

pub(crate) fn validate_input_object_extension<'a>(
    type_name: &'a str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    _obj: &'a ast::InputObjectType,
    ctx: &mut Context<'a>,
) {
    validate_directives(
        &type_definition.node.directives,
        ast::DirectiveLocation::InputObject,
        ctx,
    );

    if ctx.options.contains(Options::FORBID_EXTENDING_UNKNOWN_TYPES)
        && !matches!(
            ctx.definition_names.get(type_name).map(|t| &t.node.kind),
            Some(ast::TypeKind::InputObject(_))
        )
    {
        ctx.push_error(miette::miette!("Cannot extend unknown object {type_name}"));
    }
}
