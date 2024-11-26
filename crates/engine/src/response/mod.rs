mod extensions;
mod object_set;
mod path;
mod read;
mod shape;
mod value;
mod write;

use std::sync::Arc;

pub(crate) use error::*;
use extensions::ResponseExtensions;
pub(crate) use extensions::*;
use grafbase_telemetry::graphql::{GraphqlExecutionTelemetry, GraphqlOperationAttributes, GraphqlResponseStatus};
pub(crate) use key::*;
pub(crate) use object_set::*;
pub(crate) use path::*;
pub(crate) use read::*;
use schema::Schema;
pub(crate) use shape::*;
pub(crate) use value::*;
pub(crate) use write::*;

use crate::prepare::{CachedOperation, PreparedOperation};

pub(crate) mod error;
pub(crate) mod key;

pub(crate) enum Response<OnOperationResponseHookOutput> {
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
    Executed(ExecutedResponse<OnOperationResponseHookOutput>),
}

pub(crate) struct ExecutedResponse<OnOperationResponseHookOutput> {
    operation: Arc<CachedOperation>,
    operation_attributes: GraphqlOperationAttributes,
    data: Option<ResponseData>,
    errors: Vec<GraphqlError>,
    error_code_counter: ErrorCodeCounter,
    on_operation_response_output: Option<OnOperationResponseHookOutput>,
    extensions: Option<ResponseExtensions>,
}

impl<OnOperationResponseHookOutput> ExecutedResponse<OnOperationResponseHookOutput> {
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
    error_code_counter: ErrorCodeCounter,
    extensions: Option<ResponseExtensions>,
}

pub(crate) struct RefusedRequestResponse {
    status: http::StatusCode,
    errors: Vec<GraphqlError>,
    error_code_counter: ErrorCodeCounter,
    extensions: Option<ResponseExtensions>,
}

impl RefusedRequestResponse {
    pub(crate) fn status(&self) -> http::StatusCode {
        self.status
    }
}

impl<OnOperationResponseHookOutput> Response<OnOperationResponseHookOutput> {
    pub(crate) fn refuse_request_with(
        status: http::StatusCode,
        errors: impl IntoIterator<Item = impl Into<GraphqlError>>,
    ) -> Self {
        let mut error_code_counter = ErrorCodeCounter::default();

        let errors = errors
            .into_iter()
            .map(|error| {
                let error = error.into();
                error_code_counter.increment(error.code);
                error
            })
            .collect::<Vec<_>>();

        Self::RefusedRequest(RefusedRequestResponse {
            status,
            errors,
            error_code_counter,
            extensions: None,
        })
    }

    pub(crate) fn request_error<E>(
        operation_attributes: Option<GraphqlOperationAttributes>,
        errors: impl IntoIterator<Item = E>,
    ) -> Self
    where
        E: Into<GraphqlError>,
    {
        let errors = errors.into_iter().map(Into::into).collect::<Vec<_>>();
        let error_code_counter = ErrorCodeCounter::from_errors(&errors);

        Self::RequestError(RequestErrorResponse {
            operation_attributes,
            errors,
            error_code_counter,
            extensions: None,
        })
    }

    pub(crate) fn execution_error(
        operation: &PreparedOperation,
        on_operation_response_output: Option<OnOperationResponseHookOutput>,
        errors: impl IntoIterator<Item: Into<GraphqlError>>,
    ) -> Self {
        let errors = errors.into_iter().map(Into::into).collect::<Vec<_>>();
        let error_code_counter = ErrorCodeCounter::from_errors(&errors);

        Self::Executed(ExecutedResponse {
            operation: operation.cached.clone(),
            operation_attributes: operation.attributes(),
            data: None,
            on_operation_response_output,
            errors,
            error_code_counter,
            extensions: None,
        })
    }

    pub(crate) fn with_grafbase_extension(mut self, ext: Option<GrafbaseResponseExtension>) -> Self {
        self.extensions_mut().grafbase = ext;
        self
    }

    pub(crate) fn extensions_mut(&mut self) -> &mut ResponseExtensions {
        match self {
            Self::RefusedRequest(resp) => &mut resp.extensions,
            Self::RequestError(resp) => &mut resp.extensions,
            Self::Executed(resp) => &mut resp.extensions,
        }
        .get_or_insert_with(Default::default)
    }

    pub(crate) fn take_on_operation_response_output(&mut self) -> Option<OnOperationResponseHookOutput> {
        match self {
            Self::Executed(resp) => std::mem::take(&mut resp.on_operation_response_output),
            _ => None,
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
            Self::RefusedRequest(_) => None,
            Self::RequestError(resp) => resp.operation_attributes.as_ref(),
            Self::Executed(resp) => Some(&resp.operation_attributes),
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
            Response::RefusedRequest(resp) => &resp.errors,
            Response::RequestError(resp) => &resp.errors,
            Response::Executed(resp) => &resp.errors,
        }
    }

    pub(crate) fn error_code_counter(&self) -> &ErrorCodeCounter {
        match self {
            Response::RefusedRequest(resp) => &resp.error_code_counter,
            Response::RequestError(resp) => &resp.error_code_counter,
            Response::Executed(resp) => &resp.error_code_counter,
        }
    }
}

impl<OnOperationResponseHookOutput> std::fmt::Debug for Response<OnOperationResponseHookOutput> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response").finish_non_exhaustive()
    }
}
