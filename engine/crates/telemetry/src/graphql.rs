use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct GraphqlExecutionTelemetry<ErrorCode> {
    pub operations: Vec<(OperationType, OperationName)>,
    pub errors_count: u64,
    pub distinct_error_codes: Vec<ErrorCode>,
}

impl<ErrorCode> Default for GraphqlExecutionTelemetry<ErrorCode> {
    fn default() -> Self {
        Self {
            operations: Vec::new(),
            errors_count: 0,
            distinct_error_codes: Vec::new(),
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
    /// Indicates a successful response.
    Success,
    /// An error occurred during the execution of the query.
    FieldError {
        /// The number of field errors encountered.
        count: u64,
        /// Indicates whether the data is null.
        data_is_null: bool,
    },
    /// Represents a bad request that failed before execution,
    /// and the `data` field isn't present.
    RequestError {
        /// The number of request errors encountered.
        count: u64,
    },
    /// Indicates that the request was refused.
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

    /// Returns `true` if the response status indicates success.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Checks if the response status indicates a request error.
    ///
    /// A request error occurs when there are issues with the request
    /// before it can be executed. This may include invalid input
    /// or misformatted requests.
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
    /// An error that occurred during `on-subgraph-request` hook execution.
    HookError,
    /// An error that occurred during the HTTP request.
    HttpError,
    /// Represents an invalid GraphQL response format.
    InvalidGraphqlResponseError,
    /// A well-formed GraphQL response containing a status.
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
