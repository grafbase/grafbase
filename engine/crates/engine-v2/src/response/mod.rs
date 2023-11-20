mod data;
mod error;
mod read;
mod value;
mod write;

pub use data::{
    AnyResponseMutObject, AnyResponseObject, CompactResponseObject, ResponseData, ResponseObject, ResponseObjectId,
};
pub use error::{GraphqlError, GraphqlErrors};
use read::SerializableResponseData;
pub use read::{ReadSelection, ReadSelectionSet, ResponseObjectsView};
pub use value::ResponseValue;

#[derive(serde::Serialize)]
pub struct Response<'a> {
    pub(crate) data: Option<SerializableResponseData<'a>>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub(crate) errors: Vec<GraphqlError>,
}

impl<'a> Response<'a> {
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
