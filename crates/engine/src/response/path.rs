use std::cell::Ref;

use error::{ErrorPath, InsertIntoErrorPath};
use operation::PositionedResponseKey;

use crate::response::{PartListId, PartObjectId};

use super::{DataPartId, ResponseListId, ResponseObjectId};

/// Unique identifier of a value within the response. Used to propagate null at the right place
/// and to generate the appropriate error path for GraphQL errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResponseValueId {
    Field {
        part_id: DataPartId,
        object_id: PartObjectId,
        key: PositionedResponseKey,
        nullable: bool,
    },
    Index {
        part_id: DataPartId,
        list_id: PartListId,
        index: u32,
        nullable: bool,
    },
}

impl ResponseValueId {
    pub fn field(
        ResponseObjectId { part_id, object_id }: ResponseObjectId,
        key: PositionedResponseKey,
        nullable: bool,
    ) -> Self {
        Self::Field {
            part_id,
            object_id,
            key,
            nullable,
        }
    }
    pub fn index(ResponseListId { part_id, list_id }: ResponseListId, index: u32, nullable: bool) -> Self {
        Self::Index {
            part_id,
            list_id,
            index,
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
            ResponseValueId::Field { part_id, .. } => *part_id,
            ResponseValueId::Index { part_id, .. } => *part_id,
        }
    }
}

impl InsertIntoErrorPath for &ResponseValueId {
    fn insert_into(self, path: &mut ErrorPath) {
        match self {
            ResponseValueId::Field { key, .. } => key.insert_into(path),
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
    assert_eq!(std::mem::size_of::<ResponseValueId>(), 12);
    assert_eq!(std::mem::align_of::<ResponseValueId>(), 4);
}
