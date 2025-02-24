use grafbase_hooks::{
    CacheStatus, Context, Error, ErrorResponse, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest,
    Headers, Hooks, SharedContext, SubgraphRequestExecutionKind, grafbase_hooks, host_io::access_log,
};

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

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn on_gateway_request(&mut self, _: Context, _: String, headers: Headers) -> Result<(), ErrorResponse> {
        if headers.get("test-value").is_some() {
            let error = Error {
                extensions: Vec::new(),
                message: String::from("test-value header is not allowed"),
            };

            Err(ErrorResponse {
                status_code: 400,
                errors: vec![error],
            })
        } else {
            Ok(())
        }
    }

    fn on_subgraph_response(&mut self, _: SharedContext, request: ExecutedSubgraphRequest) -> Vec<u8> {
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

    fn on_operation_response(&mut self, _: SharedContext, operation: ExecutedOperation) -> Vec<u8> {
        let info = OperationInfo {
            name: operation.name.as_deref(),
            document: &operation.document,
            prepare_duration: operation.prepare_duration_ms,
            cached: operation.cached_plan,
            duration: operation.duration_ms,
            status: match operation.status {
                grafbase_hooks::GraphqlResponseStatus::Success => GraphqlResponseStatus::Success,
                grafbase_hooks::GraphqlResponseStatus::FieldError(e) => GraphqlResponseStatus::FieldError(FieldError {
                    count: e.count,
                    data_is_null: e.data_is_null,
                }),
                grafbase_hooks::GraphqlResponseStatus::RequestError(e) => {
                    GraphqlResponseStatus::RequestError(RequestError { count: e.count })
                }
                grafbase_hooks::GraphqlResponseStatus::RefusedRequest => GraphqlResponseStatus::RefusedRequest,
            },
            subgraphs: operation
                .on_subgraph_response_outputs
                .iter()
                .filter_map(|bytes| postcard::from_bytes(bytes).ok())
                .collect(),
        };

        postcard::to_stdvec(&info).unwrap()
    }

    fn on_http_response(&mut self, context: SharedContext, request: ExecutedHttpRequest) {
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

        access_log::send(&serde_json::to_vec(&info).unwrap()).unwrap();
    }
}

grafbase_hooks::register_hooks!(Component);
