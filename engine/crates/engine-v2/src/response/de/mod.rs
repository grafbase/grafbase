use serde::de::DeserializeSeed;

use super::{Response, ResponseObjectId, WriteSelectionSet};

mod any;

pub use any::AnyFieldsSeed;

impl Response {
    // Temporary as it's simple. We still need to validate the data we're receiving in all cases.
    // Upstream might break the contract. This basically got me started.
    pub fn insert_any<'de, D>(&mut self, object_node_id: ResponseObjectId, deserializer: D) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let seed = AnyFieldsSeed { response: self };
        let fields = seed.deserialize(deserializer)?;
        self[object_node_id].insert_fields(fields);
        Ok(())
    }

    pub fn insert<'de, D>(
        &mut self,
        _selection_set: &WriteSelectionSet,
        _node_id: ResponseObjectId,
        _fields: impl serde::Deserialize<'de>,
    ) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
    pub fn insert_multiple<'de, D>(
        &mut self,
        _selection_set: &WriteSelectionSet,
        _node_ids: Vec<ResponseObjectId>,
        _objects_fields: impl serde::Deserialize<'de>,
    ) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }

    // To be used when the nothing is guaranteed for the output and we want to keep any additional
    // data (resolvers). We should only validate fields part of the selection set.
    pub fn insert_dirty<'de, D>(
        &mut self,
        _selection_set: &WriteSelectionSet,
        _node_id: ResponseObjectId,
        _fields: impl serde::Deserialize<'de>,
    ) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
    pub fn insert_multiple_dirty<'de, D>(
        &mut self,
        _selection_set: &WriteSelectionSet,
        _node_ids: Vec<ResponseObjectId>,
        _objects_fields: impl serde::Deserialize<'de>,
    ) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}
