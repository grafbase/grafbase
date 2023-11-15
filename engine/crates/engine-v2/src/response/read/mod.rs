use super::{GraphqlError, Response, ResponseObjectId};
use crate::request::OperationPath;

mod selection_set;
mod ser;
mod view;

pub use selection_set::{ReadSelection, ReadSelectionSet};
use ser::SerializableObject;
pub use view::ResponseObjectsView;

impl Response {
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

    /// Used to provide a view on the inputs objects of a plan.
    pub fn read_objects<'a>(
        &'a self,
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

    fn find_matching_object_node_ids(&self, path: &OperationPath) -> Vec<ResponseObjectId> {
        let Some(root) = self.root else {
            return vec![];
        };
        let mut nodes = vec![root];

        for segment in path {
            if let Some(ref type_condition) = segment.type_condition {
                nodes = nodes
                    .into_iter()
                    .filter_map(|node_id| {
                        let node = self.get(node_id);
                        let object_id = node
                            .object_id()
                            .expect("Missing object_id on a node that is subject to a type condition.");
                        if type_condition.matches(object_id) {
                            node.field(segment.position, segment.name)
                                .and_then(|node| node.as_object())
                        } else {
                            None
                        }
                    })
                    .collect();
            } else {
                nodes = nodes
                    .into_iter()
                    .filter_map(|node_id| {
                        self.get(node_id)
                            .field(segment.position, segment.name)
                            .and_then(|node| node.as_object())
                    })
                    .collect();
            }
            if nodes.is_empty() {
                break;
            }
        }

        nodes
    }
}

#[derive(serde::Serialize)]
struct SerializableResponse<'a, T> {
    data: T,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    errors: &'a [GraphqlError],
}
