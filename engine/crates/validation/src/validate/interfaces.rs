use super::*;

pub(crate) fn validate_interface<'a>(
    parent_type_name: &'a str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    iface: &'a ast::InterfaceType,
    ctx: &mut Context<'a>,
) {
    validate_directives(&type_definition.node.directives, ctx);
    ctx.with_implements(parent_type_name, &iface.implements, |ctx, implements| {
        interface_implementers::validate_implements_list(parent_type_name, implements, &iface.fields, ctx);
    });

    for field in &iface.fields {
        object_field::validate_object_field(parent_type_name, field, ctx);
        let field_name = &field.node.name.node;
        let type_name = extract_type_name(&field.node.ty.node.base);
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
}

pub(crate) fn validate_interface_extension<'a>(
    type_name: &'a str,
    type_definition: &'a Positioned<ast::TypeDefinition>,
    _iface: &'a ast::InterfaceType,
    ctx: &mut Context<'a>,
) {
    validate_directives(&type_definition.node.directives, ctx);
    if !matches!(
        ctx.definition_names.get(type_name).map(|t| &t.node.kind),
        Some(ast::TypeKind::Interface(_))
    ) {
        ctx.push_error(miette::miette!("Cannot extend unknown interface {type_name}"));
    }
}
