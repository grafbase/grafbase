use common_types::{auth, UdfKind};
use runtime::udf::{
    CustomResolverInvoker, CustomResolverRequestPayload, UdfError, UdfRequest, UdfRequestContext,
    UdfRequestContextRequest, UdfResponse,
};

use super::ResolvedValue;
use crate::{ContextExt, ContextField, Error, ErrorExtensionValues};

pub use registry_v2::resolvers::custom::CustomResolver;

impl From<UdfError> for crate::Error {
    fn from(value: UdfError) -> Self {
        Self::new(value.to_string())
    }
}

pub(super) async fn resolve(
    resolver: &CustomResolver,
    ctx: &ContextField<'_>,
    last_resolver_value: Option<&ResolvedValue>,
) -> Result<ResolvedValue, Error> {
    // Little hack while QP is not live
    //
    // We know the format of the parent value, we then apply some little magic to adapt it to
    // the proper format expected.
    let parent = last_resolver_value.cloned().unwrap_or_default();

    // This next bit of the hack is _tricky_.
    // - If our parent was a model we should have a struct like: { type: Value }.
    //   Where `type` is the name of the Model.
    // - If our parent was a connector type we just want to pass it in unchanged.
    //
    // We use the presence of the type key to try and differentiate these two
    let parent = parent
        .get_field(ctx.parent_type.name())
        .map(|model_data| model_data.take())
        .map(ResolvedValue::new)
        .unwrap_or(parent);

    // -- End of hack

    let runtime_ctx = ctx.data::<runtime::Context>()?;
    let custom_resolvers_engine = ctx.data::<CustomResolverInvoker>()?;
    let arguments = ctx
        .field()
        .arguments()?
        .into_iter()
        .map(|(name, value)| value.into_json().map(|value| (name.to_string(), value)))
        .collect::<serde_json::Result<_>>()?;
    let ray_id = runtime_ctx.ray_id();
    let auth_token: Option<&auth::ExecutionAuthToken> =
        ctx.data::<auth::ExecutionAuth>().ok().and_then(|auth| match auth {
            auth::ExecutionAuth::Token(token) => Some(token),
            _ => None,
        });

    let future = custom_resolvers_engine.invoke(
        ray_id,
        UdfRequest {
            name: &resolver.resolver_name,
            request_id: ray_id,
            payload: CustomResolverRequestPayload {
                arguments,
                parent: Some(parent.data_resolved().clone()),
                context: UdfRequestContext {
                    request: UdfRequestContextRequest {
                        jwt_claims: auth_token.map(|token| token.claims().clone()).unwrap_or_default(),
                        headers: serde_json::to_value(runtime_ctx.headers_as_map()).expect("must be valid"),
                    },
                },
                info: Some(serde_json::json!({
                    "fieldName": ctx.item.name.node.as_str(),
                    "path": ctx.response_path(),
                    "variableValues": &ctx.query_env.variables,
                })),
                secrets: runtime_ctx.secrets.clone(),
            },
            udf_kind: UdfKind::Resolver,
        },
    );

    match future.await? {
        UdfResponse::Success(value) => Ok(ResolvedValue::new(value)),
        UdfResponse::GraphQLError { message, extensions } => {
            let mut error = Error::new(message);
            error.extensions = extensions.map(|extensions| {
                ErrorExtensionValues(
                    extensions
                        .into_iter()
                        .filter_map(|(key, value)| Some((key, crate::Value::from_json(value).ok()?)))
                        .collect(),
                )
            });
            Err(error)
        }
        UdfResponse::Error(_err) => Err(UdfError::InvocationError.into()),
    }
}
