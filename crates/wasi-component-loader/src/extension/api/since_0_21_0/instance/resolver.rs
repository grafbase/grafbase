use std::sync::Arc;

use engine::EngineOperationContext;
use engine_error::{ErrorCode, GraphqlError};
use event_queue::EventQueue;
use futures::future::BoxFuture;
use runtime::extension::Response;

use crate::extension::{
    ResolverExtensionInstance,
    api::since_0_21_0::wit::{self, ArgumentsId, Directive, Field, FieldId, SubscriptionItem},
};

impl ResolverExtensionInstance for super::ExtensionInstanceSince0_21_0 {
    fn prepare<'a>(
        &'a mut self,
        event_queue: Arc<EventQueue>,
        subgraph_name: &'a str,
        directive: Directive<'a>,
        field_id: FieldId,
        fields: &'a [Field<'a>],
    ) -> BoxFuture<'a, wasmtime::Result<Result<Vec<u8>, GraphqlError>>> {
        Box::pin(async move {
            let resources = &mut self.store.data_mut().resources;
            let event_queue = resources.push(event_queue)?;
            let result = self
                .inner
                .grafbase_sdk_resolver()
                .call_prepare(&mut self.store, event_queue, subgraph_name, directive, field_id, fields)
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
            let resources = &mut self.store.data_mut().resources;
            let headers = resources.push(wit::Headers::from(headers))?;
            let event_queue = resources.push(ctx.event_queue().clone())?;
            let ctx = resources.push(ctx)?;

            let response = self
                .inner
                .grafbase_sdk_resolver()
                .call_resolve(&mut self.store, event_queue, ctx, prepared, headers, arguments)
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
            let resources = &mut self.store.data_mut().resources;
            let headers = resources.push(wit::Headers::from(headers))?;
            let event_queue = resources.push(ctx.event_queue().clone())?;
            let ctx = resources.push(ctx)?;

            let result = self
                .inner
                .grafbase_sdk_resolver()
                .call_create_subscription(&mut self.store, event_queue, ctx, prepared, headers, arguments)
                .await?;

            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn drop_subscription<'a>(
        &'a mut self,
        _ctx: &'a EngineOperationContext,
    ) -> BoxFuture<'a, wasmtime::Result<wasmtime::Result<()>>> {
        Box::pin(async move {
            self.inner
                .grafbase_sdk_resolver()
                .call_drop_subscription(&mut self.store)
                .await?;

            Ok(Ok(()))
        })
    }

    fn resolve_next_subscription_item<'a>(
        &'a mut self,
        _ctx: &'a EngineOperationContext,
    ) -> BoxFuture<'a, wasmtime::Result<Result<Option<SubscriptionItem>, GraphqlError>>> {
        Box::pin(async move {
            let result = self
                .inner
                .grafbase_sdk_resolver()
                .call_resolve_next_subscription_item(&mut self.store)
                .await?;

            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }
}
