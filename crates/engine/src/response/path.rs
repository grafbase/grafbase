use std::cell::Ref;

use error::{ErrorPath, InsertIntoErrorPath};
use operation::{PositionedResponseKey, QueryPosition, ResponseKey};

use super::{DataPartId, ResponseListId, ResponseObjectId};

/// Unique identifier of a value within the response. Used to propagate null at the right place
/// and to generate the appropriate error path for GraphQL errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResponseValueId {
    Field {
        object_id: ResponseObjectId,
        query_position: Option<QueryPosition>,
        response_key: ResponseKey,
        nullable: bool,
    },
    Index {
        list_id: ResponseListId,
        index: u32,
        nullable: bool,
    },
}

impl ResponseValueId {
    pub fn field(object_id: ResponseObjectId, key: PositionedResponseKey, nullable: bool) -> Self {
        Self::Field {
            object_id,
            query_position: key.query_position,
            response_key: key.response_key,
            nullable,
        }
    }
    pub fn is_nullable(&self) -> bool {
        match self {
            ResponseValueId::Field { nullable, .. } => *nullable,
            ResponseValueId::Index { nullable, .. } => *nullable,
        }
    }

    pub fn part_id(&self) -> DataPartId {
        match self {
            ResponseValueId::Field { object_id, .. } => object_id.part_id,
            ResponseValueId::Index { list_id, .. } => list_id.part_id,
        }
    }
}

impl InsertIntoErrorPath for &ResponseValueId {
    fn insert_into(self, path: &mut ErrorPath) {
        match self {
            ResponseValueId::Field { response_key, .. } => response_key.insert_into(path),
            ResponseValueId::Index { index, .. } => index.insert_into(path),
        }
    }
}

pub(crate) trait ResponsePath {
    fn iter(&self) -> impl DoubleEndedIterator<Item = &ResponseValueId>;
}

impl ResponsePath for [ResponseValueId] {
    fn iter(&self) -> impl DoubleEndedIterator<Item = &ResponseValueId> {
        self.iter()
    }
}

impl ResponsePath for &[ResponseValueId] {
    fn iter(&self) -> impl DoubleEndedIterator<Item = &ResponseValueId> {
        (*self).iter()
    }
}

impl ResponsePath for Vec<ResponseValueId> {
    fn iter(&self) -> impl DoubleEndedIterator<Item = &ResponseValueId> {
        self.as_slice().iter()
    }
}

impl ResponsePath for Ref<'_, Vec<ResponseValueId>> {
    fn iter(&self) -> impl DoubleEndedIterator<Item = &ResponseValueId> {
        self.as_slice().iter()
    }
}

#[cfg(test)]
#[test]
fn response_value_id_size() {
    assert_eq!(std::mem::size_of::<ResponseValueId>(), 16);
    assert_eq!(std::mem::align_of::<ResponseValueId>(), 4);
}
