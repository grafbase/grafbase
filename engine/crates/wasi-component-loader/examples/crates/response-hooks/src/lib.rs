use bindings::component::grafbase::types::{
    CacheStatus, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, SharedContext,
    SubgraphRequestExecutionKind,
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
    InternalServerError,
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
            subgraph_name,
            method,
            url,
            responses,
            total_duration: total_duration_ms,
            has_errors,
            cached: matches!(cache_status, CacheStatus::Hit),
        };

        serde_json::to_vec(&info).unwrap()
    }

    fn on_operation_response(_: SharedContext, operation: ExecutedOperation) -> Vec<u8> {
        let info = OperationInfo {
            name: operation.name,
            document: operation.document,
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
