use std::{borrow::Cow, sync::Arc};

pub(crate) use error::*;
use grafbase_telemetry::gql_response_status::GraphqlResponseStatus;
pub use key::*;
pub use path::*;
pub use read::*;
use schema::Schema;
pub use shape::*;
pub use value::{ResponseObject, ResponseValue};
pub use write::*;

use crate::operation::PreparedOperation;

mod error;
mod key;
mod path;
mod read;
mod shape;
mod value;
mod write;

pub(crate) enum Response {
    Initial(InitialResponse),
    /// Engine could not process the request at all, but request was valid.
    /// Meaning `data` field is present, but null.
    ExecutionFailure(ExecutionFailureResponse),
    /// Invalid request
    PreExecutionError(PreExecutionErrorResponse),
}

pub(crate) struct InitialResponse {
    // will be None if an error propagated up to the root.
    data: ResponseData,
    errors: Vec<GraphqlError>,
}

struct ResponseData {
    schema: Arc<Schema>,
    operation: Arc<PreparedOperation>,
    root: Option<ResponseObjectId>,
    parts: Vec<ResponseDataPart>,
}

pub(crate) struct PreExecutionErrorResponse {
    errors: Vec<GraphqlError>,
}

pub(crate) struct ExecutionFailureResponse {
    errors: Vec<GraphqlError>,
}

impl Response {
    pub(crate) fn pre_execution_error(error: impl Into<GraphqlError>) -> Self {
        Self::PreExecutionError(PreExecutionErrorResponse {
            errors: vec![error.into()],
        })
    }

    pub(crate) fn pre_execution_errors<E>(errors: impl IntoIterator<Item = E>) -> Self
    where
        E: Into<GraphqlError>,
    {
        Self::PreExecutionError(PreExecutionErrorResponse {
            errors: errors.into_iter().map(Into::into).collect(),
        })
    }

    pub(crate) fn execution_error(error: impl Into<GraphqlError>) -> Self {
        Self::ExecutionFailure(ExecutionFailureResponse {
            errors: vec![error.into()],
        })
    }

    pub(crate) fn status(&self) -> GraphqlResponseStatus {
        match self {
            Self::Initial(resp) => {
                if resp.errors.is_empty() {
                    GraphqlResponseStatus::Success
                } else {
                    GraphqlResponseStatus::FieldError {
                        count: resp.errors.len() as u64,
                        data_is_null: resp.data.root.is_none(),
                    }
                }
            }
            Self::ExecutionFailure(resp) => GraphqlResponseStatus::FieldError {
                count: resp.errors.len() as u64,
                data_is_null: true,
            },
            Self::PreExecutionError(resp) => GraphqlResponseStatus::RequestError {
                count: resp.errors.len() as u64,
            },
        }
    }

    pub(crate) fn first_error_message(&self) -> Option<Cow<'static, str>> {
        match self {
            Response::Initial(resp) => resp.errors.first(),
            Response::ExecutionFailure(resp) => resp.errors.first(),
            Response::PreExecutionError(resp) => resp.errors.first(),
        }
        .map(|error| error.message.clone())
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response").finish_non_exhaustive()
    }
}
