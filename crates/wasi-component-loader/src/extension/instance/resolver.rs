use std::sync::Arc;

use engine::EngineOperationContext;
use engine_error::GraphqlError;
use event_queue::EventQueue;
use futures::future::BoxFuture;
use runtime::extension::Response;

use crate::extension::api::wit::{ArgumentsId, Directive, Field, FieldId, SubscriptionItem};

#[allow(unused_variables)]
pub(crate) trait ResolverExtensionInstance {
    fn prepare<'a>(
        &'a mut self,
        event_queue: Arc<EventQueue>,
        subgraph_name: &'a str,
        directive: Directive<'a>,
        field_id: FieldId,
        fields: &'a [Field<'a>],
    ) -> BoxFuture<'a, wasmtime::Result<Result<Vec<u8>, GraphqlError>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn resolve<'a>(
        &'a mut self,
        ctx: EngineOperationContext,
        headers: http::HeaderMap,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, wasmtime::Result<Response>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    #[allow(clippy::type_complexity)]
    fn create_subscription<'a>(
        &'a mut self,
        ctx: EngineOperationContext,
        headers: http::HeaderMap,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, wasmtime::Result<Result<Option<Vec<u8>>, GraphqlError>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    // Weird API to have double wasmtime::Result, but used as convenience for wasmsafe! macro as
    // the underlying WIT functions doesn't return a result, the only one we have right now.
    fn drop_subscription<'a>(
        &'a mut self,
        ctx: &'a EngineOperationContext,
    ) -> BoxFuture<'a, wasmtime::Result<wasmtime::Result<()>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn resolve_next_subscription_item<'a>(
        &'a mut self,
        ctx: &'a EngineOperationContext,
    ) -> BoxFuture<'a, wasmtime::Result<Result<Option<SubscriptionItem>, GraphqlError>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}
