use graph_entities::ResponseNodeId;

use crate::{
    model::{__Schema, __Type},
    ContextField, LegacyOutputType, ServerError,
};

/// Resolves the introspection __type field
///
/// This calls into some legacy resolution stuff that we should definitely
/// remove sometime.
pub async fn resolve_type_field(ctx: &ContextField<'_>) -> Result<Option<ResponseNodeId>, ServerError> {
    let (_, type_name) = ctx.param_value::<String>("name", None)?;
    let ctx_obj = ctx.with_selection_set_legacy(&ctx.item.node.selection_set);
    let resolved = LegacyOutputType::resolve(
        &ctx.schema_env
            .registry
            .lookup_type(&type_name)
            .map(|ty| __Type::new_simple(&ctx.schema_env.registry, ty)),
        &ctx_obj,
        ctx.item,
    )
    .await?;

    Ok(Some(resolved))
}

/// Resolves the introspection __schema field
///
/// This calls into some legacy resolution stuff that we should definitely
/// remove sometime.
pub async fn resolve_schema_field(ctx: &ContextField<'_>) -> Result<Option<ResponseNodeId>, ServerError> {
    let ctx_obj = ctx.with_selection_set_legacy(&ctx.item.node.selection_set);
    let resolved = LegacyOutputType::resolve(&__Schema::new(&ctx.schema_env.registry), &ctx_obj, ctx.item).await?;

    Ok(Some(resolved))
}
