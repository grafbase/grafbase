use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PaginationCursor {
    pub sk: String,
}

impl Serialize for PaginationCursor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Display for PaginationCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // No padding makes it easier to copy paste (without `=`) and just shorter.
        use base64::engine::fast_portable::{FastPortable, NO_PAD};
        let encoded = base64::encode_engine(&self.sk, &FastPortable::from(&base64::alphabet::URL_SAFE, NO_PAD));
        write!(f, "{encoded}")
    }
}

impl PaginationCursor {
    pub fn from_string(value: String) -> anyhow::Result<PaginationCursor> {
        use base64::engine::fast_portable::{FastPortable, NO_PAD};
        let utf8 = base64::decode_engine(value, &FastPortable::from(&base64::alphabet::URL_SAFE, NO_PAD))?;
        let sk = String::from_utf8(utf8)?;
        Ok(PaginationCursor { sk })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_conversion_consistency() {
        let cursor = PaginationCursor {
            sk: String::from("My shiny new cursor"),
        };
        let copy = PaginationCursor::from_string(cursor.to_string());
        assert!(copy.is_ok());
        assert_eq!(copy.unwrap(), cursor);
    }

    #[test]
    fn test_serde() {
        let cursor = PaginationCursor {
            sk: String::from("My shiny new cursor"),
        };
        insta::assert_json_snapshot!(cursor);
    }
}
