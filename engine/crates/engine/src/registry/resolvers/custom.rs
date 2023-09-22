use std::hash::Hash;

use common_types::UdfKind;
use dynamodb::attribute_to_value;
use dynomite::AttributeValue;
use runtime::{
    udf::{
        CustomResolverError, CustomResolverRequestPayload, CustomResolverResponse, CustomResolversEngine, UdfRequest,
        UdfRequestContext, UdfRequestContextRequest,
    },
    GraphqlRequestExecutionContext,
};

use super::ResolvedValue;
use crate::{ContextExt, ContextField, Error, ErrorExtensionValues};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct CustomResolver {
    pub resolver_name: String,
}

impl CustomResolver {
    pub(super) async fn resolve(
        &self,
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
            .map(|model_data| dynamodb_to_json(model_data.take()))
            .map(ResolvedValue::new)
            .unwrap_or(parent);

        // -- End of hack

        let graphql = ctx.data::<runtime::GraphqlRequestExecutionContext>()?;
        let custom_resolvers_engine = ctx.data::<CustomResolversEngine>()?;
        let arguments = ctx
            .field()
            .arguments()?
            .into_iter()
            .map(|(name, value)| value.into_json().map(|value| (name.to_string(), value)))
            .collect::<serde_json::Result<_>>()?;
        let ray_id = &ctx.data::<GraphqlRequestExecutionContext>()?.ray_id;
        let future = custom_resolvers_engine.invoke(
            ray_id,
            UdfRequest {
                name: &self.resolver_name,
                request_id: ray_id,
                payload: CustomResolverRequestPayload {
                    arguments,
                    parent: Some(parent.data_resolved().clone()),
                    context: UdfRequestContext {
                        request: UdfRequestContextRequest {
                            headers: serde_json::to_value(&graphql.headers).expect("must be valid"),
                        },
                    },
                    info: Some(serde_json::json!({
                        "fieldName": ctx.item.name.node.as_str(),
                        "path": ctx.response_path(),
                        "variableValues": &ctx.query_env.variables,
                    })),
                },
                udf_kind: UdfKind::Resolver,
            },
        );

        match future.await? {
            CustomResolverResponse::Success(value) => Ok(ResolvedValue::new(value)),
            CustomResolverResponse::GraphQLError { message, extensions } => {
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
            #[allow(unused_variables)]
            CustomResolverResponse::Error(err) => {
                log::error!(ctx.trace_id(), "UDF error: {err}");

                Err(CustomResolverError::InvocationError.into())
            }
        }
    }
}

/// Magic function to convert the dynamodb format to the format we want to have on the
/// resolver.
fn dynamodb_to_json(model_data: serde_json::Value) -> serde_json::Value {
    match model_data {
        serde_json::Value::Object(val) => {
            let temp = val
                .into_iter()
                .filter_map(|(field, val)| {
                    if field.starts_with('_') {
                        let new_field = match field.as_str() {
                            "__created_at" => Some("createdAt"),
                            "__updated_at" => Some("updatedAt"),
                            "__sk" => Some("id"),
                            _ => None,
                        };
                        new_field.map(|x| (x.to_string(), val))
                    } else {
                        Some((field, val))
                    }
                })
                .map(|(field, x)| {
                    let attribute = serde_json::from_value::<AttributeValue>(x).ok().unwrap_or_default();

                    (field, attribute_to_value(attribute))
                })
                .collect();
            serde_json::Value::Object(temp)
        }
        _ => serde_json::json!({}),
    }
}
