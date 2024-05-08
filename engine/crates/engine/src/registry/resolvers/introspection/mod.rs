use registry_v2::resolvers::introspection::IntrospectionResolver;

use super::ResolvedValue;
use crate::{registry::RegistrySdlExt, Context, ContextField, Error};

pub async fn resolve(_resolver: &IntrospectionResolver, ctx: &ContextField<'_>) -> Result<ResolvedValue, Error> {
    Ok(ResolvedValue::new(serde_json::Value::String(
        ctx.registry().export_sdl(true),
    )))
}
