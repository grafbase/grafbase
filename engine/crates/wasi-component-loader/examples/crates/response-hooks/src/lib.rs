use bindings::component::grafbase::types::{
    CacheStatus, ExecutedHttpRequest, ExecutedOperationRequest, ExecutedSubgraphRequest, Operation, ResponseKind,
    SharedContext,
};
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
    SerializationError,
    HookError,
    RequestError,
    RateLimited,
    Responsed(ResponseInfo),
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SubgraphInfo {
    subgraph_name: String,
    method: String,
    url: String,
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
struct OperationInfo {
    name: Option<String>,
    document: String,
    prepare_duration: u64,
    cached: bool,
    duration: u64,
    status: GraphqlResponseStatus,
    subgraphs: Vec<SubgraphInfo>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuditInfo {
    method: String,
    url: String,
    status_code: u16,
    trace_id: String,
    operations: Vec<OperationInfo>,
}

impl Guest for Component {
    fn on_subgraph_response(_: SharedContext, request: ExecutedSubgraphRequest) -> Vec<u8> {
        let ExecutedSubgraphRequest {
            subgraph_name,
            method,
            url,
            responses,
            cache_status,
            total_duration,
            has_errors,
        } = request;

        let responses = responses
            .into_iter()
            .map(|r| match r {
                ResponseKind::Responded(info) => ResponseData::Responsed(ResponseInfo {
                    connection_time: info.connection_time,
                    response_time: info.response_time,
                    status_code: info.status_code,
                }),
                ResponseKind::SerializationError => ResponseData::SerializationError,
                ResponseKind::HookError => ResponseData::HookError,
                ResponseKind::RequestError => ResponseData::RequestError,
                ResponseKind::RateLimited => ResponseData::RateLimited,
            })
            .collect();

        let info = SubgraphInfo {
            subgraph_name,
            method,
            url,
            responses,
            total_duration,
            has_errors,
            cached: matches!(cache_status, CacheStatus::Hit),
        };

        serde_json::to_vec(&info).unwrap()
    }

    fn on_operation_response(_: SharedContext, operation: Operation, request: ExecutedOperationRequest) -> Vec<u8> {
        let info = OperationInfo {
            name: operation.name,
            document: operation.document,
            prepare_duration: operation.prepare_duration,
            cached: operation.cached,
            duration: request.duration,
            status: match request.status {
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
            subgraphs: request
                .on_subgraph_request_outputs
                .iter()
                .filter_map(|bytes| serde_json::from_slice(bytes).ok())
                .collect(),
        };

        serde_json::to_vec(&info).unwrap()
    }

    fn on_http_response(context: SharedContext, request: ExecutedHttpRequest) {
        let info = AuditInfo {
            method: request.method,
            url: request.url,
            status_code: request.status_code,
            trace_id: context.trace_id(),
            operations: request
                .on_operation_response_outputs
                .iter()
                .filter_map(|bytes| serde_json::from_slice(bytes).ok())
                .collect(),
        };

        context.log_access(&serde_json::to_vec(&info).unwrap()).unwrap();
    }
}

bindings::export!(Component with_types_in bindings);
