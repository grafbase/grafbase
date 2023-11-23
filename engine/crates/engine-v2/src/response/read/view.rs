use schema::{ObjectId, Schema};
use serde::ser::SerializeSeq;

use super::{ser::SerializableResponseObject, ReadSelectionSet};
use crate::response::{ResponseData, ResponseObjectId};

pub struct ResponseObjectsView<'a> {
    pub(super) schema: &'a Schema,
    pub(super) response: &'a ResponseData,
    pub(super) roots: Vec<ResponseObjectRoot>,
    pub(super) selection_set: &'a ReadSelectionSet,
}

#[derive(Clone)]
pub struct ResponseObjectRoot {
    pub id: ResponseObjectId,
    pub object_id: ObjectId,
}

impl<'a> ResponseObjectsView<'a> {
    pub fn root(&self) -> ResponseObjectRoot {
        self.roots
            .first()
            .cloned()
            .expect("At least one object node id must be present in a Input.")
    }

    // Guaranteed to be in the same order as the response objects themselves
    #[allow(dead_code)]
    pub fn roots(&self) -> &[ResponseObjectRoot] {
        &self.roots
    }
}

impl<'a> serde::Serialize for ResponseObjectsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.roots.len()))?;
        for root in &self.roots {
            seq.serialize_element(&SerializableResponseObject {
                schema: self.schema,
                response: self.response,
                object: self.response.get(root.id),
                selection_set: self.selection_set,
            })?;
        }
        seq.end()
    }
}
