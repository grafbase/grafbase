/// A "safe" ResponseKey is guaranteed to exist inside ResponseKeys
/// and thus will use `get_unchecked` to be retrieved. This improves
/// performance by around 1% since we're doing a binary search for each
/// incoming field name during deserialization.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct SafeResponseKey(u16);

/// Interns all of the response keys strings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseKeys(lasso2::Rodeo<SafeResponseKey>);

impl From<SafeResponseKey> for u32 {
    fn from(key: SafeResponseKey) -> u32 {
        key.0 as u32
    }
}

impl Default for ResponseKeys {
    fn default() -> Self {
        Self(lasso2::Rodeo::new())
    }
}

impl ResponseKeys {
    pub fn get(&self, key: &str) -> Option<SafeResponseKey> {
        self.0.get(key)
    }

    pub fn get_or_intern(&mut self, s: &str) -> SafeResponseKey {
        self.0.get_or_intern(s)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.contains(key)
    }
}

impl std::ops::Index<SafeResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, key: SafeResponseKey) -> &Self::Output {
        // SAFETY: SafeResponseKey are only created by ResponseKeys, either by `get_or_intern` or `ensure_safety`.
        unsafe { self.0.resolve_unchecked(&key) }
    }
}

unsafe impl lasso2::Key for SafeResponseKey {
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
        let key = SafeResponseKey::try_from_usize(0).unwrap();
        assert_eq!(key.into_usize(), 0);

        let key = SafeResponseKey::try_from_usize(u16::MAX as usize).unwrap();
        assert_eq!(key.into_usize(), (u16::MAX as usize));
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = SafeResponseKey::try_from_usize(u64::MAX as usize);
        assert!(key.is_none());

        let key = SafeResponseKey::try_from_usize(u32::MAX as usize);
        assert!(key.is_none());
    }
}
