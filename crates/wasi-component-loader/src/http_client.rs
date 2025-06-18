use crate::extension::api::wit::HttpError;
use bytes::Bytes;
use grafbase_telemetry::otel::opentelemetry::{KeyValue, metrics::Histogram};
use http_body_util::BodyExt;
use std::time::Instant;
use tracing::{Instrument, field::Empty, info_span};

pub(crate) async fn send_request(
    (client, request): (reqwest::Client, reqwest::Request),
    request_durations: Histogram<u64>,
) -> Result<http::Response<Bytes>, HttpError> {
    let start = Instant::now();

    let mut attributes = request_attributes(&request);

    let span = info_span!(
        "hook-http-request",
        "http.request.body.size" = request
            .body()
            .and_then(|b| b.as_bytes())
            .map(|b| b.len())
            .unwrap_or_default(),
        "http.request.method" = request.method().as_ref(),
        "http.response.body.size" = Empty,
        "http.response.status_code" = Empty,
        "otel.name" = format!("{} {}", request.method().as_ref(), request.url().path()),
        "server.address" = request.url().host_str(),
        "server.port" = request.url().port(),
        "url.path" = request.url().path(),
        "otel.status_code" = Empty,
        "error.message" = Empty,
    );

    let result = client.execute(request).instrument(span.clone()).await;
    let duration = start.elapsed().as_millis() as u64;

    merge_response_attributes(&mut attributes, &result);
    request_durations.record(duration, &attributes);

    match result {
        Ok(response) => {
            let response: http::Response<reqwest::Body> = response.into();
            let (parts, body) = response.into_parts();
            let body = BodyExt::collect(body)
                .await
                .map(|buf| buf.to_bytes())
                .map_err(|err| HttpError::Connect(format!("Failed to receive body {err}")))?;

            span.record("http.response.status_code", parts.status.as_u16());
            span.record("http.response.body.size", body.len());
            Ok(http::Response::from_parts(parts, body))
        }
        Err(error) => {
            let error_message = error.to_string();

            span.record("otel.status_code", "Error");
            span.record("error.message", &error_message);

            Err(HttpError::Connect(error_message))
        }
    }
}

fn request_attributes(request: &reqwest::Request) -> Vec<KeyValue> {
    let mut attributes = Vec::new();

    attributes.push(KeyValue::new(
        "http.request.method",
        request.method().as_ref().to_string(),
    ));

    attributes.push(KeyValue::new("http.route", request.url().path().to_string()));

    if let Some(host) = request.url().host() {
        attributes.push(KeyValue::new("server.address", host.to_string()));
    }

    if let Some(port) = request.url().port() {
        attributes.push(KeyValue::new("server.port", port.to_string()));
    }

    attributes.push(KeyValue::new("url.scheme", request.url().scheme().to_string()));

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
