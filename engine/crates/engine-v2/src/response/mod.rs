mod data;
mod error;
mod read;
mod value;
mod write;

pub use data::{ResponseData, ResponseObject, ResponseObjectId};
pub use error::{GraphqlError, GraphqlErrors};
use read::SerializableResponseData;
pub use read::{ReadSelection, ReadSelectionSet, ResponseObjectRoot, ResponseObjectsView};
pub use value::ResponseValue;

#[derive(serde::Serialize)]
pub struct Response {
    pub(crate) data: Option<SerializableResponseData>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub(crate) errors: Vec<GraphqlError>,
}

impl Response {
    pub fn data(&self) -> impl serde::Serialize + '_ {
        &self.data
    }

    pub fn errors(&self) -> &[GraphqlError] {
        &self.errors
    }

    pub fn from_error(error: impl Into<GraphqlError>) -> Self {
        Self {
            data: None,
            errors: vec![error.into()],
        }
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("data", &serde_json::to_value(&self.data))
            .field("errors", &self.errors)
            .finish()
    }
}
