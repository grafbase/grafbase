use std::sync::Arc;

pub(crate) use error::GraphqlError;
use grafbase_tracing::gql_response_status::GraphqlResponseStatus;
pub use key::*;
pub use path::*;
pub use read::*;
use schema::Schema;
pub use value::{ResponseObject, ResponseValue};
pub use write::*;

use crate::{http_response::HttpGraphqlResponse, operation::Operation};

mod error;
mod key;
mod path;
mod read;
mod value;
mod write;

pub(crate) enum Response {
    Initial(InitialResponse),
    /// Engine could not process the request at all, but request was valid.
    /// Meaning `data` field is present, but null.
    ExecutionFailure(ExecutionFailureResponse),
    /// Invalid request
    BadRequest(BadRequestResponse),
}

pub(crate) struct InitialResponse {
    // will be None if an error propagated up to the root.
    data: ResponseData,
    errors: Vec<GraphqlError>,
}

struct ResponseData {
    schema: Arc<Schema>,
    operation: Arc<Operation>,
    root: Option<ResponseObjectId>,
    parts: Vec<ResponseDataPart>,
}

pub(crate) struct BadRequestResponse {
    errors: Vec<GraphqlError>,
}

pub(crate) struct ExecutionFailureResponse {
    errors: Vec<GraphqlError>,
}

impl Response {
    pub(crate) fn bad_request(error: impl Into<GraphqlError>) -> Self {
        Self::BadRequest(BadRequestResponse {
            errors: vec![error.into()],
        })
    }

    pub(crate) fn bad_request_from_errors<E>(errors: impl IntoIterator<Item = E>) -> Self
    where
        E: Into<GraphqlError>,
    {
        Self::BadRequest(BadRequestResponse {
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
            Self::BadRequest(resp) => GraphqlResponseStatus::RequestError {
                count: resp.errors.len() as u64,
            },
        }
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response").finish_non_exhaustive()
    }
}

impl From<Response> for HttpGraphqlResponse {
    fn from(response: Response) -> Self {
        HttpGraphqlResponse::from_json(response.status(), &response)
    }
}
