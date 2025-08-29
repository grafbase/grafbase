use itertools::Itertools;

use crate::{
    composition_ir as ir, diagnostics::CompositeSchemasPostMergeValidationErrorCode, subgraphs,
    validate::ValidateContext,
};

// https://graphql.github.io/composite-schemas-spec/draft/#sec-Invalid-Field-Sharing
pub(crate) fn invalid_field_sharing(ctx: &mut ValidateContext<'_>, fields: &[subgraphs::FieldView<'_>]) {
    if fields.iter().any(|field| {
        field.is_part_of_key(ctx.subgraphs)
            || field.directives.shareable(ctx.subgraphs)
            || ctx
                .subgraphs
                .at(field.parent_definition_id)
                .directives
                .shareable(ctx.subgraphs)
    }) {
        return;
    }

    let mut sources_of_truth = fields
        .iter()
        .filter(|field| {
            let directives = field.directives;
            let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
            let subgraph = ctx.subgraphs.at(parent_definition.subgraph_id);

            directives.r#override(ctx.subgraphs).is_none()
                && !directives.external(ctx.subgraphs)
                && !parent_definition.directives.external(ctx.subgraphs)
                && !directives
                    .iter_ir_directives(ctx.subgraphs)
                    .any(|d| matches!(d, ir::Directive::CompositeInternal(_)))
                // Federation v1 subgraphs are excluded, since shareable rules were different.
                && !subgraph.federation_spec.is_apollo_v1()
        })
        .peekable();

    let Some(first) = sources_of_truth.next() else {
        return;
    };

    // Single source of truth for a non-shareable field. That's fine.
    if sources_of_truth.peek().is_none() {
        return;
    }

    let field_name = &ctx.subgraphs[first.name];
    let parent = ctx.subgraphs.at(first.parent_definition_id);
    let parent_name = &ctx.subgraphs[parent.name];
    let first_subgraph = &ctx.subgraphs[ctx.subgraphs.at(parent.subgraph_id).name];

    let others = sources_of_truth
        .map(|field| {
            let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
            ctx.subgraphs[ctx.subgraphs[parent_definition.subgraph_id].name].as_ref()
        })
        .join(", ");

    ctx.diagnostics.push_composite_schemas_post_merge_validation_error(
        format!("The field `{parent_name}.{field_name}` is resolved by multiple subgraphs, but not marked as `@shareable`. The field must be marked as `@shareable` in at least one of the subgraphs: {first_subgraph}, {others}"),
        CompositeSchemasPostMergeValidationErrorCode::InvalidFieldSharing,
    );
}
