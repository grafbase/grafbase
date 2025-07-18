use futures::future::BoxFuture;
use http::{request, response};

use crate::{
    ErrorResponse, SharedContext,
    extension::{HooksInstance, api::wit::HttpMethod},
    resources::{EventQueueProxy, Headers, Lease},
};

impl HooksInstance for super::ExtensionInstanceSince0_18_0 {
    fn on_request(
        &mut self,
        context: SharedContext,
        mut parts: request::Parts,
    ) -> BoxFuture<'_, Result<request::Parts, ErrorResponse>> {
        Box::pin(async move {
            self.poisoned = true;

            let headers = std::mem::take(&mut parts.headers);
            let url = parts.uri.to_string();

            let headers = Lease::Singleton(headers);
            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let headers_rep = headers.rep();

            let context = self.store.data_mut().push_resource(context)?;

            let method = match &parts.method {
                m if m == http::Method::GET => HttpMethod::Get,
                m if m == http::Method::POST => HttpMethod::Post,
                m if m == http::Method::PUT => HttpMethod::Put,
                m if m == http::Method::DELETE => HttpMethod::Delete,
                m if m == http::Method::PATCH => HttpMethod::Patch,
                m if m == http::Method::HEAD => HttpMethod::Head,
                m if m == http::Method::OPTIONS => HttpMethod::Options,
                m => {
                    return Err(ErrorResponse::Internal(anyhow::Error::msg(format!(
                        "Invalid HTTP method: {m}"
                    ))));
                }
            };

            let result = self
                .inner
                .grafbase_sdk_hooks()
                .call_on_request(&mut self.store, context, &url, method, headers)
                .await?;

            parts.headers = self
                .store
                .data_mut()
                .take_resource::<Headers>(headers_rep)?
                .into_lease()
                .unwrap()
                .into_inner()
                .unwrap();

            self.poisoned = false;

            result.map_err(|err| ErrorResponse::from_wit(&mut self.store, err))?;

            Ok(parts)
        })
    }

    fn on_response(
        &mut self,
        context: SharedContext,
        mut parts: response::Parts,
    ) -> BoxFuture<'_, anyhow::Result<response::Parts>> {
        Box::pin(async move {
            self.poisoned = true;

            let headers = std::mem::take(&mut parts.headers);
            let status = parts.status.as_u16();

            let headers = Lease::Singleton(headers);
            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let headers_rep = headers.rep();

            let queue = self.store.data_mut().push_resource(EventQueueProxy(context.clone()))?;
            let context = self.store.data_mut().push_resource(context)?;

            let result = self
                .inner
                .grafbase_sdk_hooks()
                .call_on_response(&mut self.store, context, status, headers, queue)
                .await?;

            parts.headers = self
                .store
                .data_mut()
                .take_resource::<Headers>(headers_rep)?
                .into_lease()
                .unwrap()
                .into_inner()
                .unwrap();

            self.poisoned = false;

            match result {
                Ok(()) => Ok(parts),
                Err(err) => Err(anyhow::Error::msg(err)),
            }
        })
    }
}
