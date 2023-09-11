use std::fmt::{self, Formatter};

use engine_value::Number;
use serde::{
    de::{Error as DeError, MapAccess, SeqAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};

use super::CompactValue;

impl Serialize for CompactValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            CompactValue::Null => serializer.serialize_none(),
            CompactValue::Number(v) => v.serialize(serializer),
            CompactValue::String(v) => serializer.serialize_str(v),
            CompactValue::Boolean(v) => serializer.serialize_bool(*v),
            CompactValue::Binary(v) => serializer.serialize_bytes(v),
            CompactValue::Enum(v) => serializer.serialize_str(v),
            CompactValue::List(v) => v.serialize(serializer),
            CompactValue::Object(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;
                for (key, value) in v {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for CompactValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = CompactValue;

            #[inline]
            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("any valid value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(CompactValue::Boolean(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(CompactValue::Number(v.into()))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(CompactValue::Number(v.into()))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Number::from_f64(v).map_or(CompactValue::Null, CompactValue::Number))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(CompactValue::String(v.to_string()))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(CompactValue::String(v))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(CompactValue::Binary(v.to_vec()))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(CompactValue::Binary(v))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(CompactValue::Null)
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
                Ok(CompactValue::Null)
            }

            fn visit_seq<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(CompactValue::List(vec))
            }

            fn visit_map<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut pairs = Vec::new();
                while let Some((name, value)) = visitor.next_entry()? {
                    pairs.push((name, value));
                }
                Ok(CompactValue::Object(pairs))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
