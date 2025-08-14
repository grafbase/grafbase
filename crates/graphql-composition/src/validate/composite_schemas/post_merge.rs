use itertools::Itertools;

use crate::{
    composition_ir as ir, diagnostics::CompositeSchemasPostMergeValidationErrorCode, subgraphs,
    validate::ValidateContext,
};

// https://graphql.github.io/composite-schemas-spec/draft/#sec-Invalid-Field-Sharing
pub(crate) fn invalid_field_sharing(ctx: &mut ValidateContext<'_>, fields: &[subgraphs::FieldWalker<'_>]) {
    if fields.iter().any(|field| {
        field.is_part_of_key() || field.directives().shareable() || field.parent_definition().directives().shareable()
    }) {
        return;
    }

    let mut sources_of_truth = fields
        .iter()
        .filter(|field| {
            field.directives().r#override().is_none()
                && !field.directives().external()
                && !field.parent_definition().directives().external()
                && !field
                    .directives()
                    .iter_ir_directives()
                    .any(|d| matches!(d, ir::Directive::CompositeInternal(_)))
                // Federation v1 subgraphs are excluded, since shareable rules were different.
                && !field.parent_definition().subgraph().id.1.federation_spec.is_apollo_v1()
        })
        .peekable();

    let Some(first) = sources_of_truth.next() else {
        return;
    };

    // Single source of truth for a non-shareable field. That's fine.
    if sources_of_truth.peek().is_none() {
        return;
    }

    let field_name = first.name().as_str();
    let parent_name = first.parent_definition().name().as_str();
    let first_subgraph = first.parent_definition().subgraph().name().as_str();

    let others = sources_of_truth
        .map(|field| field.parent_definition().subgraph().name().as_str())
        .join(", ");

    ctx.diagnostics.push_composite_schemas_post_merge_validation_error(
        format!("The field `{parent_name}.{field_name}` is resolved by multiple subgraphs, but not marked as `@shareable`. The field must be marked as `@shareable` in at least one of the subgraphs: {first_subgraph}, {others}"),
        CompositeSchemasPostMergeValidationErrorCode::InvalidFieldSharing,
    );
}
