use std::{borrow::Cow, marker::PhantomData};

use serde::{Deserialize, Deserializer, de::Visitor, forward_to_deserialize_any};

/// Needed for websocket de-serialization which pre-allocates a serde_json::Value...
#[derive(Deserialize, PartialEq, Debug)]
pub(crate) struct Key<'a>(#[serde(borrow)] Cow<'a, str>);

impl AsRef<str> for Key<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> Key<'a> {
    pub fn into_deserializer<E: serde::de::Error>(self) -> KeyDeserializer<'a, E> {
        KeyDeserializer {
            key: self,
            _phantom: PhantomData,
        }
    }
}

pub(crate) struct KeyDeserializer<'de, Error> {
    key: Key<'de>,
    _phantom: PhantomData<Error>,
}

impl<'de, Error: serde::de::Error> Deserializer<'de> for KeyDeserializer<'de, Error> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.key.0 {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_string(s),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct enum identifier str
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_borrows() {
        assert_eq!(
            sonic_rs::from_str::<Key<'_>>("\"hello\"").unwrap(),
            Key(Cow::Borrowed("hello"))
        );
    }
}
