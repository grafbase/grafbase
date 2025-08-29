use std::num::NonZero;

use walker::Walk;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct QueryPosition(NonZero<u16>);

impl QueryPosition {
    pub const MAX: QueryPosition = QueryPosition(NonZero::new(u16::MAX - 1).unwrap());

    pub fn cmp_with_none_last(a: Option<QueryPosition>, b: Option<QueryPosition>) -> std::cmp::Ordering {
        // None -> 0
        // Some(x) -> x
        let mut a: u16 = zerocopy::transmute!(a.map(|qp| qp.0));
        let mut b: u16 = zerocopy::transmute!(b.map(|qp| qp.0));
        // x -> x -1
        // 0 -> u32::MAX
        a = a.wrapping_sub(1);
        b = b.wrapping_sub(1);
        a.cmp(&b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct PositionedResponseKey {
    /// If not present, it's an extra field.
    pub query_position: Option<QueryPosition>,
    pub response_key: ResponseKey,
}

impl PositionedResponseKey {
    pub fn with_query_position_if(self, included: bool) -> PositionedResponseKey {
        let mut qp: u16 = zerocopy::transmute!(self.query_position.map(|qp| qp.0));
        qp &= (!(included as u16)).wrapping_add(1);
        let qp: Option<NonZero<u16>> = zerocopy::transmute!(qp);
        Self {
            query_position: qp.map(QueryPosition),
            response_key: self.response_key,
        }
    }
}

impl From<PositionedResponseKey> for ResponseKey {
    fn from(key: PositionedResponseKey) -> Self {
        key.response_key
    }
}

impl ResponseKey {
    pub fn with_position(self, query_position: Option<QueryPosition>) -> PositionedResponseKey {
        PositionedResponseKey {
            query_position,
            response_key: self,
        }
    }
}

/// A ResponseKey is guaranteed to exist inside ResponseKeys
/// and thus will use `get_unchecked` to be retrieved. This improves
/// performance by around 1% since we're doing a binary search for each
/// incoming field name during deserialization.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ResponseKey(NonZero<u16>);

impl ResponseKey {
    /// # Safety
    /// Only use this method after we parsed a valid operation. It will contain at least one
    /// field and thus at least one response key.
    pub unsafe fn null() -> Self {
        // SAFETY: Obviously non zero
        Self(unsafe { NonZero::new_unchecked(1) })
    }
}

impl From<ResponseKey> for usize {
    fn from(key: ResponseKey) -> usize {
        (key.0.get() - 1) as usize
    }
}

impl From<ResponseKey> for u32 {
    fn from(key: ResponseKey) -> u32 {
        (key.0.get() - 1) as u32
    }
}

unsafe impl lasso2::Key for ResponseKey {
    fn into_usize(self) -> usize {
        usize::from(self)
    }

    fn try_from_usize(id: usize) -> Option<Self> {
        u16::try_from(id + 1).ok().map(|n| Self(NonZero::new(n).unwrap()))
    }
}

impl<'a> Walk<&'a ResponseKeys> for ResponseKey {
    type Walker<'w>
        = &'w str
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<&'a ResponseKeys>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let keys: &'a ResponseKeys = ctx.into();
        &keys[self]
    }
}

/// Interns all of the response keys strings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseKeys(lasso2::Rodeo<ResponseKey>);

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

#[cfg(test)]
mod tests {
    use super::*;
    use lasso2::Key;

    #[test]
    fn field_name_value_in_range() {
        let key = ResponseKey::try_from_usize(0).unwrap();
        assert_eq!(key.into_usize(), 0);

        let key = ResponseKey::try_from_usize((u16::MAX - 1) as usize).unwrap();
        assert_eq!(key.into_usize(), ((u16::MAX - 1) as usize));
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = ResponseKey::try_from_usize(u32::MAX as usize);
        assert!(key.is_none());
    }
}
