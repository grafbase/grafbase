use grafbase_workspace_hack as _;

use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, UrlSafe},
    formats::Unpadded,
    serde_as,
};

// Should be in some common library instead.
#[serde_as]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphqlCursor(#[serde_as(as = "Base64<UrlSafe, Unpadded>")] Vec<u8>);

impl GraphqlCursor {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl<T: AsRef<[u8]>> From<T> for GraphqlCursor {
    fn from(value: T) -> Self {
        GraphqlCursor(value.as_ref().to_vec())
    }
}
