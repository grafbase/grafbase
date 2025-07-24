use engine_error::{ErrorCode, ErrorResponse};
use futures::future::BoxFuture;
use http::{request, response};
use runtime::extension::OnRequest;

use crate::{
    WasmContext,
    extension::{
        HooksExtensionInstance,
        api::wit::{self, HttpMethod, HttpRequestPartsParam},
    },
    resources::{EventQueueProxy, Headers},
};

impl HooksExtensionInstance for super::ExtensionInstanceSince0_19_0 {
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
                Ok(wit::OnRequestOutput { headers, contract_key }) => {
                    parts.headers = self.store.data_mut().resources.delete(headers)?.into_inner().unwrap();
                    Ok(OnRequest {
                        context,
                        parts,
                        contract_key,
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
}
