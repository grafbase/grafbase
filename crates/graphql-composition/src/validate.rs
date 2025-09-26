use crate::{federated_graph::OverrideLabel, subgraphs};

pub(crate) mod composite_schemas;
mod directives;
mod extension_names;
mod selection;
mod subgraph_names;

type ValidateContext<'a> = crate::ComposeContext<'a>;

/// Pre-composition validations happen here.
pub(crate) fn validate_pre_merge(ctx: &mut ValidateContext<'_>) {
    composite_schemas::validate(ctx);
    extension_names::validate_extension_names(ctx);
    subgraph_names::validate_subgraph_names(ctx);
    validate_root_nonempty(ctx);
    validate_fields(ctx);
    selection::validate_keys(ctx);
    directives::validate(ctx);
}

fn validate_root_nonempty(ctx: &mut ValidateContext<'_>) {
    if ctx.subgraphs.iter_subgraphs().all(|subgraph| {
        subgraph.query_type.is_none() && subgraph.mutation_type.is_none() && subgraph.subscription_type.is_none()
    }) {
        ctx.diagnostics.push_fatal(String::from(
            "None of the subgraphs define any root (Query, Mutation, Subscription) type. The federated graph cannot be empty.",
        ));
    }
}

fn validate_fields(ctx: &mut ValidateContext<'_>) {
    for field in ctx.subgraphs.iter_fields() {
        selection::validate_selections(ctx, field);
        validate_override_labels(ctx, field);
        composite_schemas::source_schema::lookup_returns_non_nullable_type(ctx, field);
        composite_schemas::source_schema::override_from_self(ctx, field);
    }
}

fn validate_override_labels(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldView<'_>) {
    let Some(label) = field
        .directives
        .r#override(ctx.subgraphs)
        .and_then(|directive| directive.label)
    else {
        return;
    };

    let Err(err) = ctx.subgraphs[label].parse::<OverrideLabel>() else {
        return;
    };

    ctx.diagnostics.push_fatal(format!(
        "Invalid @override label argument on {ty}.{field}: {err}",
        ty = ctx.subgraphs[ctx.subgraphs.at(field.parent_definition_id).name],
        field = ctx.subgraphs[field.name],
    ));
}
