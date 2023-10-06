use crate::{Context, ContextField, Error};

use super::ResolvedValue;

/// Some resolvers for implementing introspection
///
/// Currently most introspection is _not_ handled by these resolvers,
/// but instead by some legacy async_graphql code.  We want to get rid
/// of that sometime (ideally soon) though so expect we'll fill this out
/// sooner or later
#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum IntrospectionResolver {
    FederationServiceField,
}

impl IntrospectionResolver {
    pub async fn resolve(&self, ctx: &ContextField<'_>) -> Result<ResolvedValue, Error> {
        Ok(ResolvedValue::new(serde_json::Value::String(
            ctx.registry().export_sdl(true),
        )))
    }
}
