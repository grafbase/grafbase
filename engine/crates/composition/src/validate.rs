use crate::subgraphs;

mod input_selection;
mod selection;
mod subgraph_names;

type ValidateContext<'a> = crate::ComposeContext<'a>;

/// Pre-composition validations happen here.
pub(crate) fn validate(ctx: &mut ValidateContext<'_>) {
    subgraph_names::validate_subgraph_names(ctx);
    validate_query_nonempty(ctx);
    validate_fields(ctx);
}

fn validate_query_nonempty(ctx: &mut ValidateContext<'_>) {
    if ctx
        .subgraphs
        .iter_subgraphs()
        .filter_map(|subgraph| subgraph.query_type())
        .all(|query_type| query_type.fields().next().is_none())
    {
        ctx.diagnostics
            .push_fatal(String::from("None of the subgraphs defines root query fields."));
    }
}

fn validate_fields(ctx: &mut ValidateContext<'_>) {
    for field in ctx.subgraphs.iter_all_fields() {
        selection::validate_selections(ctx, field);
        validate_override_labels(ctx, field);
        input_selection::validate_input_selections(ctx, field);
    }
}

fn validate_override_labels(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldWalker<'_>) {
    let Some(label) = field.directives().r#override().and_then(|directive| directive.label) else {
        return;
    };

    let Err(err) = ctx
        .subgraphs
        .walk(label)
        .as_str()
        .parse::<graphql_federated_graph::OverrideLabel>()
    else {
        return;
    };

    ctx.diagnostics.push_fatal(format!(
        "Invalid @override label argument on {ty}.{field}: {err}",
        ty = field.parent_definition().name().as_str(),
        field = field.name().as_str(),
    ));
}
