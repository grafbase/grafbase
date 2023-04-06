use super::{ResolvedValue, ResolverContext, ResolverTrait};

use crate::{Context, Error};
use dynamodb::attribute_to_value;
use dynomite::AttributeValue;
use grafbase_runtime::custom_resolvers::{
    CustomResolverRequest, CustomResolverRequestPayload, CustomResolversEngine,
};

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
        // Little hack while QP is not live
        //
        // We know the format of the parent value, we then apply some little magic to adapt it to
        // the proper format expected.
        let parent_data = last_resolver_value
            .map(|x| x.data_resolved.clone())
            .unwrap_or(Arc::new(serde_json::json!({})));

        // We take the first item as the parent has a struct like: { type: Value }.
        let parent_data = parent_data
            .as_object()
            .and_then(|x| x.iter().next())
            .map(|(_, x)| x.clone())
            .unwrap_or(serde_json::json!({}));

        // Magic function to convert the dynamodb format to the format we want to have on the
        // ResolveR.
        let value = match parent_data {
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
        };

        // -- End of hack

        let custom_resolvers_engine = ctx.data::<CustomResolversEngine>()?;
        let arguments = ctx
            .field()
            .arguments()?
            .into_iter()
            .map(|(name, value)| value.into_json().map(|value| (name.to_string(), value)))
            .collect::<serde_json::Result<_>>()?;
        let future = SendWrapper::new(custom_resolvers_engine.invoke(
            ctx.data()?,
            CustomResolverRequest {
                resolver_name: self.resolver_name.clone(),
                payload: CustomResolverRequestPayload {
                    arguments,
                    parent: Some(value),
                },
            },
        ));
        let value = Box::pin(future).await?;
        Ok(ResolvedValue::new(Arc::new(value.value)))
    }
}
