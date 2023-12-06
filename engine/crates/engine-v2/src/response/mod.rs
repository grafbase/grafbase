mod error;
mod path;
mod read;
mod value;
mod write;

use std::sync::Arc;

pub use error::{GraphqlError, ServerError};
pub use path::{BoundResponseKey, ResponseKey, ResponseKeys, ResponsePath};
pub use read::*;
use schema::Schema;
pub use value::{ResponseObject, ResponseValue};
pub use write::*;

pub enum Response {
    Initial(InitialResponse),
    Error(ServerErrorResponse),
}

pub struct InitialResponse {
    // will be None if an error propagated up to the root.
    data: ResponseData,
    errors: Vec<GraphqlError>,
}

struct ResponseData {
    schema: Arc<Schema>,
    keys: ResponseKeys,
    root: Option<ResponseObjectId>,
    parts: Vec<ResponseDataPart>,
}

pub struct ServerErrorResponse {
    errors: Vec<ServerError>,
}

impl Response {
    pub fn from_error(error: impl Into<ServerError>) -> Self {
        Self::Error(ServerErrorResponse {
            errors: vec![error.into()],
        })
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response").finish_non_exhaustive()
    }
}
