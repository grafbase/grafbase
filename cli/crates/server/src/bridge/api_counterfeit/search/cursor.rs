use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Cursor(Vec<u8>);

impl Cursor {
    // We could use From<Cursor> but it becomes a bit ambiguous
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<&[u8]> for Cursor {
    fn from(value: &[u8]) -> Self {
        Cursor(value.to_vec())
    }
}

impl From<Vec<u8>> for Cursor {
    fn from(value: Vec<u8>) -> Self {
        Cursor(value)
    }
}

impl TryFrom<String> for Cursor {
    type Error = base64::DecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(value)
            .map(Cursor)
    }
}

impl From<Cursor> for String {
    fn from(cursor: Cursor) -> Self {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(cursor.0)
    }
}
