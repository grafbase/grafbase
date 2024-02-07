mod private;
pub use private::*;

impl ResponseEdge {
    pub fn is_extra(&self) -> bool {
        matches!(self.unpack(), UnpackedResponseEdge::ExtraFieldResponseKey(_))
    }

    pub fn as_response_key(&self) -> Option<ResponseKey> {
        match self.unpack() {
            UnpackedResponseEdge::BoundResponseKey(key) => Some(key.into()),
            UnpackedResponseEdge::ExtraFieldResponseKey(key) => Some(key),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResponseKeys(lasso::Rodeo<ResponseKey>);

impl Default for ResponseKeys {
    fn default() -> Self {
        Self(lasso::Rodeo::new())
    }
}

impl ResponseKeys {
    pub fn get_or_intern(&mut self, s: &str) -> ResponseKey {
        self.0.get_or_intern(s)
    }

    pub fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }

    pub fn try_resolve(&self, key: ResponseKey) -> Option<&str> {
        self.0.try_resolve(&key)
    }
}

impl std::ops::Index<BoundResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, bound_key: BoundResponseKey) -> &Self::Output {
        // SAFETY: We only generate a BoundResponseKey for a key that exist.
        unsafe { self.0.resolve_unchecked(&bound_key.key()) }
    }
}

impl std::ops::Index<ResponseKey> for ResponseKeys {
    type Output = str;

    fn index(&self, key: ResponseKey) -> &Self::Output {
        // SAFETY: We only generate a ResponseKey for a key that exist.
        unsafe { self.0.resolve_unchecked(&key) }
    }
}

impl std::fmt::Debug for BoundResponseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundResponseKey")
            .field("position", &self.position())
            .field("key", &self.key())
            .finish()
    }
}

impl From<BoundResponseKey> for ResponseKey {
    fn from(bound_key: BoundResponseKey) -> Self {
        bound_key.key()
    }
}

impl From<&BoundResponseKey> for ResponseKey {
    fn from(bound_key: &BoundResponseKey) -> Self {
        bound_key.key()
    }
}
