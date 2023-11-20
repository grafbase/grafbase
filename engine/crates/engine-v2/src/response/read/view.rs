use serde::ser::SerializeSeq;

use super::{ser::SerializableObject, ReadSelectionSet};
use crate::response::{Response, ResponseObjectId};

pub struct ResponseObjectsView<'a> {
    pub(super) response: &'a Response,
    pub(super) response_object_ids: Vec<ResponseObjectId>,
    pub(super) selection_set: &'a ReadSelectionSet,
}

impl<'a> ResponseObjectsView<'a> {
    pub fn id(&self) -> ResponseObjectId {
        *self
            .response_object_ids
            .first()
            .expect("At least one object node id must be present in a Input.")
    }

    // Guaranteed to be in the same order as the response objects themselves
    pub fn ids(&self) -> &[ResponseObjectId] {
        &self.response_object_ids
    }
}

impl<'a> serde::Serialize for ResponseObjectsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.response_object_ids.len()))?;
        for node_id in &self.response_object_ids {
            seq.serialize_element(&SerializableObject {
                response: self.response,
                object: self.response.get(*node_id),
                selection_set: self.selection_set,
            })?;
        }
        seq.end()
    }
}
