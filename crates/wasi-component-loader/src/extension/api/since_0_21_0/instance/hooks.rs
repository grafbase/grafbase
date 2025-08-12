use engine_error::{ErrorCode, ErrorResponse, GraphqlError};
use engine_schema::GraphqlSubgraph;
use futures::future::BoxFuture;
use http::{request, response};
use runtime::extension::{OnRequest, ReqwestParts};
use url::Url;

use crate::{
    WasmContext,
    extension::{
        HooksExtensionInstance,
        api::wit::{self, HttpMethod, HttpRequestPartsParam},
    },
    resources::{EventQueueProxy, Headers},
};

impl HooksExtensionInstance for super::ExtensionInstanceSince0_21_0 {
    fn on_request<'a>(
        &'a mut self,
        context: WasmContext,
        mut parts: request::Parts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<OnRequest<WasmContext>, ErrorResponse>>> {
        Box::pin(async move {
            let headers = std::mem::take(&mut parts.headers);
            let url = parts.uri.to_string();

            let headers = self.store.data_mut().resources.push(Headers::from(headers))?;

            let ctx = self.store.data_mut().resources.push(context.clone())?;

            let method: HttpMethod = (&parts.method).try_into()?;

            let result = self
                .inner
                .grafbase_sdk_hooks()
                .call_on_request(
                    &mut self.store,
                    ctx,
                    HttpRequestPartsParam {
                        url: url.as_str(),
                        method,
                        headers,
                    },
                )
                .await?;

            let output = match result {
                Ok(wit::OnRequestOutput {
                    headers,
                    contract_key,
                    state,
                }) => {
                    parts.headers = self.store.data_mut().resources.delete(headers)?.into_inner().unwrap();
                    Ok(OnRequest {
                        context,
                        parts,
                        contract_key,
                        state,
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
        context: WasmContext,
        mut parts: response::Parts,
    ) -> BoxFuture<'_, wasmtime::Result<Result<response::Parts, String>>> {
        Box::pin(async move {
            let headers = std::mem::take(&mut parts.headers);
            let status = parts.status.as_u16();

            let headers = self.store.data_mut().resources.push(Headers::from(headers))?;

            let queue = self.store.data_mut().resources.push(EventQueueProxy(context.clone()))?;
            let context = self.store.data_mut().resources.push(context.clone())?;

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
        context: &'a WasmContext,
        subgraph: GraphqlSubgraph<'a>,
        ReqwestParts { url, method, headers }: ReqwestParts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<ReqwestParts, GraphqlError>>> {
        Box::pin(async move {
            let method: HttpMethod = (&method).try_into()?;
            let headers = self.store.data_mut().resources.push(Headers::from(headers))?;
            let context = self.store.data_mut().resources.push(context.clone())?;
            let result = self
                .inner
                .grafbase_sdk_hooks()
                .call_on_graphql_subgraph_request(
                    &mut self.store,
                    context,
                    subgraph.name(),
                    HttpRequestPartsParam {
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
