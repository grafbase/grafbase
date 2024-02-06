use std::sync::Arc;

pub(crate) use error::GraphqlError;
pub use metadata::*;
pub use path::*;
pub use read::*;
use schema::Schema;
pub use value::{ResponseObject, ResponseValue};
pub use write::*;

use crate::plan::OperationPlan;

pub(crate) mod cacheable;
mod error;
mod metadata;
mod path;
mod read;
mod value;
mod write;

pub enum Response {
    Initial(InitialResponse),
    /// Engine could not execute the request.
    RequestError(RequestErrorResponse),
}

pub struct InitialResponse {
    // will be None if an error propagated up to the root.
    data: ResponseData,
    errors: Vec<GraphqlError>,
    metadata: ExecutionMetadata,
}

struct ResponseData {
    schema: Arc<Schema>,
    operation: Arc<OperationPlan>,
    root: Option<ResponseObjectId>,
    parts: Vec<ResponseDataPart>,
}

pub struct RequestErrorResponse {
    errors: Vec<GraphqlError>,
    metadata: ExecutionMetadata,
}

impl Response {
    pub fn error(message: impl Into<String>) -> Self {
        Self::from_error(GraphqlError::new(message), ExecutionMetadata::default())
    }

    pub(crate) fn from_error(error: impl Into<GraphqlError>, metadata: ExecutionMetadata) -> Self {
        Self::RequestError(RequestErrorResponse {
            errors: vec![error.into()],
            metadata,
        })
    }

    pub(crate) fn from_errors<E>(errors: impl IntoIterator<Item = E>, metadata: ExecutionMetadata) -> Self
    where
        E: Into<GraphqlError>,
    {
        Self::RequestError(RequestErrorResponse {
            errors: errors.into_iter().map(Into::into).collect(),
            metadata,
        })
    }

    // Our internal error struct is NOT meant to be public. If we ever need it, we should consider
    // exposing it through a Serializable struct, in the same way 'data' is only available through
    // serialization.
    pub fn has_errors(&self) -> bool {
        match self {
            Self::Initial(resp) => !resp.errors.is_empty(),
            Self::RequestError(resp) => !resp.errors.is_empty(),
        }
    }

    pub fn metadata(&self) -> &ExecutionMetadata {
        match self {
            Self::Initial(resp) => &resp.metadata,
            Self::RequestError(resp) => &resp.metadata,
        }
    }

    pub fn take_metadata(self) -> ExecutionMetadata {
        match self {
            Self::Initial(initial) => initial.metadata,
            Self::RequestError(request_error) => request_error.metadata,
        }
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response").finish_non_exhaustive()
    }
}
