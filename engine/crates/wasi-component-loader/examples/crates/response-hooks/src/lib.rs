use bindings::component::grafbase::types::{
    CacheStatus, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, SharedContext,
    SubgraphRequestExecutionKind,
};
use bindings::exports::component::grafbase::gateway_request;
use bindings::exports::component::grafbase::responses::Guest;

#[allow(warnings)]
mod bindings;

struct Component;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ResponseInfo {
    pub connection_time: u64,
    pub response_time: u64,
    pub status_code: u16,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ResponseData {
    InternalServerError,
    HookError,
    RequestError,
    RateLimited,
    Responsed(ResponseInfo),
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SubgraphInfo<'a> {
    subgraph_name: &'a str,
    method: &'a str,
    url: &'a str,
    responses: Vec<ResponseData>,
    total_duration: u64,
    has_errors: bool,
    cached: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FieldError {
    count: u64,
    data_is_null: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RequestError {
    count: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum GraphqlResponseStatus {
    Success,
    FieldError(FieldError),
    RequestError(RequestError),
    RefusedRequest,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct OperationInfo<'a> {
    name: Option<&'a str>,
    document: &'a str,
    prepare_duration: u64,
    cached: bool,
    duration: u64,
    status: GraphqlResponseStatus,
    subgraphs: Vec<SubgraphInfo<'a>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuditInfo<'a> {
    method: &'a str,
    url: &'a str,
    status_code: u16,
    trace_id: &'a str,
    operations: Vec<OperationInfo<'a>>,
}

impl gateway_request::Guest for Component {
    fn on_gateway_request(
        _: gateway_request::Context,
        headers: gateway_request::Headers,
    ) -> Result<(), gateway_request::Error> {
        if headers.get("test-value").is_some() {
            Err(gateway_request::Error {
                extensions: Vec::new(),
                message: String::from("test-value header is not allowed"),
            })
        } else {
            Ok(())
        }
    }
}

impl Guest for Component {
    fn on_subgraph_response(_: SharedContext, request: ExecutedSubgraphRequest) -> Vec<u8> {
        let ExecutedSubgraphRequest {
            subgraph_name,
            method,
            url,
            executions,
            cache_status,
            total_duration_ms,
            has_errors,
        } = request;

        let responses = executions
            .into_iter()
            .map(|r| match r {
                SubgraphRequestExecutionKind::Response(info) => ResponseData::Responsed(ResponseInfo {
                    connection_time: info.connection_time_ms,
                    response_time: info.response_time_ms,
                    status_code: info.status_code,
                }),
                SubgraphRequestExecutionKind::InternalServerError => ResponseData::InternalServerError,
                SubgraphRequestExecutionKind::HookError => ResponseData::HookError,
                SubgraphRequestExecutionKind::RequestError => ResponseData::RequestError,
                SubgraphRequestExecutionKind::RateLimited => ResponseData::RateLimited,
            })
            .collect();

        let info = SubgraphInfo {
            subgraph_name: &subgraph_name,
            method: &method,
            url: &url,
            responses,
            total_duration: total_duration_ms,
            has_errors,
            cached: matches!(cache_status, CacheStatus::Hit),
        };

        postcard::to_stdvec(&info).unwrap()
    }

    fn on_operation_response(_: SharedContext, operation: ExecutedOperation) -> Vec<u8> {
        let info = OperationInfo {
            name: operation.name.as_deref(),
            document: &operation.document,
            prepare_duration: operation.prepare_duration_ms,
            cached: operation.cached_plan,
            duration: operation.duration_ms,
            status: match operation.status {
                bindings::component::grafbase::types::GraphqlResponseStatus::Success => GraphqlResponseStatus::Success,
                bindings::component::grafbase::types::GraphqlResponseStatus::FieldError(e) => {
                    GraphqlResponseStatus::FieldError(FieldError {
                        count: e.count,
                        data_is_null: e.data_is_null,
                    })
                }
                bindings::component::grafbase::types::GraphqlResponseStatus::RequestError(e) => {
                    GraphqlResponseStatus::RequestError(RequestError { count: e.count })
                }
                bindings::component::grafbase::types::GraphqlResponseStatus::RefusedRequest => {
                    GraphqlResponseStatus::RefusedRequest
                }
            },
            subgraphs: operation
                .on_subgraph_response_outputs
                .iter()
                .filter_map(|bytes| postcard::from_bytes(bytes).ok())
                .collect(),
        };

        postcard::to_stdvec(&info).unwrap()
    }

    fn on_http_response(context: SharedContext, request: ExecutedHttpRequest) {
        let info = AuditInfo {
            method: &request.method,
            url: &request.url,
            status_code: request.status_code,
            trace_id: &context.trace_id(),
            operations: request
                .on_operation_response_outputs
                .iter()
                .filter_map(|bytes| postcard::from_bytes(bytes).ok())
                .collect(),
        };

        context.log_access(&serde_json::to_vec(&info).unwrap()).unwrap();
    }
}

bindings::export!(Component with_types_in bindings);
