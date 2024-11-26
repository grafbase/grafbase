use crate::{
    names::{HTTP_CLIENT_EXECUTE_FUNCTION, HTTP_CLIENT_EXECUTE_MANY_FUNCTION, HTTP_CLIENT_RESOURCE},
    state::WasiState,
};
use futures::FutureExt;
use grafbase_telemetry::{
    metrics::meter_from_global_provider,
    otel::opentelemetry::{metrics::Histogram, KeyValue},
};
use http::{HeaderName, HeaderValue};
use std::{
    future::Future,
    str::FromStr,
    sync::LazyLock,
    time::{Duration, Instant},
};
use tracing::{field::Empty, info_span, Instrument};
use wasmtime::{
    component::{ComponentType, Lift, LinkerInstance, Lower, ResourceType},
    StoreContextMut,
};

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

static REQUEST_METRICS: LazyLock<Histogram<u64>> = LazyLock::new(|| {
    let meter = meter_from_global_provider();
    meter.u64_histogram("grafbase.hook.http_request.duration").init()
});

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
    Box::new(async move { Ok((send_request(request).await,)) })
}

fn execute_many(_: StoreContextMut<'_, WasiState>, (requests,): (Vec<HttpRequest>,)) -> HttpManyResult<'_> {
    Box::new(async move {
        let futures = requests
            .into_iter()
            .map(|request| send_request(request).boxed())
            .collect::<Vec<_>>();

        let responses = futures::future::join_all(futures).await;

        Ok((responses,))
    })
}

async fn send_request(request: HttpRequest) -> Result<HttpResponse, HttpError> {
    let start = Instant::now();

    let mut attributes = request_attributes(&request);

    let HttpRequest {
        method,
        url,
        headers,
        body,
        timeout_ms,
    } = request;

    let span = info_span!(
        "hook-http-request",
        "http.request.body.size" = body.len(),
        "http.request.method" = method.as_ref(),
        "http.response.body.size" = Empty,
        "http.response.status_code" = Empty,
        "otel.name" = Empty,
        "server.address" = Empty,
        "server.port" = Empty,
        "url.path" = Empty,
        "otel.status_code" = Empty,
        "error.message" = Empty,
    );

    let Ok(url) = reqwest::Url::parse(&url) else {
        let duration = start.elapsed().as_millis() as u64;
        let message = format!("invalid url: {url}");

        span.record("otel.status_code", "Error");
        span.record("error.message", &message);

        attributes.push(KeyValue::new("otel.status_code", "Error"));

        REQUEST_METRICS.record(duration, &attributes);

        return Err(HttpError::Request(message));
    };

    span.record("server.address", url.host_str());
    span.record("server.port", url.port());
    span.record("url.path", url.path());
    span.record("otel.name", format!("{} {}", method.as_ref(), url.path()));

    let mut builder = HTTP_CLIENT.request(method.into(), url);

    for (key, value) in headers {
        let Ok(key) = HeaderName::from_str(&key) else {
            let duration = start.elapsed().as_millis() as u64;
            let message = format!("invalid header key: {key}");

            span.record("otel.status_code", "Error");
            span.record("error.message", &message);

            attributes.push(KeyValue::new("otel.status_code", "Error"));

            REQUEST_METRICS.record(duration, &attributes);

            return Err(HttpError::Request(message));
        };

        let Ok(value) = HeaderValue::from_str(&value) else {
            let duration = start.elapsed().as_millis() as u64;
            let message = format!("invalid header value: {value}");

            span.record("otel.status_code", "Error");
            span.record("error.message", &message);

            attributes.push(KeyValue::new("otel.status_code", "Error"));

            REQUEST_METRICS.record(duration, &attributes);

            return Err(HttpError::Request(message));
        };

        builder = builder.header(key, value);
    }

    builder = builder.body(body);

    if let Some(timeout_ms) = timeout_ms {
        builder = builder.timeout(Duration::from_millis(timeout_ms));
    }

    let result = builder.send().instrument(span.clone()).await;
    let duration = start.elapsed().as_millis() as u64;

    merge_response_attributes(&mut attributes, &result);
    REQUEST_METRICS.record(duration, &attributes);

    match result {
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

            span.record("http.response.status_code", status);

            match response.bytes().await.map(|b| b.to_vec()) {
                Ok(body) => {
                    span.record("http.response.body.size", body.len());

                    Ok(HttpResponse {
                        status,
                        version,
                        headers,
                        body,
                    })
                }
                Err(error) => {
                    let error_message = error.to_string();

                    span.record("otel.status_code", "Error");
                    span.record("error.message", &error_message);

                    Err(HttpError::Connect(error_message))
                }
            }
        }
        Err(error) => {
            let error_message = error.to_string();

            span.record("otel.status_code", "Error");
            span.record("error.message", &error_message);

            Err(HttpError::Connect(error_message))
        }
    }
}

fn request_attributes(request: &HttpRequest) -> Vec<KeyValue> {
    let mut attributes = Vec::new();

    let HttpRequest { method, url, .. } = request;

    attributes.push(KeyValue::new("http.request.method", method.as_ref().to_string()));

    if let Ok(url) = reqwest::Url::parse(url) {
        attributes.push(KeyValue::new("http.route", url.path().to_string()));

        if let Some(host) = url.host() {
            attributes.push(KeyValue::new("server.address", host.to_string()));
        }

        if let Some(port) = url.port() {
            attributes.push(KeyValue::new("server.port", port.to_string()));
        }

        attributes.push(KeyValue::new("url.scheme", url.scheme().to_string()));
    }

    attributes
}

fn merge_response_attributes(attributes: &mut Vec<KeyValue>, result: &Result<reqwest::Response, reqwest::Error>) {
    match result {
        Ok(ref response) => {
            attributes.push(KeyValue::new(
                "http.response.status_code",
                response.status().as_u16().to_string(),
            ));
        }
        Err(_) => {
            attributes.push(KeyValue::new("otel.status_code", "Error"));
        }
    }
}
