use super::{input_types::ValidateInputTypeResult, *};

pub(crate) fn validate_arguments<'a>(
    parent_field: (&str, &str),
    args: &'a [Positioned<ast::InputValueDefinition>],
    ctx: &mut Context<'a>,
) {
    {
        let arg_names = args.iter().map(|arg| arg.node.name.node.as_str());
        ctx.find_duplicates(arg_names, |ctx, _, duplicate| {
            let name = args[duplicate].node.name.node.as_str();
            ctx.push_error(miette::miette!(
                "Duplicate argument {name} in {}.{}",
                parent_field.0,
                parent_field.1
            ));
        });
    }

    for arg in args {
        let arg_name = &arg.node.name.node;

        if arg_name.starts_with("__") {
            let label = vec![miette::LabeledSpan::new_with_span(
                Some("here".to_owned()),
                miette::SourceSpan::new(ctx.miette_pos(arg.node.name.pos), arg.node.name.node.len().into()),
            )];
            ctx.push_error(miette::miette!(labels = label, "Argument names can't start with __"));
        }

        let type_name = super::extract_type_name(&arg.node.ty.node.base);
        let location = || format!("{}.{}({arg_name}:)", parent_field.0, parent_field.1);
        match super::input_types::validate_input_type(type_name, arg.node.ty.pos, ctx) {
            ValidateInputTypeResult::Ok => (),
            ValidateInputTypeResult::UnknownType => diagnostics::unknown_type(type_name, &location(), ctx),
            ValidateInputTypeResult::NotAnInputType => {
                diagnostics::output_type_in_input_position(type_name, &location(), ctx);
            }
        }

        if ctx.options.contains(crate::Options::DRAFT_VALIDATIONS) {
            let is_non_null_without_default = !arg.node.ty.node.nullable && arg.node.default_value.is_none();
            if is_non_null_without_default && arg.node.directives.iter().any(|dir| dir.node.name.node == "deprecated") {
                ctx.push_error(miette::miette!(
                    "Required argument {}.{}({}:) cannot be deprecated.",
                    parent_field.0,
                    parent_field.1,
                    arg.node.name.node,
                ));
            }
        }

        validate_directives(&arg.node.directives, ast::DirectiveLocation::ArgumentDefinition, ctx);
    }
}
