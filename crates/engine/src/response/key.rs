use std::cmp::Ordering;

use crate::operation::QueryPosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PositionedResponseKey {
    /// If not present, it's an extra field.
    pub query_position: Option<QueryPosition>,
    pub response_key: ResponseKey,
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

impl ResponseKey {
    pub(crate) fn with_position(self, query_position: QueryPosition) -> PositionedResponseKey {
        PositionedResponseKey {
            query_position: Some(query_position),
            response_key: self,
        }
    }
}

/// A ResponseKey is guaranteed to exist inside ResponseKeys
/// and thus will use `get_unchecked` to be retrieved. This improves
/// performance by around 1% since we're doing a binary search for each
/// incoming field name during deserialization.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ResponseKey(u16);

/// Interns all of the response keys strings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseKeys(lasso2::Rodeo<ResponseKey>);

impl From<ResponseKey> for u32 {
    fn from(key: ResponseKey) -> u32 {
        key.0 as u32
    }
}

impl Default for ResponseKeys {
    fn default() -> Self {
        Self(lasso2::Rodeo::new())
    }
}

impl ResponseKeys {
    pub fn get(&self, key: &str) -> Option<ResponseKey> {
        self.0.get(key)
    }

    pub fn get_or_intern(&mut self, s: &str) -> ResponseKey {
        self.0.get_or_intern(s)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.contains(key)
    }
}

impl std::ops::Index<PositionedResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, key: PositionedResponseKey) -> &Self::Output {
        &self[key.response_key]
    }
}

impl std::ops::Index<ResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, key: ResponseKey) -> &Self::Output {
        // SAFETY: SafeResponseKey are only created by ResponseKeys, either by `get_or_intern` or `ensure_safety`.
        unsafe { self.0.resolve_unchecked(&key) }
    }
}

unsafe impl lasso2::Key for ResponseKey {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn try_from_usize(id: usize) -> Option<Self> {
        u16::try_from(id).ok().map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lasso2::Key;

    #[test]
    fn field_name_value_in_range() {
        let key = ResponseKey::try_from_usize(0).unwrap();
        assert_eq!(key.into_usize(), 0);

        let key = ResponseKey::try_from_usize(u16::MAX as usize).unwrap();
        assert_eq!(key.into_usize(), (u16::MAX as usize));
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = ResponseKey::try_from_usize(u64::MAX as usize);
        assert!(key.is_none());

        let key = ResponseKey::try_from_usize(u32::MAX as usize);
        assert!(key.is_none());
    }
}
