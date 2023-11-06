use super::*;

/// First pass: resolve all definitions and their extensions.
pub(crate) fn validate_type_definition<'a>(typedef: &'a Positioned<ast::TypeDefinition>, ctx: &mut Context<'a>) {
    let type_name = typedef.node.name.node.as_str();

    if !typedef.node.extend && is_builtin_scalar(type_name) {
        let labels = vec![miette::LabeledSpan::new_with_span(
            None,
            (
                ctx.miette_pos(typedef.node.name.pos),
                miette::SourceOffset::from(typedef.node.name.node.len()),
            ),
        )];

        ctx.push_error(miette::miette! {
            labels = labels,
            "`{type_name}` is a reserved name.",
        });
    }

    if type_name.starts_with("__") {
        diagnostics::double_underscore_name(type_name, typedef.node.name.pos, ctx);
    }

    if typedef.node.extend {
        match &typedef.node.kind {
            ast::TypeKind::Object(obj) => {
                ctx.extended_fields.entry(type_name).or_default().push(&obj.fields);
                ctx.extended_interface_implementations
                    .entry(type_name)
                    .or_default()
                    .extend(obj.implements.iter());
            }
            ast::TypeKind::Interface(iface) => {
                ctx.extended_fields.entry(type_name).or_default().push(&iface.fields);
                ctx.extended_interface_implementations
                    .entry(type_name)
                    .or_default()
                    .extend(iface.implements.iter());
            }
            ast::TypeKind::Enum(enm) => {
                ctx.extended_enums.entry(type_name).or_default().push(&enm.values);
            }
            ast::TypeKind::Union(union) => {
                ctx.extended_unions.entry(type_name).or_default().push(&union.members);
            }
            _ => (),
        }
        return;
    }

    if let Some(existing_typedef) = ctx.definition_names.insert(type_name, typedef) {
        let labels = vec![
            miette::LabeledSpan::new_with_span(
                Some("Previous definition".to_owned()),
                miette::SourceSpan::new(
                    ctx.miette_pos(existing_typedef.node.name.pos),
                    existing_typedef.node.name.node.len().into(),
                ),
            ),
            miette::LabeledSpan::new_with_span(
                Some("Second definition".to_owned()),
                miette::SourceSpan::new(
                    ctx.miette_pos(typedef.node.name.pos),
                    typedef.node.name.node.len().into(),
                ),
            ),
        ];

        ctx.push_error(miette::miette! {
            labels = labels,
            r#"Duplicate definition. There can only be one typed name "{type_name}""#,
        });
    }
}

/// Second pass, after all the definitions have been traversed once.
pub(crate) fn validate_definitions_second_pass<'a>(ast: &'a ast::ServiceDocument, ctx: &mut Context<'a>) {
    for def in &ast.definitions {
        match def {
            ast::TypeSystemDefinition::Schema(_) | ast::TypeSystemDefinition::Directive(_) => (),
            ast::TypeSystemDefinition::Type(typedef) => {
                let type_name = typedef.node.name.node.as_str();
                let is_extension = typedef.node.extend;

                match &typedef.node.kind {
                    ast::TypeKind::Object(obj) if is_extension => {
                        validate_object_extension(type_name, typedef, obj, ctx);
                    }
                    ast::TypeKind::Object(obj) => {
                        validate_object(type_name, typedef, obj, ctx);
                    }

                    ast::TypeKind::Interface(iface) if is_extension => {
                        validate_interface_extension(type_name, typedef, iface, ctx);
                    }
                    ast::TypeKind::Interface(iface) => validate_interface(type_name, typedef, iface, ctx),

                    ast::TypeKind::InputObject(input_object) if is_extension => {
                        validate_input_object_extension(type_name, typedef, input_object, ctx);
                    }
                    ast::TypeKind::InputObject(input_object) => {
                        validate_input_object(type_name, typedef, input_object, ctx);
                    }

                    ast::TypeKind::Union(_) if is_extension => validate_union_extension(type_name, typedef, ctx),
                    ast::TypeKind::Union(union) => {
                        validate_union_members(type_name, typedef, union, ctx);
                    }

                    ast::TypeKind::Enum(_) if is_extension => validate_enum_extension(type_name, typedef, ctx),
                    ast::TypeKind::Enum(enm) => {
                        validate_enum_members(type_name, typedef, enm, ctx);
                    }

                    ast::TypeKind::Scalar if is_extension => validate_scalar_extension(type_name, typedef, ctx),
                    ast::TypeKind::Scalar => (),
                }
            }
        }
    }
}
