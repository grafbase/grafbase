use std::sync::Arc;

use engine_error::{ErrorCode, ErrorResponse};
use event_queue::EventQueue;
use futures::future::BoxFuture;
use http::{request, response};
use runtime::extension::{ExtensionRequestContext, OnRequest};

use crate::{
    extension::{HooksExtensionInstance, api::since_0_17_0::world::HttpMethod},
    resources::{EventQueueProxy, LegacyHeaders, LegacyWasmContext, OwnedOrShared},
};

impl HooksExtensionInstance for super::ExtensionInstanceSince0_17_0 {
    fn on_request<'a>(
        &'a mut self,
        event_queue: EventQueue,
        mut parts: request::Parts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<OnRequest, ErrorResponse>>> {
        Box::pin(async move {
            let headers = std::mem::take(&mut parts.headers);
            let url = parts.uri.to_string();

            let headers = OwnedOrShared::Owned(headers);
            let headers = self.store.data_mut().resources.push(LegacyHeaders::from(headers))?;
            let headers_rep = headers.rep();

            let event_queue = Arc::new(event_queue);
            let ctx = self
                .store
                .data_mut()
                .resources
                .push(LegacyWasmContext::from(event_queue.clone()))?;

            let method = match &parts.method {
                m if m == http::Method::GET => HttpMethod::Get,
                m if m == http::Method::POST => HttpMethod::Post,
                m if m == http::Method::PUT => HttpMethod::Put,
                m if m == http::Method::DELETE => HttpMethod::Delete,
                m if m == http::Method::PATCH => HttpMethod::Patch,
                m if m == http::Method::HEAD => HttpMethod::Head,
                m if m == http::Method::OPTIONS => HttpMethod::Options,
                m => {
                    return Err(wasmtime::Error::msg(format!("Invalid HTTP method: {m}")));
                }
            };

            let result = self
                .inner
                .grafbase_sdk_hooks()
                .call_on_request(&mut self.store, ctx, &url, method, headers)
                .await?;

            parts.headers = self
                .store
                .data_mut()
                .take_leased_resource(headers_rep)?
                .into_inner()
                .unwrap();

            let output = match result {
                Ok(()) => Ok(OnRequest {
                    parts,
                    contract_key: None,
                    context: Arc::new(ExtensionRequestContext {
                        event_queue,
                        hooks_context: Vec::new(),
                    }),
                }),
                Err(err) => Err(self
                    .store
                    .data_mut()
                    .take_error_response_sdk17(err, ErrorCode::ExtensionError)?),
            };

            Ok(output)
        })
    }

    fn on_response(
        &mut self,
        ctx: Arc<ExtensionRequestContext>,
        mut parts: response::Parts,
    ) -> BoxFuture<'_, wasmtime::Result<Result<response::Parts, String>>> {
        Box::pin(async move {
            let headers = std::mem::take(&mut parts.headers);
            let status = parts.status.as_u16();

            let headers = OwnedOrShared::Owned(headers);
            let headers = self.store.data_mut().resources.push(LegacyHeaders::from(headers))?;
            let headers_rep = headers.rep();

            let ctx = LegacyWasmContext::from(ctx.event_queue.clone());
            let queue = self.store.data_mut().resources.push(EventQueueProxy(ctx.clone()))?;
            let context = self.store.data_mut().resources.push(ctx)?;

            let result = self
                .inner
                .grafbase_sdk_hooks()
                .call_on_response(&mut self.store, context, status, headers, queue)
                .await?;

            parts.headers = self
                .store
                .data_mut()
                .take_leased_resource(headers_rep)?
                .into_inner()
                .unwrap();

            Ok(result.map(|_| parts))
        })
    }
}
