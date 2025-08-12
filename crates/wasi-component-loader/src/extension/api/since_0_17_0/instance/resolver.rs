use engine::EngineOperationContext;
use engine_error::{ErrorCode, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::Response;

use crate::{
    extension::{
        ResolverExtensionInstance,
        api::wit::{ArgumentsId, Directive, Field, FieldId, SubscriptionItem},
    },
    resources::{LegacyHeaders, LegacyWasmContext},
};

impl ResolverExtensionInstance for super::ExtensionInstanceSince0_17_0 {
    fn prepare<'a>(
        &'a mut self,
        subgraph_name: &'a str,
        directive: Directive<'a>,
        field_id: FieldId,
        fields: &'a [Field<'a>],
    ) -> BoxFuture<'a, wasmtime::Result<Result<Vec<u8>, GraphqlError>>> {
        Box::pin(async move {
            let context = self.store.data_mut().resources.push(LegacyWasmContext::default())?;

            let result = self
                .inner
                .grafbase_sdk_resolver()
                .call_prepare(&mut self.store, context, subgraph_name, directive, field_id, fields)
                .await?;

            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn resolve<'a>(
        &'a mut self,
        ctx: EngineOperationContext,
        headers: http::HeaderMap,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, wasmtime::Result<Response>> {
        Box::pin(async move {
            let headers = self.store.data_mut().resources.push(LegacyHeaders::from(headers))?;
            let context = self.store.data_mut().resources.push(LegacyWasmContext::from(&ctx))?;

            let response = self
                .inner
                .grafbase_sdk_resolver()
                .call_resolve(&mut self.store, context, prepared, headers, arguments)
                .await?;

            Ok(response.into())
        })
    }

    fn create_subscription<'a>(
        &'a mut self,
        ctx: EngineOperationContext,
        headers: http::HeaderMap,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, wasmtime::Result<Result<Option<Vec<u8>>, GraphqlError>>> {
        Box::pin(async move {
            let headers = self.store.data_mut().resources.push(LegacyHeaders::from(headers))?;
            let context = self.store.data_mut().resources.push(LegacyWasmContext::from(&ctx))?;

            let result = self
                .inner
                .grafbase_sdk_resolver()
                .call_create_subscription(&mut self.store, context, prepared, headers, arguments)
                .await?;

            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn drop_subscription<'a>(
        &'a mut self,
        ctx: &'a EngineOperationContext,
    ) -> BoxFuture<'a, wasmtime::Result<wasmtime::Result<()>>> {
        Box::pin(async move {
            let context = self.store.data_mut().resources.push(LegacyWasmContext::from(ctx))?;

            self.inner
                .grafbase_sdk_resolver()
                .call_drop_subscription(&mut self.store, context)
                .await?;

            Ok(Ok(()))
        })
    }

    fn resolve_next_subscription_item<'a>(
        &'a mut self,
        ctx: &'a EngineOperationContext,
    ) -> BoxFuture<'a, wasmtime::Result<Result<Option<SubscriptionItem>, GraphqlError>>> {
        Box::pin(async move {
            let context = self.store.data_mut().resources.push(LegacyWasmContext::from(ctx))?;

            let result = self
                .inner
                .grafbase_sdk_resolver()
                .call_resolve_next_subscription_item(&mut self.store, context)
                .await?;

            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }
}
