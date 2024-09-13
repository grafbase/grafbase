use std::sync::Arc;

static X_GRAFBASE_GQL_RESPONSE_STATUS: http::HeaderName =
    http::HeaderName::from_static("x-grafbase-graphql-response-status");

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

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
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

    pub fn header_name() -> &'static http::HeaderName {
        &X_GRAFBASE_GQL_RESPONSE_STATUS
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
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

    pub fn encode(&self) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
        URL_SAFE_NO_PAD.encode(postcard::to_stdvec(self).expect("valid json"))
    }

    pub fn decode(bytes: &str) -> Option<Self> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
        let bytes = URL_SAFE_NO_PAD.decode(bytes).ok()?;
        postcard::from_bytes(&bytes).ok()
    }
}

impl headers::Header for GraphqlResponseStatus {
    fn name() -> &'static http::HeaderName {
        &X_GRAFBASE_GQL_RESPONSE_STATUS
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        values
            .next()
            .and_then(|value| value.to_str().ok())
            .and_then(GraphqlResponseStatus::decode)
            .ok_or_else(headers::Error::invalid)
    }

    fn encode<E: Extend<http::HeaderValue>>(&self, values: &mut E) {
        values.extend(Some(GraphqlResponseStatus::encode(self).try_into().unwrap()))
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
