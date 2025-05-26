use engine_error::GraphqlError;
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::{
    Error,
    extension::{FieldResolverExtensionInstance, InputList, api::wit::FieldDefinitionDirective},
    resources::{Headers, Lease},
};

impl FieldResolverExtensionInstance for super::ExtensionInstanceSince0_16_0 {
    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, Result<Vec<Result<Data, GraphqlError>>, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let inputs = inputs.0.iter().map(Vec::as_slice).collect::<Vec<_>>();

            let result = self
                .inner
                .grafbase_sdk_field_resolver()
                .call_resolve_field(&mut self.store, headers, subgraph_name, directive, &inputs)
                .await?;

            self.poisoned = false;

            Ok(result?.into())
        })
    }

    fn subscription_key<'a>(
        &'a mut self,
        headers: Lease<http::HeaderMap>,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(Lease<http::HeaderMap>, Option<Vec<u8>>), Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let headers_rep = headers.rep();

            let result = self
                .inner
                .grafbase_sdk_field_resolver()
                .call_subscription_key(&mut self.store, headers, subgraph_name, directive)
                .await?;

            let headers = self
                .store
                .data_mut()
                .take_resource::<Headers>(headers_rep)?
                .into_lease()
                .unwrap();

            self.poisoned = false;

            let key = result?;
            Ok((headers, key))
        })
    }

    fn resolve_subscription<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;

            let result = self
                .inner
                .grafbase_sdk_field_resolver()
                .call_resolve_subscription(&mut self.store, headers, subgraph_name, directive)
                .await?;

            self.poisoned = false;

            result.map_err(Into::into)
        })
    }

    fn resolve_next_subscription_item(
        &mut self,
    ) -> BoxFuture<'_, Result<Option<Vec<Result<Data, GraphqlError>>>, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let result = self
                .inner
                .grafbase_sdk_field_resolver()
                .call_resolve_next_subscription_item(&mut self.store)
                .await?;

            self.poisoned = false;

            Ok(result?.map(Into::into))
        })
    }
}
