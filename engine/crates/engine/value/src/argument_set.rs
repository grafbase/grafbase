//! Defines ArgumentSet - a special kind of Value that can be serialized to JSON while preserving
//! enums and variables.
//!
//! This should not be used for user facing things, but can be used when you want to preserve a set
//! of GraphQL arguments in the registry for example.

use std::fmt::{self, Formatter};

use bytes::Bytes;
use indexmap::IndexMap;
use serde::{
    de::{Error as DeError, MapAccess, SeqAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Number;

use crate::Name;

/// A Serializable set of arguments to a GraphQL field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentSet(Vec<(Name, SerializableArgument)>);

impl ArgumentSet {
    /// Create a new ArgumentSet
    pub fn new(arguments: Vec<(Name, crate::Value)>) -> Self {
        ArgumentSet(
            arguments
                .into_iter()
                .map(|(name, value)| (name, value.into()))
                .collect(),
        )
    }

    /// Checks if the ArgumentSet contains the given argument
    pub fn contains_argument(&self, name: &str) -> bool {
        self.0.iter().any(|(argument_name, _)| argument_name == name)
    }

    /// Returns true if the ArgumentSet is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterator over the names of the arguments
    pub fn iter_names(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(name, _)| name.as_str())
    }

    /// Borrowing iterator
    pub fn iter(&self) -> impl Iterator<Item = (&str, &SerializableArgument)> {
        self.0.iter().map(|(name, arg)| (name.as_str(), arg))
    }
}

impl IntoIterator for ArgumentSet {
    type Item = (Name, crate::Value);

    type IntoIter = ArgumentSetIter;

    fn into_iter(self) -> Self::IntoIter {
        ArgumentSetIter(self.0.into_iter())
    }
}

/// Owned iterator for ArgumentSet
pub struct ArgumentSetIter(<Vec<(Name, SerializableArgument)> as IntoIterator>::IntoIter);

impl Iterator for ArgumentSetIter {
    type Item = (Name, crate::Value);

    fn next(&mut self) -> Option<Self::Item> {
        let (name, argument) = self.0.next()?;
        Some((name, argument.into()))
    }
}

/// A Serializable argument to a GraphQL field
///
/// This is very similar to Value, but it can serialize both variables and enums instead of erroring
/// when it encounters them:
///
/// 1. Variables will be serialized as `{"$var": "X"}`
/// 2. Enums will be serialized ass `{"$enum": "X"}`
///
/// Accordingly, this should only be used in contexts where we control the inputs and such an object can't exist,
/// e.g. in the registry.
///
/// Note that this is private intentionally - this should only really be used for serialization via
/// ArgumentSet above.  Convert it to Value if you want to work with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializableArgument {
    /// A variable, without the `$`.
    Variable(Name),
    /// `null`.
    Null,
    /// A number.
    Number(Number),
    /// A string.
    String(String),
    /// A boolean.
    Boolean(bool),
    /// A binary.
    Binary(Bytes),
    /// An enum. These are typically in `SCREAMING_SNAKE_CASE`.
    Enum(Name),
    /// A list of values.
    List(Vec<SerializableArgument>),
    /// An object. This is a map of keys to values.
    Object(IndexMap<Name, SerializableArgument>),
}

impl From<SerializableArgument> for crate::Value {
    fn from(value: SerializableArgument) -> Self {
        match value {
            SerializableArgument::Variable(name) => crate::Value::Variable(name),
            SerializableArgument::Null => crate::Value::Null,
            SerializableArgument::Number(num) => {
                // We force it to be a f64 in the internal representation to generate the
                // appropriate ArrowSchema
                crate::Value::Number(Number::from_f64(num.as_f64().expect("can't fail")).expect("can't fail"))
            }
            SerializableArgument::String(s) => crate::Value::String(s),
            SerializableArgument::Boolean(b) => crate::Value::Boolean(b),
            SerializableArgument::Binary(bytes) => crate::Value::Binary(bytes),
            SerializableArgument::Enum(v) => crate::Value::Enum(v),
            SerializableArgument::List(items) => crate::Value::List(items.into_iter().map(Into::into).collect()),
            SerializableArgument::Object(map) => {
                crate::Value::Object(map.into_iter().map(|(key, value)| (key, value.into())).collect())
            }
        }
    }
}

