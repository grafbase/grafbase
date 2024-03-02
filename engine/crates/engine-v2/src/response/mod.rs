use std::sync::Arc;

use engine::HttpGraphqlResponse;
pub use engine_v2_common::ExecutionMetadata;
pub(crate) use error::GraphqlError;
pub use key::*;
pub use path::*;
pub use read::*;
use schema::Schema;
pub use value::{ResponseObject, ResponseValue};
pub use write::*;

use crate::plan::OperationPlan;

mod error;
mod key;
mod path;
mod read;
mod value;
mod write;

pub enum Response {
    Initial(InitialResponse),
    /// Engine could not execute the request.
    BadRequest(BadRequest),
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

pub struct BadRequest {
    errors: Vec<GraphqlError>,
    metadata: ExecutionMetadata,
}

impl Response {
    pub fn error(
        message: impl Into<String>,
        extensions: impl IntoIterator<Item = (String, serde_json::Value)>,
    ) -> Self {
        Self::from_error(
            GraphqlError {
                message: message.into(),
                extensions: extensions.into_iter().collect(),
                ..Default::default()
            },
            ExecutionMetadata::default(),
        )
    }

    pub(crate) fn bad_request(error: GraphqlError) -> Self {
        Self::BadRequest(BadRequest {
            errors: vec![error],
            metadata: ExecutionMetadata::default(),
        })
    }

    pub(crate) fn from_error(error: impl Into<GraphqlError>, metadata: ExecutionMetadata) -> Self {
        Self::BadRequest(BadRequest {
            errors: vec![error.into()],
            metadata,
        })
    }

    pub(crate) fn from_errors<E>(errors: impl IntoIterator<Item = E>, metadata: ExecutionMetadata) -> Self
    where
        E: Into<GraphqlError>,
    {
        Self::BadRequest(BadRequest {
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
            Self::BadRequest(resp) => !resp.errors.is_empty(),
        }
    }

    pub fn metadata(&self) -> &ExecutionMetadata {
        match self {
            Self::Initial(resp) => &resp.metadata,
            Self::BadRequest(resp) => &resp.metadata,
        }
    }

    pub fn take_metadata(self) -> ExecutionMetadata {
        match self {
            Self::Initial(initial) => initial.metadata,
            Self::BadRequest(request_error) => request_error.metadata,
        }
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, Vec<u8>> {
        serde_json::to_vec(self).map_err(|err| {
            tracing::error!("Failed to serialize response: {}", err);
            serde_json::to_vec(&serde_json::json!({
                "errors": [
                    {"message": "Internal server error"}
                ]
            }))
            .unwrap()
        })
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response").finish_non_exhaustive()
    }
}

impl From<Response> for HttpGraphqlResponse {
    fn from(response: Response) -> Self {
        HttpGraphqlResponse::from_json(&response).with_metadata(response.take_metadata())
    }
}
