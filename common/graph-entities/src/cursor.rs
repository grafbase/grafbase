use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PaginationCursor {
    pub id: String,
}

impl Serialize for PaginationCursor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PaginationCursor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self {
            id: String::deserialize(deserializer)?,
        })
    }
}

impl Display for PaginationCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use base64::Engine;
        // No padding makes it easier to copy paste (without `=`) and just shorter.
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&self.id);
        write!(f, "{encoded}")
    }
}

impl PaginationCursor {
    pub fn encode(value: &str) -> String {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value)
    }

    pub fn from_string(value: String) -> Result<PaginationCursor, base64::DecodeError> {
        use base64::Engine;
        let utf8 = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(value)?;
        // TODO: FIXME
        let id = String::from_utf8(utf8).unwrap();
        Ok(PaginationCursor { id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_conversion_consistency() {
        let cursor = PaginationCursor {
            id: String::from("My shiny new cursor"),
        };
        let copy = PaginationCursor::from_string(cursor.to_string());
        assert!(copy.is_ok());
        assert_eq!(copy.unwrap(), cursor);
    }

    /*
    #[test]
    fn test_serde() {
        let cursor = PaginationCursor {
            id: String::from("My shiny new cursor"),
        };
        insta::assert_json_snapshot!(cursor);
    }
    */
}