impl From<crate::Value> for SerializableArgument {
    fn from(value: crate::Value) -> Self {
        match value {
            crate::Value::Variable(name) => SerializableArgument::Variable(name),
            crate::Value::Null => SerializableArgument::Null,
            crate::Value::Number(num) => {
                // We force it to be a f64 in the internal representation to generate the
                // appropriate ArrowSchema
                SerializableArgument::Number(Number::from_f64(num.as_f64().expect("can't fail")).expect("can't fail"))
            }
            crate::Value::String(s) => SerializableArgument::String(s),
            crate::Value::Boolean(b) => SerializableArgument::Boolean(b),
            crate::Value::Binary(bytes) => SerializableArgument::Binary(bytes),
            crate::Value::Enum(v) => SerializableArgument::Enum(v),
            crate::Value::List(items) => SerializableArgument::List(items.into_iter().map(Into::into).collect()),
            crate::Value::Object(map) => {
                SerializableArgument::Object(map.into_iter().map(|(key, value)| (key, value.into())).collect())
            }
        }
    }
}

impl Serialize for SerializableArgument {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            SerializableArgument::Variable(name) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("$var", name)?;
                map.end()
            }
            SerializableArgument::Null => serializer.serialize_none(),
            SerializableArgument::Number(v) => v.serialize(serializer),
            SerializableArgument::String(v) => serializer.serialize_str(v),
            SerializableArgument::Boolean(v) => serializer.serialize_bool(*v),
            SerializableArgument::Binary(v) => serializer.serialize_bytes(v),
            SerializableArgument::Enum(name) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("$enum", name)?;
                map.end()
            }
            SerializableArgument::List(v) => v.serialize(serializer),
            SerializableArgument::Object(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for SerializableArgument {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = SerializableArgument;

            #[inline]
            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("any valid value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SerializableArgument::Boolean(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SerializableArgument::Number(v.into()))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SerializableArgument::Number(v.into()))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Number::from_f64(v).map_or(SerializableArgument::Null, SerializableArgument::Number))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SerializableArgument::String(v.to_string()))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SerializableArgument::String(v))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SerializableArgument::Binary(v.to_vec().into()))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SerializableArgument::Binary(v.into()))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SerializableArgument::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(SerializableArgument::Null)
            }

            fn visit_seq<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(SerializableArgument::List(vec))
            }

            fn visit_map<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut map = IndexMap::new();
                while let Some((name, value)) = visitor.next_entry()? {
                    map.insert(name, value);
                }
                if map.len() == 1 {
                    if let Some(SerializableArgument::String(value)) = map.get("$var") {
                        return Ok(SerializableArgument::Variable(Name::new(value)));
                    }
                    if let Some(SerializableArgument::String(value)) = map.get("$enum") {
                        return Ok(SerializableArgument::Enum(Name::new(value)));
                    }
                }
                Ok(SerializableArgument::Object(map))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    fn roundtrip_test(input: SerializableArgument) {
        assert_eq!(
            SerializableArgument::deserialize(serde_json::to_value(&input).unwrap()).unwrap(),
            input,
        );
    }

    #[test]
    fn serde_tests() {
        roundtrip_test(SerializableArgument::Enum(Name::new("foo")));
        roundtrip_test(SerializableArgument::Variable(Name::new("bar")));

        roundtrip_test(SerializableArgument::Object(IndexMap::from([
            (Name::new("foo"), SerializableArgument::Boolean(true)),
            (Name::new("bar"), SerializableArgument::Boolean(false)),
        ])));

        roundtrip_test(SerializableArgument::List(vec![
            SerializableArgument::Boolean(true),
            SerializableArgument::Boolean(false),
        ]));

        roundtrip_test(SerializableArgument::Null);
        roundtrip_test(SerializableArgument::String("hello".into()));
        roundtrip_test(SerializableArgument::Number(1.into()));
    }
}
