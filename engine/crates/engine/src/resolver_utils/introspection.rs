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
    let visible_types = ctx.schema_env.registry.find_visible_types(ctx);
    let resolved = LegacyOutputType::resolve(
        &ctx.schema_env
            .registry
            .types
            .get(&type_name)
            .filter(|_| visible_types.contains(type_name.as_str()))
            .map(|ty| __Type::new_simple(&ctx.schema_env.registry, &visible_types, ty)),
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
    let visible_types = ctx.schema_env.registry.find_visible_types(ctx);
    let resolved = LegacyOutputType::resolve(
        &__Schema::new(&ctx.schema_env.registry, &visible_types),
        &ctx_obj,
        ctx.item,
    )
    .await?;

    Ok(Some(resolved))
}
