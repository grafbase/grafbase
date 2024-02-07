use crate::response::BoundResponseKey;

use super::{ResponseKey, MAX_RESPONSE_KEY};

/// A "safe" ResponseKey is guaranteed to exist inside ResponseKeys
/// and thus will use `get_unchecked` to be retrieved. This improves
/// performance by around 1% since we're doing a binary search for each
/// incoming field name during deserialization.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SafeResponseKey(u32);

/// Interns all of the response keys strings.
#[derive(Debug, Clone)]
pub struct ResponseKeys(lasso::Rodeo<SafeResponseKey>);

impl From<SafeResponseKey> for u32 {
    fn from(key: SafeResponseKey) -> u32 {
        key.0
    }
}

impl From<SafeResponseKey> for ResponseKey {
    fn from(value: SafeResponseKey) -> Self {
        ResponseKey(value.0)
    }
}

impl Default for ResponseKeys {
    fn default() -> Self {
        Self(lasso::Rodeo::new())
    }
}

impl ResponseKeys {
    pub fn get_or_intern(&mut self, s: &str) -> SafeResponseKey {
        self.0.get_or_intern(s)
    }

    pub fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }

    pub fn try_resolve(&self, key: ResponseKey) -> Option<&str> {
        self.0.try_resolve(&SafeResponseKey(key.0))
    }

    pub fn ensure_safety(&self, key: ResponseKey) -> SafeResponseKey {
        let key = SafeResponseKey(key.0);
        assert!(self.0.contains_key(&key));
        key
    }
}

impl std::ops::Index<BoundResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, bound_key: BoundResponseKey) -> &Self::Output {
        self.0.resolve(&SafeResponseKey(bound_key.as_response_key().0))
    }
}

impl std::ops::Index<ResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, key: ResponseKey) -> &Self::Output {
        self.0.resolve(&SafeResponseKey(key.0))
    }
}

impl std::ops::Index<SafeResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, key: SafeResponseKey) -> &Self::Output {
        // SAFETY: SafeResponseKey are only created by ResponseKeys, either by `get_or_intern` or `ensure_safety`.
        unsafe { self.0.resolve_unchecked(&key) }
    }
}

unsafe impl lasso::Key for SafeResponseKey {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn try_from_usize(id: usize) -> Option<Self> {
        let id = u32::try_from(id).ok()?;
        if id <= MAX_RESPONSE_KEY {
            Some(Self(id))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lasso::Key;

    #[test]
    fn field_name_value_in_range() {
        let key = SafeResponseKey::try_from_usize(0).unwrap();
        assert_eq!(key.into_usize(), 0);

        let key = SafeResponseKey::try_from_usize(MAX_RESPONSE_KEY as usize).unwrap();
        assert_eq!(key.into_usize(), (MAX_RESPONSE_KEY as usize));
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = SafeResponseKey::try_from_usize((MAX_RESPONSE_KEY + 1) as usize);
        assert!(key.is_none());

        let key = SafeResponseKey::try_from_usize(u32::max_value() as usize);
        assert!(key.is_none());
    }
}
