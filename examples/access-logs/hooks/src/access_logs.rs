use grafbase_hooks::SubgraphRequestExecutionKind;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ResponseInfo {
    pub connection_time_ms: u64,
    pub response_time_ms: u64,
    pub status_code: u16,
}

impl From<grafbase_hooks::SubgraphResponse> for ResponseInfo {
    fn from(value: grafbase_hooks::SubgraphResponse) -> Self {
        let grafbase_hooks::SubgraphResponse {
            connection_time_ms,
            response_time_ms,
            status_code,
        } = value;

        Self {
            connection_time_ms,
            response_time_ms,
            status_code,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ResponseData {
    InternalServerError,
    HookError,
    RequestError,
    RateLimited,
    Response(ResponseInfo),
}

impl From<SubgraphRequestExecutionKind> for ResponseData {
    fn from(value: SubgraphRequestExecutionKind) -> Self {
        match value {
            SubgraphRequestExecutionKind::InternalServerError => Self::InternalServerError,
            SubgraphRequestExecutionKind::HookError => Self::HookError,
            SubgraphRequestExecutionKind::RequestError => Self::RequestError,
            SubgraphRequestExecutionKind::RateLimited => Self::RateLimited,
            SubgraphRequestExecutionKind::Response(info) => Self::Response(info.into()),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SubgraphInfo<'a> {
    pub subgraph_name: &'a str,
    pub method: &'a str,
    pub url: &'a str,
    pub responses: Vec<ResponseData>,
    pub total_duration_ms: u64,
    pub has_errors: bool,
    pub cached: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FieldError {
    pub count: u64,
    pub data_is_null: bool,
}

impl From<grafbase_hooks::FieldError> for FieldError {
    fn from(value: grafbase_hooks::FieldError) -> Self {
        Self {
            count: value.count,
            data_is_null: value.data_is_null,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RequestError {
    pub count: u64,
}

impl From<grafbase_hooks::RequestError> for RequestError {
    fn from(value: grafbase_hooks::RequestError) -> Self {
        Self { count: value.count }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum GraphqlResponseStatus {
    Success,
    FieldError(FieldError),
    RequestError(RequestError),
    RefusedRequest,
}

impl From<grafbase_hooks::GraphqlResponseStatus> for GraphqlResponseStatus {
    fn from(value: grafbase_hooks::GraphqlResponseStatus) -> Self {
        match value {
            grafbase_hooks::GraphqlResponseStatus::Success => Self::Success,
            grafbase_hooks::GraphqlResponseStatus::FieldError(error) => Self::FieldError(error.into()),
            grafbase_hooks::GraphqlResponseStatus::RequestError(error) => Self::RequestError(error.into()),
            grafbase_hooks::GraphqlResponseStatus::RefusedRequest => Self::RefusedRequest,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OperationInfo<'a> {
    pub name: Option<&'a str>,
    pub document: &'a str,
    pub prepare_duration_ms: u64,
    pub cached_plan: bool,
    pub duration_ms: u64,
    pub status: GraphqlResponseStatus,
    pub subgraphs: Vec<SubgraphInfo<'a>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuditInfo<'a> {
    pub method: &'a str,
    pub url: &'a str,
    pub status_code: u16,
    pub trace_id: &'a str,
    pub operations: Vec<OperationInfo<'a>>,
}
