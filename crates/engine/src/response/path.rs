use super::{DataPartId, ErrorPathSegment, ResponseListId, ResponseObjectId, SafeResponseKey};

/// Unique identifier of a value within the response. Used to propagate null at the right place
/// and to generate the appropriate error path for GraphQL errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResponseValueId {
    Field {
        object_id: ResponseObjectId,
        key: SafeResponseKey,
        nullable: bool,
    },
    Index {
        list_id: ResponseListId,
        index: u32,
        nullable: bool,
    },
}

impl ResponseValueId {
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

impl From<&ResponseValueId> for ErrorPathSegment {
    fn from(segment: &ResponseValueId) -> Self {
        match segment {
            ResponseValueId::Field { key, .. } => ErrorPathSegment::Field(*key),
            ResponseValueId::Index { index, .. } => ErrorPathSegment::Index(*index as usize),
        }
    }
}
