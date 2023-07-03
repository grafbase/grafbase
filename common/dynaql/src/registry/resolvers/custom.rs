use super::{ResolvedValue, ResolverContext, ResolverTrait};

use crate::{Context, Error};
use dynamodb::attribute_to_value;
use dynomite::AttributeValue;
use grafbase::UdfKind;
use grafbase_runtime::udf::{
    CustomResolverRequestPayload, CustomResolversEngine, UdfRequest, UdfRequestContext,
    UdfRequestContextRequest,
};
use grafbase_runtime::GraphqlRequestExecutionContext;

use send_wrapper::SendWrapper;

use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct CustomResolver {
    pub resolver_name: String,
}

#[async_trait::async_trait]
impl ResolverTrait for CustomResolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        _resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        use crate::registry::resolver_chain::ResolverChainNode;

        // Little hack while QP is not live
        //
        // We know the format of the parent value, we then apply some little magic to adapt it to
        // the proper format expected.
        let parent_data = last_resolver_value
            .map(|x| x.data_resolved.clone())
            .unwrap_or(Arc::new(serde_json::json!({})));

        // This next bit of the hack is _tricky_.
        // - If our parent was a model we should have a struct like: { type: Value }.
        //   Where `type` is the name of the Model.
        // - If our parent was a connector type we just want to pass it in unchanged.
        //
        // We use the presence of the `type` key to try and differentiate these two
        let model_data = parent_data.as_object().and_then(|parent_object| {
            // parent_object might also contain relations so we need to find the
            // correct key to take.  We're currently resolving a field so we look
            // two levels up the resolver chain to find the current type we're within.
            ctx.resolver_node
                .as_ref()
                .and_then(|node| Some(node.parent?.ty?.name()))
                .and_then(|current_type_name| parent_object.get(current_type_name))
        });

        let parent = match model_data {
            Some(model_data) => dynamodb_to_json(model_data.clone()),
            None => (*parent_data).clone(),
        };

        // -- End of hack

        let graphql = ctx.data::<grafbase_runtime::GraphqlRequestExecutionContext>()?;
        let custom_resolvers_engine = ctx.data::<CustomResolversEngine>()?;
        let arguments = ctx
            .field()
            .arguments()?
            .into_iter()
            .map(|(name, value)| value.into_json().map(|value| (name.to_string(), value)))
            .collect::<serde_json::Result<_>>()?;
        let future = SendWrapper::new(custom_resolvers_engine.invoke(
            &ctx.data::<GraphqlRequestExecutionContext>()?.ray_id,
            UdfRequest {
                name: self.resolver_name.clone(),
                payload: CustomResolverRequestPayload {
                    arguments,
                    parent: Some(parent),
                    context: UdfRequestContext {
                        request: UdfRequestContextRequest {
                            headers: serde_json::to_value(&graphql.headers).expect("must be valid"),
                        },
                    },
                    info: Some(serde_json::json!({
                        "fieldName": ctx.item.name.node.as_str(),
                        "path": ctx.resolver_node.as_ref().map(ResolverChainNode::to_response_path),
                        "variableValues": &ctx.query_env.variables,
                    })),
                },
                udf_kind: UdfKind::Resolver,
            },
        ));

        let value = Box::pin(future).await?;
        Ok(ResolvedValue::new(Arc::new(value.value)))
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
                    let attribute = serde_json::from_value::<AttributeValue>(x)
                        .ok()
                        .unwrap_or_default();

                    (field, attribute_to_value(attribute))
                })
                .collect();
            serde_json::Value::Object(temp)
        }
        _ => serde_json::json!({}),
    }
}
