use crate::{
    names::{HTTP_CLIENT_EXECUTE_FUNCTION, HTTP_CLIENT_EXECUTE_MANY_FUNCTION, HTTP_CLIENT_RESOURCE},
    state::WasiState,
};
use futures::FutureExt;
use http::{HeaderName, HeaderValue};
use reqwest::RequestBuilder;
use std::{future::Future, str::FromStr, sync::LazyLock, time::Duration};
use wasmtime::{
    component::{ComponentType, Lift, LinkerInstance, Lower, ResourceType},
    StoreContextMut,
};

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub(crate) fn map(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(HTTP_CLIENT_RESOURCE, ResourceType::host::<()>(), |_, _| Ok(()))?;

    types.func_wrap_async(HTTP_CLIENT_EXECUTE_FUNCTION, execute)?;
    types.func_wrap_async(HTTP_CLIENT_EXECUTE_MANY_FUNCTION, execute_many)?;

    Ok(())
}

#[derive(Clone, Lower, Lift, ComponentType)]
#[component(record)]
struct HttpRequest {
    #[component(name = "method")]
    method: HttpMethod,
    #[component(name = "url")]
    url: String,
    #[component(name = "headers")]
    headers: Vec<(String, String)>,
    #[component(name = "body")]
    body: Vec<u8>,
    #[component(name = "timeout-ms")]
    timeout_ms: Option<u64>,
}

#[derive(Clone, Copy, Lower, Lift, ComponentType)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)] // for some reason clippy thinks this is dead code, it's not.
enum HttpMethod {
    #[component(name = "get")]
    Get,
    #[component(name = "post")]
    Post,
    #[component(name = "put")]
    Put,
    #[component(name = "delete")]
    Delete,
    #[component(name = "patch")]
    Patch,
    #[component(name = "head")]
    Head,
    #[component(name = "options")]
    Options,
    #[component(name = "connect")]
    Connect,
    #[component(name = "trace")]
    Trace,
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

#[derive(Debug, Clone, Lower, Lift, ComponentType)]
#[component(record)]
struct HttpResponse {
    #[component(name = "status")]
    status: u16,
    #[component(name = "version")]
    version: HttpVersion,
    #[component(name = "headers")]
    headers: Vec<(String, String)>,
    #[component(name = "body")]
    body: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Lower, Lift, ComponentType)]
#[component(enum)]
#[repr(u8)]
enum HttpVersion {
    #[component(name = "http09")]
    Http09,
    #[component(name = "http10")]
    Http1,
    #[component(name = "http11")]
    Http11,
    #[component(name = "http20")]
    Http2,
    #[component(name = "http30")]
    Http3,
}

impl From<reqwest::Version> for HttpVersion {
    fn from(value: reqwest::Version) -> Self {
        match value {
            reqwest::Version::HTTP_09 => HttpVersion::Http09,
            reqwest::Version::HTTP_10 => HttpVersion::Http1,
            reqwest::Version::HTTP_11 => HttpVersion::Http11,
            reqwest::Version::HTTP_2 => HttpVersion::Http2,
            reqwest::Version::HTTP_3 => HttpVersion::Http3,
            version => todo!("unsupported http version: {version:?}"),
        }
    }
}

#[derive(Debug, Clone, Lower, Lift, ComponentType)]
#[component(variant)]
enum HttpError {
    #[component(name = "timeout")]
    Timeout,
    #[component(name = "request")]
    Request(String),
    #[component(name = "connect")]
    Connect(String),
}

type HttpResult<'a> = Box<dyn Future<Output = anyhow::Result<(Result<HttpResponse, HttpError>,)>> + Send + 'a>;
type HttpManyResult<'a> = Box<dyn Future<Output = anyhow::Result<(Vec<Result<HttpResponse, HttpError>>,)>> + Send + 'a>;

fn execute(_: StoreContextMut<'_, WasiState>, (request,): (HttpRequest,)) -> HttpResult<'_> {
    Box::new(async move {
        let HttpRequest {
            method,
            url,
            headers,
            body,
            timeout_ms,
        } = request;

        let mut builder = HTTP_CLIENT.request(method.into(), &url);

        for (key, value) in headers {
            let Ok(key) = HeaderName::from_str(&key) else {
                let message = format!("invalid header key: {key}");
                return Ok((Err(HttpError::Request(message)),));
            };

            let Ok(value) = HeaderValue::from_str(&value) else {
                let message = format!("invalid header value: {value}");
                return Ok((Err(HttpError::Request(message)),));
            };

            builder = builder.header(key, value);
        }

        builder = builder.body(body);

        if let Some(timeout_ms) = timeout_ms {
            builder = builder.timeout(Duration::from_millis(timeout_ms));
        }

        Ok((send_request(builder).await,))
    })
}

fn execute_many(_: StoreContextMut<'_, WasiState>, (requests,): (Vec<HttpRequest>,)) -> HttpManyResult<'_> {
    Box::new(async move {
        let mut futures = Vec::with_capacity(requests.len());

        for request in requests {
            let HttpRequest {
                method,
                url,
                headers,
                body,
                timeout_ms,
            } = request;

            let mut builder = HTTP_CLIENT.request(method.into(), &url);

            for (key, value) in headers {
                let Ok(key) = HeaderName::from_str(&key) else {
                    let message = format!("invalid header key: {key}");
                    futures.push(request_error(message).boxed());

                    continue;
                };

                let Ok(value) = HeaderValue::from_str(&value) else {
                    let message = format!("invalid header value: {value}");
                    futures.push(request_error(message).boxed());

                    continue;
                };

                builder = builder.header(key, value);
            }

            builder = builder.body(body);

            if let Some(timeout_ms) = timeout_ms {
                builder = builder.timeout(Duration::from_millis(timeout_ms));
            }

            futures.push(send_request(builder).boxed())
        }

        let responses = futures::future::join_all(futures).await;

        Ok((responses,))
    })
}

async fn send_request(builder: RequestBuilder) -> Result<HttpResponse, HttpError> {
    match builder.send().await {
        Ok(response) => {
            let headers = response
                .headers()
                .iter()
                .flat_map(|(key, value)| {
                    let key = key.as_str().to_string();
                    let value = value.to_str().map(ToString::to_string).ok()?;

                    Some((key, value))
                })
                .collect();

            let status = response.status().as_u16();
            let version = response.version().into();

            match response.bytes().await.map(|b| b.to_vec()) {
                Ok(body) => Ok(HttpResponse {
                    status,
                    version,
                    headers,
                    body,
                }),
                Err(error) => Err(HttpError::Connect(error.to_string())),
            }
        }
        Err(error) => Err(HttpError::Connect(error.to_string())),
    }
}

async fn request_error(message: String) -> Result<HttpResponse, HttpError> {
    Err(HttpError::Request(message))
}
