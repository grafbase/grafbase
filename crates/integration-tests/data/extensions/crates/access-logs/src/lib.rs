use std::path::PathBuf;

use grafbase_sdk::{
    HooksExtension,
    host_io::{
        self,
        event_queue::{self, Event, EventQueue, GraphqlResponseStatus, OperationType, RequestExecution},
        http::{Method, StatusCode},
        logger::FileLogger,
    },
    types::{Configuration, Error, ErrorResponse, GatewayHeaders, HttpHeaders},
};

#[derive(HooksExtension)]
struct AccessLogs {
    logger: FileLogger,
}

#[derive(serde::Serialize, Debug)]
struct Custom {
    on_request: OnRequest,
    extension_name: String,
    event_name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct OnRequest {
    value: u64,
}

#[derive(serde::Deserialize, Debug)]
struct Config {
    path: PathBuf,
}

#[derive(serde::Serialize, Debug, Default)]
struct LogLine {
    operations: Vec<Operation>,
    subgraph_requests: Vec<SubgraphRequest>,
    http_requests: Vec<HttpRequest>,
    custom: Vec<Custom>,
}

#[derive(serde::Serialize, Debug)]
struct Operation {
    name: Option<String>,
    document: String,
    cached: bool,
    status: GraphqlResponseStatus,
    r#type: &'static str,
    complexity: Option<u64>,
}

#[derive(serde::Serialize, Debug)]
struct SubgraphRequest {
    subgraph_name: String,
    method: String,
    url: String,
    executions: Vec<Execution>,
    cache_status: &'static str,
    has_errors: bool,
}

#[derive(serde::Serialize, Debug)]
enum Execution {
    InternalServerError,
    RequestError,
    RateLimitExceeded,
    Response {
        status: u16,
        special_header_value: Option<String>,
    },
}

#[derive(serde::Serialize, Debug)]
struct HttpRequest {
    method: String,
    url: String,
    status: u16,
}

impl HooksExtension for AccessLogs {
    fn new(config: Configuration) -> Result<Self, Error> {
        let config: Config = config.deserialize()?;
        let logger = FileLogger::new(config.path, None)?;

        Ok(Self { logger })
    }

    fn on_request(&mut self, _: &str, _: Method, _: &mut GatewayHeaders) -> Result<(), ErrorResponse> {
        event_queue::send("on_request", OnRequest { value: 1 }).unwrap();

        Ok(())
    }

    fn on_response(&mut self, _: StatusCode, _: &mut GatewayHeaders, event_queue: EventQueue) -> Result<(), String> {
        let mut message = LogLine::default();

        while let Some(event) = event_queue.pop() {
            match event {
                Event::Operation(op) => {
                    message.operations.push(Operation {
                        name: op.name().map(ToString::to_string),
                        document: op.document().to_string(),
                        cached: op.cached_plan(),
                        status: op.status(),
                        r#type: match op.operation_type() {
                            OperationType::Query => "query",
                            OperationType::Mutation => "mutation",
                            OperationType::Subscription => "subscription",
                        },
                        complexity: op.complexity(),
                    });
                }
                Event::Subgraph(req) => message.subgraph_requests.push(SubgraphRequest {
                    subgraph_name: req.subgraph_name().to_string(),
                    method: req.method().to_string(),
                    url: {
                        let url = req.url().split("1:").next().unwrap();
                        format!("{}1:XXXXX/", url)
                    },
                    executions: req
                        .executions()
                        .map(|exec| match exec {
                            RequestExecution::InternalServerError => Execution::InternalServerError,
                            RequestExecution::RequestError => Execution::RequestError,
                            RequestExecution::RateLimited => Execution::RateLimitExceeded,
                            RequestExecution::Response(resp) => Execution::Response {
                                status: resp.status().as_u16(),
                                special_header_value: resp
                                    .response_headers()
                                    .get("X-Special")
                                    .map(|v| v.to_str().unwrap().to_string()),
                            },
                        })
                        .collect(),
                    cache_status: req.cache_status().as_str(),
                    has_errors: req.has_errors(),
                }),
                Event::Http(resp) => message.http_requests.push(HttpRequest {
                    method: resp.method().to_string(),
                    url: resp.url().to_string(),
                    status: resp.response_status().as_u16(),
                }),
                Event::Extension(extension_event) => {
                    let event = Custom {
                        on_request: extension_event.deserialize().unwrap(),
                        extension_name: extension_event.extension_name().to_string(),
                        event_name: extension_event.event_name().to_string(),
                    };

                    message.custom.push(event);
                }
            }
        }

        host_io::logger::log::info!(
            operations = message.operations.len(),
            subgraph_requests = message.subgraph_requests.len(),
            http_requests = message.http_requests.len(),
            custom_events = message.custom.len(),
            empty_field = Option::<usize>::None,
            optional_field = Some("foo"),
            random_string = "random_string_value";
            "on-response-hook"
        );

        self.logger.log_json(message).unwrap();

        Ok(())
    }
}
