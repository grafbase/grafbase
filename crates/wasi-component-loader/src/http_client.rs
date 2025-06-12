use crate::extension::api::wit::{HttpError, HttpRequest, HttpResponse};
use grafbase_telemetry::otel::opentelemetry::{KeyValue, metrics::Histogram};
use http::{HeaderName, HeaderValue};
use std::{
    str::FromStr,
    time::{Duration, Instant},
};
use tracing::{Instrument, field::Empty, info_span};

pub(crate) async fn send_request(
    http_client: reqwest::Client,
    request_durations: Histogram<u64>,
    request: HttpRequest,
) -> Result<HttpResponse, HttpError> {
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

        request_durations.record(duration, &attributes);

        return Err(HttpError::Request(message));
    };

    span.record("server.address", url.host_str());
    span.record("server.port", url.port());
    span.record("url.path", url.path());
    span.record("otel.name", format!("{} {}", method.as_ref(), url.path()));

    let mut builder = http_client.request(method.into(), url);

    for (key, value) in headers {
        let Ok(key) = HeaderName::from_str(&key) else {
            let duration = start.elapsed().as_millis() as u64;
            let message = format!("invalid header key: {key}");

            span.record("otel.status_code", "Error");
            span.record("error.message", &message);

            attributes.push(KeyValue::new("otel.status_code", "Error"));

            request_durations.record(duration, &attributes);

            return Err(HttpError::Request(message));
        };

        let Ok(value) = HeaderValue::from_str(&value) else {
            let duration = start.elapsed().as_millis() as u64;
            let message = format!("invalid header value: {value}");

            span.record("otel.status_code", "Error");
            span.record("error.message", &message);

            attributes.push(KeyValue::new("otel.status_code", "Error"));

            request_durations.record(duration, &attributes);

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
    request_durations.record(duration, &attributes);

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
        Ok(response) => {
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
