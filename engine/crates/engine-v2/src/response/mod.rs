use std::sync::Arc;

use enumset::EnumSet;
pub(crate) use error::*;
use grafbase_telemetry::graphql::{GraphqlExecutionTelemetry, GraphqlOperationAttributes, GraphqlResponseStatus};
pub(crate) use key::*;
pub(crate) use object_set::*;
pub(crate) use path::*;
pub(crate) use read::*;
use schema::Schema;
pub(crate) use shape::*;
pub(crate) use value::*;
pub(crate) use write::*;

use crate::operation::PreparedOperation;

pub(crate) mod error;
mod key;
mod object_set;
mod path;
mod read;
mod shape;
mod value;
mod write;

pub(crate) enum Response {
    /// Before or while validating we have a well-formed GraphQL-over-HTTP request, we may
    /// reject the request with GraphQL errors.
    /// HTTP status code MUST NOT be 2xx according to the GraphQL-over-HTTP spec
    RefusedRequest(RefusedRequestResponse),
    /// We did receive a well-formed GraphQL-over-HTTP request, but preparation failed:
    /// unknown field, wrong arguments, etc.
    /// It's a "request error" as defined in the spec (no `data` field)
    /// HTTP status code MUST be 4xx or 5xx according to the GraphQL-over-HTTP spec for application/graphql-response+json
    RequestError(RequestErrorResponse),
    /// We have a well-formed GraphQL-over-HTTP request, and preparation succeeded.
    /// So `data` is present, even if null. That's considered to be a "partial response" and
    /// HTTP status code SHOULD be 2xx according to the GraphQL-over-HTTP spec for application/graphql-response+json
    Executed(ExecutedResponse),
}

pub(crate) struct ExecutedResponse {
    operation: Arc<PreparedOperation>,
    data: Option<ResponseData>,
    errors: Vec<GraphqlError>,
    on_operation_response_output: Option<Vec<u8>>,
}

impl ExecutedResponse {
    pub(crate) fn is_data_null(&self) -> bool {
        self.data.is_none()
    }

    pub(crate) fn graphql_status(&self) -> GraphqlResponseStatus {
        if self.errors.is_empty() {
            GraphqlResponseStatus::Success
        } else {
            GraphqlResponseStatus::FieldError {
                count: self.errors.len() as u64,
                data_is_null: self.is_data_null(),
            }
        }
    }
}

struct ResponseData {
    schema: Arc<Schema>,
    root: ResponseObjectId,
    parts: Vec<ResponseDataPart>,
}

pub(crate) struct RequestErrorResponse {
    operation_attributes: Option<GraphqlOperationAttributes>,
    errors: Vec<GraphqlError>,
}

impl From<RequestErrorResponse> for Response {
    fn from(resp: RequestErrorResponse) -> Self {
        Self::RequestError(resp)
    }
}

pub(crate) struct RefusedRequestResponse {
    status: http::StatusCode,
    error: GraphqlError,
}

impl RefusedRequestResponse {
    pub(crate) fn status(&self) -> http::StatusCode {
        self.status
    }
}

impl Response {
    pub(crate) fn refuse_request_with(status: http::StatusCode, error: impl Into<GraphqlError>) -> Self {
        Self::RefusedRequest(RefusedRequestResponse {
            status,
            error: error.into(),
        })
    }

    pub(crate) fn request_error<E>(
        operation_attributes: Option<GraphqlOperationAttributes>,
        errors: impl IntoIterator<Item = E>,
    ) -> Self
    where
        E: Into<GraphqlError>,
    {
        Self::RequestError(RequestErrorResponse {
            operation_attributes,
            errors: errors.into_iter().map(Into::into).collect(),
        })
    }

    pub(crate) fn execution_error(
        operation: Arc<PreparedOperation>,
        on_operation_response_output: Option<Vec<u8>>,
        errors: impl IntoIterator<Item: Into<GraphqlError>>,
    ) -> Self {
        Self::Executed(ExecutedResponse {
            operation,
            data: None,
            on_operation_response_output,
            errors: errors.into_iter().map(Into::into).collect(),
        })
    }

    pub(crate) fn take_on_operation_response_output(&mut self) -> Option<Vec<u8>> {
        match self {
            Self::Executed(resp) => std::mem::take(&mut resp.on_operation_response_output),
            _ => None,
        }
    }

    pub(crate) fn execution_telemetry(&self) -> GraphqlExecutionTelemetry<ErrorCode> {
        GraphqlExecutionTelemetry {
            errors_count: self.errors().len() as u64,
            distinct_error_codes: self.distinct_error_codes().into_iter().collect(),
            operations: self
                .operation_attributes()
                .into_iter()
                .map(|attributes| (attributes.ty, attributes.name.clone()))
                .collect(),
        }
    }

    pub(crate) fn operation_attributes(&self) -> Option<&GraphqlOperationAttributes> {
        match self {
            Self::RefusedRequest(_) => None,
            Self::RequestError(resp) => resp.operation_attributes.as_ref(),
            Self::Executed(resp) => Some(&resp.operation.attributes),
        }
    }

    pub(crate) fn graphql_status(&self) -> GraphqlResponseStatus {
        match self {
            Self::Executed(resp) => resp.graphql_status(),
            Self::RequestError(resp) => GraphqlResponseStatus::RequestError {
                count: resp.errors.len() as u64,
            },
            Self::RefusedRequest(_) => GraphqlResponseStatus::RefusedRequest,
        }
    }

    pub(crate) fn errors(&self) -> &[GraphqlError] {
        match self {
            Response::RefusedRequest(resp) => std::array::from_ref(&resp.error),
            Response::RequestError(resp) => &resp.errors,
            Response::Executed(resp) => &resp.errors,
        }
    }

    pub(crate) fn distinct_error_codes(&self) -> EnumSet<ErrorCode> {
        self.errors()
            .iter()
            .fold(EnumSet::<ErrorCode>::empty(), |mut set, error| {
                set |= error.code;
                set
            })
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response").finish_non_exhaustive()
    }
}
