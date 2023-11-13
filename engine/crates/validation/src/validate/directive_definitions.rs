use super::*;

pub(crate) fn validate_directive_definition<'a>(
    definition: &'a Positioned<ast::DirectiveDefinition>,
    ctx: &mut Context<'a>,
) {
    let directive_name = definition.node.name.node.as_str();
    if definition.node.name.node.starts_with("__") {
        ctx.push_error(miette::miette! {
            r#"Directive names must not start with "__""#,
        });
    }

    for arg in &definition.node.arguments {
        let type_name = extract_type_name(&arg.node.ty.node.base);
        let location = || format!("@{directive_name}({}:)", arg.node.name.node);
        match validate_input_type(type_name, arg.node.ty.pos, ctx) {
            ValidateInputTypeResult::Ok => (),
            ValidateInputTypeResult::UnknownType => diagnostics::unknown_type(type_name, &location(), ctx),
            ValidateInputTypeResult::NotAnInputType => {
                diagnostics::output_type_in_input_position(type_name, &location(), ctx);
            }
        }
    }

    ctx.directive_names
        .insert(definition.node.name.node.as_str(), definition);
}

pub(crate) fn validate_directives<'a>(
    directives: &'a [Positioned<ast::ConstDirective>],
    location: ast::DirectiveLocation,
    ctx: &mut Context<'a>,
) {
    let names = directives.iter().map(|d| d.node.name.node.as_str());
    ctx.find_duplicates(names, |ctx, first, _| {
        let directive_name = directives[first].node.name.node.as_str();
        if ctx
            .directive_names
            .get(directive_name)
            .map(|directive| directive.node.is_repeatable)
            .unwrap_or(true)
        {
            return;
        }

        ctx.push_error(miette::miette!("{directive_name} is not repeatable"));
    });

    for directive in directives {
        let directive_name = directive.node.name.node.as_str();
        if let Some(definition) = ctx.directive_names.get(directive_name) {
            if !definition.node.locations.iter().any(|loc| loc.node == location) {
                ctx.push_error(miette::miette!(
                    "Directive @{directive_name} used at an invalid location: {:?}",
                    location
                ));
            }
        }
    }
}
