mod private;
use std::cmp::Ordering;

pub use private::*;

use crate::operation::QueryPosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PositionedResponseKey {
    /// If not present, it's an extra field.
    pub query_position: Option<QueryPosition>,
    pub response_key: SafeResponseKey,
}

impl Ord for PositionedResponseKey {
    // Inverting the ordering of Option. We want extra fields at the end, so None should be treated
    // as the highest position possible.
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.query_position, other.query_position) {
            (None, None) => self.response_key.cmp(&other.response_key),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(l), Some(r)) => l.cmp(&r).then(self.response_key.cmp(&other.response_key)),
        }
    }
}

impl PartialOrd for PositionedResponseKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl SafeResponseKey {
    pub(crate) fn with_position(self, query_position: QueryPosition) -> PositionedResponseKey {
        PositionedResponseKey {
            query_position: Some(query_position),
            response_key: self,
        }
    }
}
