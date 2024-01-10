use std::borrow::Cow;

use serde::Deserialize;

/// Type for deserializing arbitrary keys in JSON
///
/// We use this to avoid allocating a string where we don't have to, while still
/// keeping support for owned deserialization where it makes sense.
///
/// Afaik we can't just use `Cow` directly as that always does owned
/// deserializaiton
#[derive(Deserialize, PartialEq, Debug)]
pub struct Key<'a>(#[serde(borrow)] Cow<'a, str>);

impl Key<'_> {
    pub fn into_string(self) -> String {
        self.0.into_owned()
    }
}

impl AsRef<str> for Key<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_borrows() {
        assert_eq!(
            serde_json::from_str::<Key<'_>>("\"hello\"").unwrap(),
            Key(Cow::Borrowed("hello"))
        );
    }
}