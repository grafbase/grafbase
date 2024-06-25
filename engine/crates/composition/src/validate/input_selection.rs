use super::*;

pub(super) fn validate_input_selections(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldWalker<'_>) {
    for (selection, directive_name) in field
        .directives()
        .authorized()
        .into_iter()
        .flat_map(|authorized| authorized.arguments.as_ref().into_iter().flatten())
        .map(|input_selection| (input_selection, "authorized"))
    {
        let directive_path = || {
            format!(
                "{}.{}",
                field.parent_definition().name().as_str(),
                field.name().as_str()
            )
        };
        validate_input_selection_on_arguments(ctx, selection, field, &directive_path, directive_name);
    }
}

fn validate_input_selection_on_arguments(
    ctx: &mut ValidateContext<'_>,
    selection: &subgraphs::Selection,
    field: subgraphs::FieldWalker<'_>,
    directive_path: &dyn Fn() -> String,
    directive_name: &str,
) {
    if field.argument_by_name(selection.field).is_none() {
        ctx.diagnostics.push_fatal(format!(
            "Error in @{directive_name}: the {field_in_selection} argument does not exist on {directive_path}. Did you use the `arguments` argument instead of `fields`?",
            field_in_selection = ctx.subgraphs.walk(selection.field).as_str(),
            directive_path = directive_path(),
        ));
    };
}
