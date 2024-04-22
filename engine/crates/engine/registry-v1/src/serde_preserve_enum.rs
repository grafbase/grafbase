//! Custom serde functions for ConstValue that preserve Enums.
//!
//! The default impl will change them to strings which is often fine
//! but very bad for default_value above

use engine_value::{ConstValue, Name};
use indexmap::IndexMap;
use serde::{
    de::{self, IntoDeserializer},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};

pub fn serialize<S>(value: &Option<ConstValue>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.as_ref().map(BorrowedConstValueWrapper).serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ConstValue>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<OwnedConstValueWrapper>::deserialize(deserializer)?.map(|value| value.0))
}

/// A wrapper around ConstValue that serializes enums "safely"
struct BorrowedConstValueWrapper<'a>(&'a ConstValue);

impl serde::Serialize for BorrowedConstValueWrapper<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // We wrap enums in an object so we can preserve them.  Because of this
        // we also need to wrap objects so things aren't ambiguous.
        match self.0 {
            ConstValue::Enum(name) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("__enum", name.as_str())?;
                map.end()
            }

            ConstValue::List(items) => serializer.collect_seq(items.iter().map(BorrowedConstValueWrapper)),
            ConstValue::Object(object) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(
                    "__object",
                    &object
                        .iter()
                        .map(|(name, value)| (name, BorrowedConstValueWrapper(value)))
                        .collect::<IndexMap<_, _>>(),
                )?;
                map.end()
            }
            other => other.serialize(serializer),
        }
    }
}

macro_rules! forward_to_const_value {
    ($name:ident, $ty:ty) => {
        fn $name<E>(self, v: $ty) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            ConstValue::deserialize(v.into_deserializer()).map(OwnedConstValueWrapper)
        }
    };
}

/// A wrapper around ConstValue that deserializes enums "safely"
struct OwnedConstValueWrapper(ConstValue);

impl<'de> serde::de::Deserialize<'de> for OwnedConstValueWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = OwnedConstValueWrapper;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a const value")
            }

            forward_to_const_value!(visit_bool, bool);
            forward_to_const_value!(visit_i8, i8);
            forward_to_const_value!(visit_i16, i16);
            forward_to_const_value!(visit_i32, i32);
            forward_to_const_value!(visit_i64, i64);
            forward_to_const_value!(visit_i128, i128);
            forward_to_const_value!(visit_u8, u8);
            forward_to_const_value!(visit_u16, u16);
            forward_to_const_value!(visit_u32, u32);
            forward_to_const_value!(visit_u64, u64);
            forward_to_const_value!(visit_u128, u128);
            forward_to_const_value!(visit_f32, f32);
            forward_to_const_value!(visit_f64, f64);
            forward_to_const_value!(visit_char, char);
            forward_to_const_value!(visit_str, &str);
            forward_to_const_value!(visit_borrowed_str, &'de str);
            forward_to_const_value!(visit_string, String);
            forward_to_const_value!(visit_bytes, &[u8]);
            forward_to_const_value!(visit_borrowed_bytes, &'de [u8]);
            forward_to_const_value!(visit_byte_buf, Vec<u8>);

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(OwnedConstValueWrapper(ConstValue::Null))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(Visitor)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                ConstValue::deserialize(().into_deserializer()).map(OwnedConstValueWrapper)
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                ConstValue::deserialize(deserializer).map(OwnedConstValueWrapper)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut output = Vec::new();
                while let Some(value) = seq.next_element::<OwnedConstValueWrapper>()? {
                    output.push(value.0);
                }
                Ok(OwnedConstValueWrapper(ConstValue::List(output)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let Some((key, value)) = map.next_entry::<String, OwnedConstValueWrapper>()? else {
                    return Ok(OwnedConstValueWrapper(ConstValue::Object(IndexMap::new())));
                };

                match (key.as_str(), value) {
                    ("__enum", OwnedConstValueWrapper(ConstValue::String(string))) => {
                        Ok(OwnedConstValueWrapper(ConstValue::Enum(Name::new(string))))
                    }
                    ("__object", value @ OwnedConstValueWrapper(ConstValue::Object(_))) => Ok(value),
                    (key, value) => {
                        let mut object = IndexMap::new();
                        object.insert(Name::new(key), value.0);
                        while let Some((key, value)) = map.next_entry::<Name, OwnedConstValueWrapper>()? {
                            object.insert(key, value.0);
                        }

                        Ok(OwnedConstValueWrapper(ConstValue::Object(object)))
                    }
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::MetaInputValue;

    fn run_test(input: ConstValue) {
        let output: OwnedConstValueWrapper =
            serde_json::from_value(serde_json::to_value(BorrowedConstValueWrapper(&input)).unwrap()).unwrap();
        assert_eq!(output.0, input);
    }

    #[test]
    fn test_roundtrip() {
        run_test(ConstValue::Boolean(true));
        run_test(ConstValue::Boolean(false));
        run_test(ConstValue::Enum(Name::new("HELLO")));
        run_test(ConstValue::Number(123.into()));

        run_test(ConstValue::String("hello".into()));

        run_test(ConstValue::Object(
            [
                (Name::new("null"), ConstValue::Null),
                (Name::new("enum"), ConstValue::Enum(Name::new("HELLO"))),
                (Name::new("bool"), ConstValue::Boolean(false)),
            ]
            .into(),
        ));

        run_test(ConstValue::List(vec![
            ConstValue::Null,
            ConstValue::Enum(Name::new("HELLO")),
            ConstValue::Boolean(true),
        ]));
    }

    #[test]
    fn test_enum() {
        let value = ConstValue::Enum(Name::new("HELLO"));
        assert_eq!(
            serde_json::to_value(BorrowedConstValueWrapper(&value)).unwrap(),
            json!({
                "__enum": "HELLO"
            })
        );
    }

    #[test]
    fn test_object() {
        let value = ConstValue::Object([(Name::new("hello"), ConstValue::Null)].into());
        assert_eq!(
            serde_json::to_value(BorrowedConstValueWrapper(&value)).unwrap(),
            json!({
                "__object": {"hello": null}
            })
        );
    }

    #[test]
    fn test_deser_plain_object() {
        let value = ConstValue::Object([(Name::new("hello"), ConstValue::Null)].into());
        assert_eq!(
            serde_json::from_value::<OwnedConstValueWrapper>(serde_json::to_value(&value).unwrap())
                .unwrap()
                .0,
            value
        );
    }

    #[test]
    fn test_input_default_value() {
        let input = MetaInputValue {
            default_value: Some(ConstValue::Enum(Name::new("A_VALUE"))),
            ..MetaInputValue::new("someEnum", "AnEnum")
        };
        let output = serde_json::from_value::<MetaInputValue>(serde_json::to_value(&input).unwrap()).unwrap();

        assert_eq!(input.default_value, output.default_value);
    }
}
