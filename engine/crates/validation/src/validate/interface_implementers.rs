use std::collections::HashMap;

use crate::context::Context;
use async_graphql_parser::{types as ast, Positioned};
use async_graphql_value::Name;

pub(crate) fn validate_implements_list<'a>(
    parent_name: &'a str,
    parent_implements: &[&'a Positioned<Name>],
    parent_fields: &'a [Positioned<ast::FieldDefinition>],
    ctx: &mut Context<'a>,
) {
    let implement_names = parent_implements.iter().map(|i| i.node.as_str());
    ctx.find_duplicates(implement_names, |ctx, idx, _| {
        let iface_name = parent_implements[idx].node.as_str();
        ctx.push_error(miette::miette!(
            r#"Type "{parent_name}" can only implement "{iface_name}" once."#
        ));
    });

    for iface in parent_implements {
        let iface_name = iface.node.as_str();
        match ctx.definition_names.get(iface_name).copied() {
            Some(ty) => match &ty.node.kind {
                ast::TypeKind::Interface(iface) => {
                    validate_implements_interface_transitively(
                        parent_name,
                        parent_implements,
                        &ty.node.name.node,
                        iface,
                        ctx,
                    );

                    validate_fields_implement_interface(parent_name, parent_fields, &ty.node.name.node, iface, ctx);
                }
                _ => ctx.push_error(miette::miette!(
                    r#""{parent_name}" cannot implement non-interface type "{}"."#,
                    ty.node.name.node.as_str()
                )),
            },
            None => ctx.push_error(miette::miette!(
                r#""{parent_name} cannot implement unknown type "{iface_name}"."#
            )),
        }
    }
}

fn validate_fields_implement_interface<'a>(
    implementer_name: &str,
    implementer_fields: &'a [Positioned<ast::FieldDefinition>],
    iface_name: &'a str,
    iface: &'a ast::InterfaceType,
    ctx: &mut Context<'a>,
) {
    let mut errs = Vec::new();

    ctx.with_fields(implementer_name, implementer_fields, |ctx, candidate_superset| {
        for field in &iface.fields {
            let candidate_field = candidate_superset
                .iter()
                .find(|candidate| candidate.node.name.node == field.node.name.node);

            match candidate_field {
                Some(candidate_field) => {
                    validate_field_type_implements_interface_field(
                        iface_name,
                        implementer_name,
                        candidate_field,
                        field,
                        ctx,
                    );
                    validate_field_arguments_implement_interface_field(
                        iface_name,
                        implementer_name,
                        candidate_field,
                        field,
                        ctx,
                    );
                }
                None => errs.push(miette::miette!(
                    "Missing `{}` field in `{implementer_name}` (required by the `{iface_name}` interface)",
                    field.node.name.node
                )),
            }
        }
    });

    for err in errs {
        ctx.push_error(err);
    }
}

fn validate_field_arguments_implement_interface_field(
    iface_name: &str,
    implementer_name: &str,
    candidate_field: &Positioned<ast::FieldDefinition>,
    iface_field: &Positioned<ast::FieldDefinition>,
    ctx: &mut Context<'_>,
) {
    let field_name = candidate_field.node.name.node.as_str();
    let candidate_args = &candidate_field.node.arguments;
    let iface_args = &iface_field.node.arguments;
    let mut args: HashMap<&str, (Option<usize>, Option<usize>)> =
        HashMap::with_capacity(candidate_args.len().max(iface_args.len()));

    for (idx, arg) in candidate_args.iter().enumerate() {
        args.insert(arg.node.name.node.as_str(), (Some(idx), None));
    }

    for (idx, arg) in iface_args.iter().enumerate() {
        args.entry(arg.node.name.node.as_str()).or_default().1 = Some(idx);
    }

    for (candidate, iface) in args.into_values() {
        let candidate = candidate.map(|idx| &candidate_args[idx]);
        let iface = iface.map(|idx| &iface_args[idx]);
        match (candidate, iface) {
            (Some(candidate), Some(iface)) => {
                if candidate.node.ty != iface.node.ty {
                    let arg_name = candidate.node.name.node.as_str();
                    let expected_type = iface.node.ty.to_string();
                    let found_type = candidate.node.ty.to_string();
                    let iface_arg_location = format!("{iface_name}.{field_name}({arg_name}:)");
                    let candidate_arg_location = format!("{implementer_name}.{field_name}({arg_name}:)");

                    ctx.push_error(miette::miette!("Interface field argument {iface_arg_location} expects type {expected_type} but {candidate_arg_location} is type {found_type}"));
                }
            }
            (Some(candidate), None) if candidate.node.ty.node.nullable => (), // ok
            (Some(candidate), None) => {
                let arg_name = candidate.node.name.node.as_str();
                let iface_field = format!("{iface_name}.{field_name}");
                let candidate_field = format!("{implementer_name}.{field_name}");
                ctx.push_error(miette::miette!("Field {candidate_field} includes required argument {arg_name} that is missing from the Interface field {iface_field}"));
            }
            (None, Some(arg)) => {
                let arg_name = arg.node.name.node.as_str();
                ctx.push_error(miette::miette!("Interface field argument {iface_name}.{field_name}({arg_name}:) expected but {implementer_name}.{field_name} does not provide it."));
            }
            (None, None) => unreachable!(),
        }
    }
}

