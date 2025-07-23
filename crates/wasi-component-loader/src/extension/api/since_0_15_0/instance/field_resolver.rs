use engine_error::{ErrorCode, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::{
    extension::{FieldResolverExtensionInstance, InputList, SubscriptionItem, api::wit::FieldDefinitionDirective},
    resources::{Headers, Lease},
};

impl FieldResolverExtensionInstance for super::ExtensionInstanceSince0_15_0 {
    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, wasmtime::Result<Result<Vec<Result<Data, GraphqlError>>, GraphqlError>>> {
        Box::pin(async move {
            let headers = self.store.data_mut().resources.push(Headers::from(headers))?;
            let inputs = inputs.0.iter().map(Vec::as_slice).collect::<Vec<_>>();

            let result = self
                .inner
                .grafbase_sdk_field_resolver()
                .call_resolve_field(&mut self.store, headers, subgraph_name, directive, &inputs)
                .await?;

            Ok(result
                .map(Into::into)
                .map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn subscription_key<'a>(
        &'a mut self,
        headers: Lease<http::HeaderMap>,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<(Lease<http::HeaderMap>, Option<Vec<u8>>), GraphqlError>>> {
        Box::pin(async move {
            let headers = self.store.data_mut().resources.push(Headers::from(headers))?;
            let headers_rep = headers.rep();

            let result = self
                .inner
                .grafbase_sdk_field_resolver()
                .call_subscription_key(&mut self.store, headers, subgraph_name, directive)
                .await?;

            let headers = self.store.data_mut().take_leased_resource(headers_rep)?;

            Ok(result
                .map(|key| (headers, key))
                .map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn resolve_subscription<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<(), GraphqlError>>> {
        Box::pin(async move {
            let headers = self.store.data_mut().resources.push(Headers::from(headers))?;

            let result = self
                .inner
                .grafbase_sdk_field_resolver()
                .call_resolve_subscription(&mut self.store, headers, subgraph_name, directive)
                .await?;

            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn field_resolver_resolve_next_subscription_item(
        &mut self,
    ) -> BoxFuture<'_, wasmtime::Result<Result<Option<SubscriptionItem>, GraphqlError>>> {
        Box::pin(async move {
            let result = self
                .inner
                .grafbase_sdk_field_resolver()
                .call_resolve_next_subscription_item(&mut self.store)
                .await?;

            Ok(result
                .map(|opt| opt.map(Into::into))
                .map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }
}
