use base64::Engine;
use grafbase_sdk::{
    HooksExtension,
    host_io::event_queue::{CacheStatus, Event, EventQueue, GraphqlResponseStatus, RequestExecution},
    host_io::http::{HeaderValue, Method, StatusCode},
    types::{Configuration, Error, ErrorResponse, GatewayHeaders},
};
use serde_json::json;

#[derive(HooksExtension)]
struct Hooks {
    config: TestConfig,
}

#[derive(serde::Deserialize)]
struct TestConfig {
    incoming_header: Option<HeaderTest>,
    outgoing_header: Option<HeaderTest>,
    events_header_name: Option<String>,
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

    fn on_request(&mut self, _: &str, _: Method, headers: &mut GatewayHeaders) -> Result<(), ErrorResponse> {
        if let Some(ref header_test) = self.config.incoming_header {
            headers.append(
                header_test.key.as_str(),
                HeaderValue::from_str(&header_test.value).unwrap(),
            );
        }

        Ok(())
    }

    fn on_response(&mut self, _: StatusCode, headers: &mut GatewayHeaders, queue: EventQueue) -> Result<(), String> {
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
                            "name": op.name(),
                            "document": op.document(),
                            "prepare_duration_ms": op.prepare_duration().as_millis(),
                            "duration_ms": op.duration().as_millis(),
                            "cached_plan": op.cached_plan(),
                            "status": match op.status() {
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
                            "subgraph_name": subgraph.subgraph_name(),
                            "method": subgraph.method().as_str(),
                            "url": subgraph.url(),
                            "cache_status": match subgraph.cache_status() {
                                CacheStatus::Hit => "hit",
                                CacheStatus::PartialHit => "partial_hit",
                                CacheStatus::Miss => "miss",
                            },
                            "total_duration_ms": subgraph.total_duration().as_millis(),
                            "has_errors": subgraph.has_errors(),
                            "executions": subgraph.executions().map(|exec| {
                                match exec {
                                    RequestExecution::InternalServerError => json!({"type": "internal_server_error"}),
                                    RequestExecution::RequestError => json!({"type": "request_error"}),
                                    RequestExecution::RateLimited => json!({"type": "rate_limited"}),
                                    RequestExecution::Response(resp) => {
                                        json!({
                                            "type": "response",
                                            "connection_time_ms": resp.connection_time().as_millis(),
                                            "response_time_ms": resp.response_time().as_millis(),
                                            "status_code": resp.status().as_u16()
                                        })
                                    }
                                }
                            }).collect::<Vec<_>>()
                        })
                    }
                    Event::Http(http) => {
                        json!({
                            "type": "http",
                            "method": http.method().as_str(),
                            "url": http.url(),
                            "status_code": http.response_status().as_u16()
                        })
                    }
                    Event::Extension(ext) => {
                        json!({
                            "type": "extension",
                            "event_name": ext.event_name(),
                            "extension_name": ext.extension_name()
                        })
                    }
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
}
