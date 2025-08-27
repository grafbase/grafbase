use crate::diagnostics::CompositeSchemasPostMergeValidationErrorCode;

use super::*;

pub(super) fn merge_root_fields(ctx: &mut Context<'_>) {
    let mut query_types = Vec::new();
    let mut mutation_types = Vec::new();
    let mut subscription_types = Vec::new();

    for subgraph in ctx.subgraphs.iter_subgraphs() {
        if let Some(query_type) = subgraph.query_type() {
            query_types.push(query_type);
        }

        if let Some(mutation_type) = subgraph.mutation_type() {
            mutation_types.push(mutation_type);
        }

        if let Some(subscription_type) = subgraph.subscription_type() {
            subscription_types.push(subscription_type);
        }
    }

    if let Some(query_id) = merge_fields("Query", &query_types, ctx) {
        ctx.set_query(query_id);
    };

    if let Some(mutation_id) = merge_fields("Mutation", &mutation_types, ctx) {
        ctx.set_mutation(mutation_id);
    }

    if let Some(subscription_id) = merge_fields("Subscription", &subscription_types, ctx) {
        ctx.set_subscription(subscription_id);
    }
}

fn merge_fields<'a>(
    root: &'static str,
    definitions: &[subgraphs::DefinitionWalker<'a>],
    ctx: &mut Context<'a>,
) -> Option<federated::ObjectId> {
    if definitions.is_empty() {
        return None;
    }

    let type_name = ctx.insert_static_str(root);
    let directives = collect_composed_directives(definitions.iter().map(|def| def.view().directives), ctx);

    let object_id = ctx.insert_object(type_name, None, directives);

    if let "Subscription" = root {
        for definition in definitions.iter().map(|def| def.view()) {
            if definition.directives.shareable(ctx.subgraphs) {
                ctx.diagnostics.push_composite_schemas_post_merge_validation_error(
                    format!(
                        "[{}] The Subscription type cannot be marked as @shareable.",
                        ctx.subgraphs[ctx.subgraphs.at(definition.subgraph_id).name],
                    ),
                    CompositeSchemasPostMergeValidationErrorCode::InvalidFieldSharing,
                );
            }
        }

        fields::for_each_field_group(definitions, |fields| {
            for shareable_field in fields
                .iter()
                .filter(|field| field.id.1.directives.shareable(ctx.subgraphs))
            {
                ctx.diagnostics.push_composite_schemas_post_merge_validation_error(
                    format!(
                        "[{}] Subscription root fields cannot be marked as @shareable: {}.{}.",
                        shareable_field.parent_definition().subgraph().name().as_str(),
                        shareable_field.parent_definition().name().as_str(),
                        shareable_field.name().as_str()
                    ),
                    CompositeSchemasPostMergeValidationErrorCode::InvalidFieldSharing,
                );
            }
        });
    }

    let fields = object::compose_fields(ctx, definitions, type_name);
    for field in fields {
        ctx.insert_field(field);
    }

    Some(object_id)
}
