use std::sync::Arc;

pub(crate) use error::GraphqlError;
pub use key::*;
pub use path::*;
pub use read::*;
use schema::Schema;
pub use value::{ResponseObject, ResponseValue};
pub use write::*;

use crate::{http_response::HttpGraphqlResponse, plan::OperationPlan};

mod error;
mod key;
mod path;
mod read;
mod value;
mod write;

pub(crate) enum Response {
    Initial(InitialResponse),
    /// Engine could not execute the request.
    BadRequest(BadRequestResponse),
}

pub(crate) struct InitialResponse {
    // will be None if an error propagated up to the root.
    data: ResponseData,
    errors: Vec<GraphqlError>,
}

struct ResponseData {
    schema: Arc<Schema>,
    operation: Arc<OperationPlan>,
    root: Option<ResponseObjectId>,
    parts: Vec<ResponseDataPart>,
}

pub(crate) struct BadRequestResponse {
    errors: Vec<GraphqlError>,
}

impl Response {
    pub(crate) fn from_error(error: impl Into<GraphqlError>) -> Self {
        Self::BadRequest(BadRequestResponse {
            errors: vec![error.into()],
        })
    }

    pub(crate) fn from_errors<E>(errors: impl IntoIterator<Item = E>) -> Self
    where
        E: Into<GraphqlError>,
    {
        Self::BadRequest(BadRequestResponse {
            errors: errors.into_iter().map(Into::into).collect(),
        })
    }

    pub(crate) fn has_errors(&self) -> bool {
        match self {
            Self::Initial(resp) => !resp.errors.is_empty(),
            Self::BadRequest(resp) => !resp.errors.is_empty(),
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
        HttpGraphqlResponse::from_json(&response)
    }
}
