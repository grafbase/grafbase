mod data;
mod error;
mod extensions;
mod object_set;
mod path;
mod read;
mod value;
mod write;

use std::sync::Arc;

pub(crate) use data::*;
pub(crate) use error::*;
pub(crate) use extensions::*;
use grafbase_telemetry::graphql::{GraphqlExecutionTelemetry, GraphqlOperationAttributes, GraphqlResponseStatus};
pub(crate) use object_set::*;
pub(crate) use path::*;
pub(crate) use read::*;
use schema::Schema;
pub(crate) use value::*;
pub(crate) use write::*;

use crate::prepare::PreparedOperation;

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
    schema: Arc<Schema>,
    operation: Arc<PreparedOperation>,
    operation_attributes: GraphqlOperationAttributes,
    data: Option<ResponseData>,
    errors: ErrorParts,
    extensions: ResponseExtensions,
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

pub(crate) struct RequestErrorResponse {
    operation_attributes: Option<GraphqlOperationAttributes>,
    errors: Vec<GraphqlError>,
    error_code_counter: ErrorCodeCounter,
    extensions: ResponseExtensions,
}

pub(crate) struct RefusedRequestResponse {
    status: http::StatusCode,
    operation_attributes: Option<GraphqlOperationAttributes>,
    errors: Vec<GraphqlError>,
    error_code_counter: ErrorCodeCounter,
    extensions: ResponseExtensions,
    pub(crate) headers: http::HeaderMap,
}

impl RefusedRequestResponse {
    pub(crate) fn status(&self) -> http::StatusCode {
        self.status
    }
}

impl Response {
    pub(crate) fn refuse_request_with(
        status: http::StatusCode,
        errors: impl IntoIterator<Item = impl Into<GraphqlError>>,
        headers: http::HeaderMap,
    ) -> Self {
        let mut error_code_counter = ErrorCodeCounter::default();

        let errors = errors
            .into_iter()
            .map(|error| {
                let error: GraphqlError = error.into();
                error_code_counter.increment(error.code);
                error
            })
            .collect::<Vec<_>>();

        Self::RefusedRequest(RefusedRequestResponse {
            status,
            operation_attributes: None,
            errors,
            error_code_counter,
            extensions: Default::default(),
            headers,
        })
    }

    pub(crate) fn request_error<E>(errors: impl IntoIterator<Item = E>) -> Self
    where
        E: Into<GraphqlError>,
    {
        let errors = errors.into_iter().map(Into::into).collect::<Vec<_>>();
        let error_code_counter = ErrorCodeCounter::from_errors(&errors);

        Self::RequestError(RequestErrorResponse {
            operation_attributes: None,
            errors,
            error_code_counter,
            extensions: Default::default(),
        })
    }

    pub(crate) fn execution_error(
        schema: &Arc<Schema>,
        operation: &Arc<PreparedOperation>,
        errors: impl IntoIterator<Item: Into<GraphqlError>>,
    ) -> Self {
        let errors = ErrorParts::from_errors(errors);

        Self::Executed(ExecutedResponse {
            schema: schema.clone(),
            operation: operation.clone(),
            operation_attributes: operation.attributes(),
            data: None,
            errors,
            extensions: Default::default(),
        })
    }

    pub(crate) fn with_extensions(mut self, extensions: ResponseExtensions) -> Self {
        let ext = self.extensions_mut();
        *ext = std::mem::take(ext).merge(extensions);

        self
    }

    pub(crate) fn extensions_mut(&mut self) -> &mut ResponseExtensions {
        match self {
            Self::RefusedRequest(resp) => &mut resp.extensions,
            Self::RequestError(resp) => &mut resp.extensions,
            Self::Executed(resp) => &mut resp.extensions,
        }
    }

    pub(crate) fn execution_telemetry(&self) -> GraphqlExecutionTelemetry<ErrorCode> {
        GraphqlExecutionTelemetry {
            errors_count_by_code: self.error_code_counter().to_vec(),
            operations: self
                .operation_attributes()
                .into_iter()
                .map(|attributes| (attributes.ty, attributes.name.clone()))
                .collect(),
        }
    }

    pub(crate) fn operation_attributes(&self) -> Option<&GraphqlOperationAttributes> {
        match self {
            Self::RefusedRequest(resp) => resp.operation_attributes.as_ref(),
            Self::RequestError(resp) => resp.operation_attributes.as_ref(),
            Self::Executed(resp) => Some(&resp.operation_attributes),
        }
    }

    pub(crate) fn with_operation_attributes(mut self, operation_attributes: GraphqlOperationAttributes) -> Self {
        match &mut self {
            Self::RefusedRequest(resp) => {
                resp.operation_attributes = Some(operation_attributes);
            }
            Self::RequestError(resp) => {
                resp.operation_attributes = Some(operation_attributes);
            }
            Self::Executed(resp) => {
                resp.operation_attributes = operation_attributes;
            }
        }
        self
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

    pub(crate) fn pre_execution_errors(&self) -> &[GraphqlError] {
        match self {
            Response::RefusedRequest(resp) => &resp.errors,
            Response::RequestError(resp) => &resp.errors,
            Response::Executed(_) => unreachable!(),
        }
    }

    pub(crate) fn error_code_counter(&self) -> &ErrorCodeCounter {
        match self {
            Response::RefusedRequest(resp) => &resp.error_code_counter,
            Response::RequestError(resp) => &resp.error_code_counter,
            Response::Executed(resp) => resp.errors.code_counter(),
        }
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response").finish_non_exhaustive()
    }
}
