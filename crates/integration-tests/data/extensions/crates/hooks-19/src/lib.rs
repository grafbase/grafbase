use base64::Engine;
use grafbase_sdk::{
    HooksExtension,
    host_io::{
        event_queue::{CacheStatus, Event, EventQueue, GraphqlResponseStatus, RequestExecution},
        http::{HeaderValue, Method, StatusCode},
    },
    types::{Configuration, Error, ErrorResponse, Headers, HttpRequestParts, OnRequestOutput},
};
use serde_json::json;

#[derive(HooksExtension)]
struct Hooks {
    config: TestConfig,
}

#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
struct TestConfig {
    incoming_header: Option<HeaderTest>,
    outgoing_header: Option<HeaderTest>,
    events_header_name: Option<String>,
    on_subgraph_request: Option<OnSubgraphRequestConfig>,
}

#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
struct OnSubgraphRequestConfig {
    url: Option<String>,
    header_name: Option<String>,
    header_value: Option<String>,
}

#[derive(serde::Deserialize)]
struct HeaderTest {
    key: String,
    value: String,
}

impl HooksExtension for Hooks {
    fn new(config: Configuration) -> Result<Self, Error> {
        let config = config.deserialize::<TestConfig>()?;

        Ok(Self { config })
    }

    #[allow(refining_impl_trait)]
    fn on_request(&mut self, _: &str, _: Method, headers: &mut Headers) -> Result<OnRequestOutput, ErrorResponse> {
        if let Some(ref header_test) = self.config.incoming_header {
            headers.append(
                header_test.key.as_str(),
                HeaderValue::from_str(&header_test.value).unwrap(),
            );
        }

        let mut output = OnRequestOutput::new();
        if let Some(value) = headers.get("contract-key") {
            output = output.contract_key(value.to_str().unwrap().to_owned());
        }

        Ok(output)
    }

    fn on_response(&mut self, _: StatusCode, headers: &mut Headers, queue: EventQueue) -> Result<(), Error> {
        if let Some(ref header_test) = self.config.outgoing_header {
            headers.append(
                header_test.key.as_str(),
                HeaderValue::from_str(&header_test.value).unwrap(),
            );
        }

        if let Some(ref name) = self.config.events_header_name {
            let mut events_json = Vec::new();

            while let Some(event) = queue.pop() {
                let event_json = match event {
                    Event::Operation(op) => {
                        json!({
                            "type": "operation",
                            "name": op.name,
                            "document": op.document,
                            "prepare_duration_ms": op.prepare_duration.as_millis(),
                            "duration_ms": op.duration.as_millis(),
                            "cached_plan": op.cached_plan,
                            "status": match op.status {
                                GraphqlResponseStatus::Success => json!({"type": "success"}),
                                GraphqlResponseStatus::FieldError(ref err) => {
                                    json!({
                                        "type": "field_error",
                                        "count": err.count,
                                        "data_is_null": err.data_is_null
                                    })
                                },
                                GraphqlResponseStatus::RequestError(ref err) => {
                                    json!({
                                        "type": "request_error",
                                        "count": err.count
                                    })
                                },
                                GraphqlResponseStatus::RefusedRequest => json!({"type": "refused_request"}),
                            }
                        })
                    }
                    Event::Subgraph(subgraph) => {
                        json!({
                            "type": "subgraph",
                            "subgraph_name": subgraph.subgraph_name,
                            "method": subgraph.method.as_str(),
                            "url": subgraph.url,
                            "cache_status": match subgraph.cache_status {
                                CacheStatus::Hit => "hit",
                                CacheStatus::PartialHit => "partial_hit",
                                CacheStatus::Miss => "miss",
                            },
                            "total_duration_ms": subgraph.total_duration.as_millis(),
                            "has_errors": subgraph.has_errors,
                            "executions": subgraph.into_executions().map(|exec| {
                                match exec {
                                    RequestExecution::InternalServerError => json!({"type": "internal_server_error"}),
                                    RequestExecution::RequestError => json!({"type": "request_error"}),
                                    RequestExecution::RateLimited => json!({"type": "rate_limited"}),
                                    RequestExecution::Response(resp) => {
                                        json!({
                                            "type": "response",
                                            "connection_time_ms": resp.connection_time.as_millis(),
                                            "response_time_ms": resp.response_time.as_millis(),
                                            "status_code": resp.status_code.as_u16()
                                        })
                                    }
                                    _ => json!({"type": "Unknown event"})
                                }
                            }).collect::<Vec<_>>()
                        })
                    }
                    Event::Http(http) => {
                        json!({
                            "type": "http",
                            "method": http.method.as_str(),
                            "url": http.url,
                            "status_code": http.status_code.as_u16()
                        })
                    }
                    Event::Extension(ext) => {
                        json!({
                            "type": "extension",
                            "event_name": ext.event_name,
                            "extension_name": ext.extension_name
                        })
                    }
                    _ => json!({"type": "Unknown event"}),
                };

                events_json.push(event_json);
            }

            let events_json_string = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(serde_json::to_string(&events_json).unwrap_or_else(|_| "[]".to_string()));
            headers.append(
                name,
                HeaderValue::from_str(&events_json_string).unwrap_or_else(|_| HeaderValue::from_static("[]")),
            );
        }

        Ok(())
    }

    fn on_subgraph_request(&mut self, parts: &mut HttpRequestParts) -> Result<(), Error> {
        let Some(ref config) = self.config.on_subgraph_request else {
            return Ok(());
        };
        if let Some(ref url) = config.url {
            parts.url = url.clone();
        }
        if let Some((name, value)) = config.header_name.as_ref().zip(config.header_value.as_ref()) {
            parts.headers.append(name, value);
        }

        Ok(())
    }
}
