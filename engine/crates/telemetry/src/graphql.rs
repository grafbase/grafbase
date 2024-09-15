use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct GraphqlExecutionTelemetry<ErrorCode> {
    pub operations: Vec<(OperationType, OperationName)>,
    pub errors_count_by_code: Vec<(ErrorCode, u16)>,
}

impl<E> GraphqlExecutionTelemetry<E> {
    pub fn errors_count(&self) -> u64 {
        self.errors_count_by_code.iter().map(|(_, count)| *count as u64).sum()
    }
}

impl<ErrorCode> Default for GraphqlExecutionTelemetry<ErrorCode> {
    fn default() -> Self {
        Self {
            operations: Vec::new(),
            errors_count_by_code: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl OperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
            Self::Subscription => "subscription",
        }
    }

    // for engine-v1
    pub fn is_mutation(&self) -> bool {
        matches!(self, Self::Mutation)
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum OperationName {
    Original(String),
    Computed(String),
    #[default]
    Unknown,
}

impl std::fmt::Display for OperationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationName::Original(name) => name.fmt(f),
            OperationName::Computed(name) => name.fmt(f),
            OperationName::Unknown => Ok(()),
        }
    }
}

impl OperationName {
    pub fn original(&self) -> Option<&str> {
        match self {
            OperationName::Original(name) => Some(name),
            OperationName::Computed(_) | OperationName::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphqlOperationAttributes {
    pub ty: OperationType,
    pub name: OperationName,
    pub sanitized_query: Arc<str>,
}

#[derive(Clone, Copy, Debug)]
pub enum GraphqlResponseStatus {
    Success,
    /// Error happened during the execution of the query
    FieldError {
        count: u64,
        data_is_null: bool,
    },
    /// Bad request, failed before the execution and `data` field isn't present.
    RequestError {
        count: u64,
    },
    RefusedRequest,
}

impl GraphqlResponseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "SUCCESS",
            Self::FieldError { data_is_null, .. } => {
                if data_is_null {
                    "FIELD_ERROR_NULL_DATA"
                } else {
                    "FIELD_ERROR"
                }
            }
            Self::RequestError { .. } => "REQUEST_ERROR",
            Self::RefusedRequest => "REFUSED_REQUEST",
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    pub fn is_request_error(&self) -> bool {
        matches!(self, Self::RequestError { .. })
    }

    // Used to generate a status for a streaming response in engine-v1
    pub fn union(self, other: Self) -> Self {
        match (self, other) {
            (Self::RefusedRequest, _) | (_, Self::RefusedRequest) => Self::RefusedRequest,
            (s @ Self::RequestError { .. }, _) => s,
            (_, s @ Self::RequestError { .. }) => s,
            (Self::Success, s @ Self::FieldError { .. }) => s,
            (s @ Self::FieldError { .. }, Self::Success) => s,
            (Self::FieldError { count, data_is_null }, Self::FieldError { count: extra_count, .. }) => {
                Self::FieldError {
                    count: count + extra_count,
                    data_is_null,
                }
            }
            (Self::Success, Self::Success) => Self::Success,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SubgraphResponseStatus {
    HookError,
    HttpError,
    InvalidGraphqlResponseError,
    WellFormedGraphqlResponse(GraphqlResponseStatus),
}

impl SubgraphResponseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SubgraphResponseStatus::HookError => "HOOK_ERROR",
            SubgraphResponseStatus::HttpError => "HTTP_ERROR",
            SubgraphResponseStatus::InvalidGraphqlResponseError => "INVALID_RESPONSE",
            SubgraphResponseStatus::WellFormedGraphqlResponse(response) => response.as_str(),
        }
    }
}
