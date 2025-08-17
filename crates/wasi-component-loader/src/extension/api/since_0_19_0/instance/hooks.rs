use std::sync::Arc;

use engine::EngineOperationContext;
use engine_error::{ErrorCode, ErrorResponse, GraphqlError};
use engine_schema::GraphqlSubgraph;
use event_queue::EventQueue;
use futures::future::BoxFuture;
use http::{request, response};
use runtime::extension::{OnRequest, ReqwestParts};
use url::Url;

use crate::{
    extension::{
        HooksExtensionInstance,
        api::{since_0_19_0::world as wit19, wit::HttpMethod},
    },
    resources::{Headers, LegacyWasmContext},
};

impl HooksExtensionInstance for super::ExtensionInstanceSince0_19_0 {
    fn on_request<'a>(
        &'a mut self,
        event_queue: EventQueue,
        mut parts: request::Parts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<OnRequest, ErrorResponse>>> {
        Box::pin(async move {
            let headers = std::mem::take(&mut parts.headers);
            let url = parts.uri.to_string();

            let headers = self.store.data_mut().resources.push(Headers::from(headers))?;

            let event_queue = Arc::new(event_queue);
            let ctx = self
                .store
                .data_mut()
                .resources
                .push(LegacyWasmContext::from(event_queue.clone()))?;

            let method: HttpMethod = (&parts.method).try_into()?;

            let result = self
                .inner
                .grafbase_sdk_hooks()
                .call_on_request(
                    &mut self.store,
                    ctx,
                    wit19::HttpRequestPartsParam {
                        url: url.as_str(),
                        method,
                        headers,
                    },
                )
                .await?;

            let output = match result {
                Ok(wit19::OnRequestOutput { headers, contract_key }) => {
                    parts.headers = self.store.data_mut().resources.delete(headers)?.into_inner().unwrap();
                    Ok(OnRequest {
                        parts,
                        contract_key,
                        event_queue,
                        hooks_context: Default::default(),
                    })
                }
                Err(err) => Err(self
                    .store
                    .data_mut()
                    .take_error_response(err, ErrorCode::ExtensionError)?),
            };

            Ok(output)
        })
    }

    fn on_response(
        &mut self,
        event_queue: Arc<EventQueue>,
        _hooks_context: Arc<[u8]>,
        mut parts: response::Parts,
    ) -> BoxFuture<'_, wasmtime::Result<Result<response::Parts, String>>> {
        Box::pin(async move {
            let headers = std::mem::take(&mut parts.headers);
            let status = parts.status.as_u16();

            let headers = self.store.data_mut().resources.push(Headers::from(headers))?;

            let ctx = LegacyWasmContext::from(event_queue.clone());
            let queue = self.store.data_mut().resources.push(event_queue)?;
            let context = self.store.data_mut().resources.push(ctx)?;

            let result = self
                .inner
                .grafbase_sdk_hooks()
                .call_on_response(&mut self.store, context, status, headers, queue)
                .await?;

            let result = match result {
                Ok(headers) => {
                    parts.headers = self.store.data_mut().resources.delete(headers)?.into_inner().unwrap();
                    Ok(parts)
                }
                Err(err) => Err(err),
            };
            Ok(result)
        })
    }

    fn on_graphql_subgraph_request<'a>(
        &'a mut self,
        ctx: EngineOperationContext,
        _subgraph: GraphqlSubgraph<'a>,
        ReqwestParts { url, method, headers }: ReqwestParts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<ReqwestParts, GraphqlError>>> {
        Box::pin(async move {
            let method: HttpMethod = (&method).try_into()?;
            let headers = self.store.data_mut().resources.push(Headers::from(headers))?;
            let context = self.store.data_mut().resources.push(LegacyWasmContext::from(&ctx))?;
            let result = self
                .inner
                .grafbase_sdk_hooks()
                .call_on_subgraph_request(
                    &mut self.store,
                    context,
                    wit19::HttpRequestPartsParam {
                        url: url.as_str(),
                        method,
                        headers,
                    },
                )
                .await?;

            let result = match result {
                Ok(parts) => {
                    let headers = self
                        .store
                        .data_mut()
                        .resources
                        .delete(parts.headers)?
                        .into_inner()
                        .unwrap();
                    // Must be *after* the headers, to ensure the wasm store is kept clean.
                    let url = match parts.url.parse::<Url>() {
                        Ok(url) => url,
                        Err(err) => {
                            tracing::error!("Invalid URL ({:?}) returned by extension: {err}", parts.url);
                            return Ok(Err(GraphqlError::internal_extension_error()));
                        }
                    };

                    Ok(ReqwestParts {
                        url,
                        method: parts.method.into(),
                        headers,
                    })
                }
                Err(err) => Err(err.into_graphql_error(ErrorCode::ExtensionError)),
            };
            Ok(result)
        })
    }
}
