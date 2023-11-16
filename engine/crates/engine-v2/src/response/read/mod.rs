use super::{GraphqlError, Response};
use crate::request::OperationPath;

mod selection_set;
mod ser;
mod view;

pub use selection_set::{ReadSelection, ReadSelectionSet};
use ser::SerializableObject;
pub use view::ResponseObjectsView;

impl Response {
    /// Used to provide a view on the inputs objects of a plan.
    pub fn read_objects<'a>(
        &'a mut self,
        path: &'a OperationPath,
        selection_set: &'a ReadSelectionSet,
    ) -> Option<ResponseObjectsView<'a>> {
        let response_object_ids = self.find_matching_object_node_ids(path);
        if response_object_ids.is_empty() {
            None
        } else {
            Some(ResponseObjectsView {
                response_object_ids,
                response: self,
                selection_set,
            })
        }
    }

    pub fn as_serializable<'s>(&'s self, selection_set: &'s ReadSelectionSet) -> impl serde::Serialize + 's {
        SerializableResponse {
            data: self.root.map(|root| SerializableObject {
                response: self,
                object: self.get(root),
                selection_set,
            }),
            errors: &self.errors,
        }
    }
}

#[derive(serde::Serialize)]
struct SerializableResponse<'a, T> {
    data: T,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    errors: &'a [GraphqlError],
}
