use crate::{federated_graph::OverrideLabel, subgraphs};

mod compose_directive;
mod composite_schemas;
mod extension_names;
mod selection;
mod subgraph_names;

type ValidateContext<'a> = crate::ComposeContext<'a>;

/// Pre-composition validations happen here.
pub(crate) fn validate(ctx: &mut ValidateContext<'_>) {
    composite_schemas::validate(ctx);
    extension_names::validate_extension_names(ctx);
    subgraph_names::validate_subgraph_names(ctx);
    validate_root_nonempty(ctx);
    validate_fields(ctx);
    compose_directive::validate_compose_directive(ctx);
    selection::validate_keys(ctx);
}

fn validate_root_nonempty(ctx: &mut ValidateContext<'_>) {
    if ctx.subgraphs.iter_subgraphs().all(|subgraph| {
        subgraph.query_type().is_none() && subgraph.mutation_type().is_none() && subgraph.subscription_type().is_none()
    }) {
        ctx.diagnostics.push_fatal(String::from(
            "None of the subgraphs define any root (Query, Mutation, Subscription) type. The federated graph cannot be empty.",
        ));
    }
}

fn validate_fields(ctx: &mut ValidateContext<'_>) {
    for field in ctx.subgraphs.iter_all_fields() {
        selection::validate_selections(ctx, field);
        validate_override_labels(ctx, field);
    }
}

fn validate_override_labels(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldWalker<'_>) {
    let Some(label) = field.directives().r#override().and_then(|directive| directive.label) else {
        return;
    };

    let Err(err) = ctx.subgraphs.walk(label).as_str().parse::<OverrideLabel>() else {
        return;
    };

    ctx.diagnostics.push_fatal(format!(
        "Invalid @override label argument on {ty}.{field}: {err}",
        ty = field.parent_definition().name().as_str(),
        field = field.name().as_str(),
    ));
}
