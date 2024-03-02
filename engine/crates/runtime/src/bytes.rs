use ::bytes::Bytes;

/// Most of the time the engine returns an owned Vec<u8>, but they are at least two cases where we
/// must return a shared reference (Bytes):
/// - when updating the cache in the background for a stale value
/// - the multipart_stream crate returns Bytes.
///
/// While axum is does transform everything into Bytes anyway, Cloudflare worker requires Vec<u8>. An
/// Bytes does not provide a zero-copy transformation to Vec, even if it's not shared... The last
/// issue on it was: https://github.com/tokio-rs/bytes/issues/86
///
/// So this enum allows the engine to return whatever it produced and let the caller deal with it
/// the way it wants depending on the HTTP framework used. Not sure of the impact, but it just
/// profoundly annoys me that a full response copy would happen while we put a lot of effort to
/// minimize allocations in the engine.
pub enum OwnedOrSharedBytes {
    Owned(Vec<u8>),
    Shared(Bytes),
}

impl OwnedOrSharedBytes {
    pub fn len(&self) -> usize {
        match self {
            Self::Owned(v) => v.len(),
            Self::Shared(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Owned(v) => v.is_empty(),
            Self::Shared(v) => v.is_empty(),
        }
    }
}

impl std::ops::Deref for OwnedOrSharedBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<[u8]> for OwnedOrSharedBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Owned(v) => v.as_ref(),
            Self::Shared(v) => v.as_ref(),
        }
    }
}

impl From<OwnedOrSharedBytes> for Vec<u8> {
    fn from(value: OwnedOrSharedBytes) -> Self {
        match value {
            OwnedOrSharedBytes::Owned(v) => v,
            OwnedOrSharedBytes::Shared(v) => v.to_vec(),
        }
    }
}

impl From<String> for OwnedOrSharedBytes {
    fn from(v: String) -> Self {
        Self::Owned(v.into_bytes())
    }
}

impl From<Vec<u8>> for OwnedOrSharedBytes {
    fn from(v: Vec<u8>) -> Self {
        Self::Owned(v)
    }
}

impl From<OwnedOrSharedBytes> for Bytes {
    fn from(value: OwnedOrSharedBytes) -> Self {
        match value {
            OwnedOrSharedBytes::Owned(v) => Bytes::from(v),
            OwnedOrSharedBytes::Shared(v) => v,
        }
    }
}

impl From<Bytes> for OwnedOrSharedBytes {
    fn from(v: Bytes) -> Self {
        Self::Shared(v)
    }
}
