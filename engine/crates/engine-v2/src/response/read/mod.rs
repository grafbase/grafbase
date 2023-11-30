use super::ResponseBuilder;
use crate::{request::QueryPath, response::ResponsePath};
mod selection_set;
mod ser;
mod view;

use schema::Schema;
pub use selection_set::ReadSelectionSet;
pub use view::{ResponseObjectRoot, ResponseObjectsView};

impl ResponseBuilder {
    /// Used to provide a view on the inputs objects of a plan.
    pub fn read_objects<'a>(
        &'a self,
        schema: &'a Schema,
        path: &'a QueryPath,
        selection_set: &'a ReadSelectionSet,
    ) -> Option<ResponseObjectsView<'a>> {
        let response_object_ids = self.find_matching_object_node_ids(path);
        if response_object_ids.is_empty() {
            None
        } else {
            Some(ResponseObjectsView {
                schema,
                roots: response_object_ids,
                response: self,
                selection_set,
            })
        }
    }

    // Will be removed later. We can keep track of those during the writing of the previous plan.
    fn find_matching_object_node_ids(&self, _path: &QueryPath) -> Vec<ResponseObjectRoot> {
        let Some(root) = self.root else {
            return vec![];
        };
        vec![ResponseObjectRoot {
            id: root,
            object_id: self[root].object_id,
            path: ResponsePath::default(),
        }]
    }
}
