use std::time::Duration;

use bytes::Bytes;
use futures::{TryFutureExt, future::BoxFuture, stream::FuturesOrdered};
use tokio_stream::StreamExt as _;
use wasmtime::component::Resource;

pub use super::grafbase::sdk::http_client::*;
use crate::{WasiState, http_client::send_request, resources::WasmOwnedOrLease};

impl Host for WasiState {}

impl HostHttpClient for WasiState {
    async fn execute(&mut self, request: HttpRequest) -> wasmtime::Result<Result<HttpResponse, HttpError>> {
        if !self.network_enabled() {
            return Ok(Err(HttpError::Connect("Network is disabled".into())));
        }

        let request = match convert_http_request(self, request)? {
            Ok(req) => req,
            Err(e) => return Ok(Err(e)),
        };

        let response = match send_request(request, self.request_durations().clone()).await {
            Ok(resp) => resp,
            Err(e) => return Ok(Err(e.into())),
        };

        let (parts, body) = response.into_parts();
        let headers = self.push_resource(WasmOwnedOrLease::Owned(parts.headers))?;

        Ok(Ok(HttpResponse {
            status: parts.status.as_u16(),
            headers,
            // FIXME: Avoid unnecessary copy....
            body: body.to_vec(),
        }))
    }

    async fn execute_many(
        &mut self,
        requests: Vec<HttpRequest>,
    ) -> wasmtime::Result<Vec<Result<HttpResponse, HttpError>>> {
        if !self.network_enabled() {
            let err = HttpError::Connect("Network is disabled".into());
            let mut results = Vec::with_capacity(requests.len());
            for _ in 0..requests.len() {
                results.push(Err(err.clone()));
            }
            return Ok(results);
        }

        requests
            .into_iter()
            .map(|request| convert_http_request(self, request))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|request| {
                let request_durations = self.request_durations().clone();
                let fut: BoxFuture<'_, Result<http::Response<Bytes>, HttpError>> = match request {
                    Ok(request) => Box::pin(send_request(request, request_durations).map_err(Into::into)),
                    Err(e) => Box::pin(async move { Err(e) }),
                };
                fut
            })
            .collect::<FuturesOrdered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(move |response| match response {
                Ok(response) => {
                    let (parts, body) = response.into_parts();
                    let headers = self.push_resource(WasmOwnedOrLease::Owned(parts.headers))?;
                    Ok(Ok(HttpResponse {
                        status: parts.status.as_u16(),
                        headers,
                        body: body.to_vec(),
                    }))
                }
                Err(e) => Ok(Err(e)),
            })
            .collect::<Result<Vec<_>, _>>()
    }

    async fn drop(&mut self, _: Resource<HttpClient>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}

fn convert_http_request(
    state: &mut WasiState,
    request: HttpRequest,
) -> wasmtime::Result<Result<(reqwest::Client, reqwest::Request), HttpError>> {
    let HttpRequest {
        method,
        url,
        headers,
        body,
        timeout_ms,
    } = request;

    let headers = state
        .table
        .delete(headers)?
        .into_inner()
        .expect("The guest doesn't receive a shared header, nor should it be able to create one.");

    let mut req = state
        .http_client()
        .request(method.into(), url)
        .headers(headers)
        .body(body);

    if let Some(timeout_ms) = timeout_ms {
        req = req.timeout(Duration::from_millis(timeout_ms));
    }

    Ok(match req.build_split() {
        (client, Ok(req)) => Ok((client, req)),
        (_, Err(e)) => Err(HttpError::Request(e.to_string())),
    })
}

impl From<HttpMethod> for reqwest::Method {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
            HttpMethod::Connect => reqwest::Method::CONNECT,
            HttpMethod::Trace => reqwest::Method::TRACE,
        }
    }
}

impl From<http::Method> for HttpMethod {
    fn from(value: http::Method) -> Self {
        match value {
            http::Method::GET => HttpMethod::Get,
            http::Method::POST => HttpMethod::Post,
            http::Method::PUT => HttpMethod::Put,
            http::Method::DELETE => HttpMethod::Delete,
            http::Method::PATCH => HttpMethod::Patch,
            http::Method::HEAD => HttpMethod::Head,
            http::Method::OPTIONS => HttpMethod::Options,
            http::Method::CONNECT => HttpMethod::Connect,
            http::Method::TRACE => HttpMethod::Trace,
            method => todo!("unsupported http method: {method:?}"),
        }
    }
}

impl From<reqwest::Version> for HttpVersion {
    fn from(value: reqwest::Version) -> Self {
        match value {
            reqwest::Version::HTTP_09 => HttpVersion::Http09,
            reqwest::Version::HTTP_10 => HttpVersion::Http10,
            reqwest::Version::HTTP_11 => HttpVersion::Http11,
            reqwest::Version::HTTP_2 => HttpVersion::Http20,
            reqwest::Version::HTTP_3 => HttpVersion::Http30,
            version => todo!("unsupported http version: {version:?}"),
        }
    }
}

impl AsRef<str> for HttpMethod {
    fn as_ref(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Trace => "TRACE",
        }
    }
}