// http://spec.graphql.org/draft/#IsValidImplementationFieldType()
// http://spec.graphql.org/draft/#IsSubType()
fn validate_field_type_implements_interface_field(
    interface_name: &str,
    implementer_name: &str,
    candidate_field: &Positioned<ast::FieldDefinition>,
    interface_field: &Positioned<ast::FieldDefinition>,
    ctx: &mut Context<'_>,
) {
    let candidate_field_name = &candidate_field.node.name.node;
    let candidate_type_name = super::extract_type_name(&candidate_field.node.ty.node.base);
    let iface_field_type_name = super::extract_type_name(&interface_field.node.ty.node.base);

    if validate_implementer_wrapper_types(&candidate_field.node.ty.node, &interface_field.node.ty.node)
        && validate_implementer_inner_type(candidate_type_name, iface_field_type_name, ctx)
    {
        return;
    }

    let candidate_field_type = candidate_field.node.ty.to_string();
    let iface_field_type = interface_field.node.ty.to_string();

    ctx.push_error(miette::miette!(
            "Interface field {interface_name}.{candidate_field_name} expects type {iface_field_type} but {implementer_name}.{candidate_field_name} of type {candidate_field_type} is not a proper subtype.`"
        ));
}

fn validate_implementer_inner_type(
    candidate_type_name: &str,
    iface_field_type_name: &str,
    ctx: &mut Context<'_>,
) -> bool {
    if candidate_type_name == iface_field_type_name {
        return true;
    }

    // Check if the candidate is a refinement of the interface.
    match ctx
        .definition_names
        .get(iface_field_type_name)
        .map(|def| &def.node.kind)
    {
        Some(ast::TypeKind::Union(union)) => {
            if union.members.iter().any(|member| member.node == candidate_type_name) {
                return true;
            }
        }
        Some(ast::TypeKind::Interface(_)) => {
            match ctx.definition_names.get(candidate_type_name).map(|def| &def.node.kind) {
                Some(ast::TypeKind::Object(obj)) => {
                    if obj
                        .implements
                        .iter()
                        .any(|implemented| implemented.node == iface_field_type_name)
                    {
                        return true;
                    }
                }
                Some(ast::TypeKind::Interface(field_iface)) => {
                    if field_iface
                        .implements
                        .iter()
                        .any(|implemented| implemented.node == iface_field_type_name)
                    {
                        return true;
                    }
                }
                _ => (),
            }
        }
        _ => (),
    }

    false
}

fn validate_implementer_wrapper_types(candidate: &ast::Type, iface: &ast::Type) -> bool {
    if !iface.nullable && candidate.nullable {
        return false;
    }

    match (&candidate.base, &iface.base) {
        (ast::BaseType::Named(_), ast::BaseType::Named(_)) => true,
        (ast::BaseType::Named(_), ast::BaseType::List(_)) | (ast::BaseType::List(_), ast::BaseType::Named(_)) => false,
        (ast::BaseType::List(inner_candidate), ast::BaseType::List(inner_iface)) => {
            validate_implementer_wrapper_types(inner_candidate, inner_iface)
        }
    }
}

fn validate_implements_interface_transitively<'a>(
    parent_name: &'a str,
    parent_implements: &[&'a Positioned<Name>],
    iface_name: &'a str,
    iface: &'a ast::InterfaceType,
    ctx: &mut Context<'a>,
) {
    for iface_implements in &iface.implements {
        if !parent_implements
            .iter()
            .any(|obj_implements| obj_implements.node == iface_implements.node)
        {
            let implements = iface_implements.node.as_str();
            ctx.push_error(miette::miette!(
                "`Type {parent_name}` must implement `{implements}` because it is implemented by `{iface_name}`"
            ));
        }
    }
}
