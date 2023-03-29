use std::fmt::Display;

use dynaql::Name;
use dynaql_parser::types::ConstDirective;
use dynaql_value::ConstValue;
use serde::de::IntoDeserializer;

/// Parses a ConstDirective into a type that impls Deserialize
pub fn parse_directive<D: serde::de::DeserializeOwned>(directive: &ConstDirective) -> Result<D, Error> {
    D::deserialize(DirectiveDeserializer {
        directive,
        current_index: 0,
    })
}

struct DirectiveDeserializer<'de> {
    directive: &'de ConstDirective,
    current_index: usize,
}

impl<'de> serde::de::Deserializer<'de> for DirectiveDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> serde::de::MapAccess<'de> for DirectiveDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.current_index >= self.directive.arguments.len() {
            return Ok(None);
        }
        return seed
            .deserialize(NameDeserializer(&self.directive.arguments[self.current_index].0.node))
            .map(Some);
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let val = &self.directive.arguments[self.current_index].1;
        self.current_index += 1;
        seed.deserialize(ValueDeserializer(&val.node))
    }
}

struct NameDeserializer<'de>(&'de Name);

impl<'de> serde::de::Deserializer<'de> for NameDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(self.0.as_str())
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ValueDeserializer<'de>(&'de ConstValue);

impl<'de> serde::de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0 {
            ConstValue::Null => visitor.visit_none(),
            ConstValue::Number(num) if num.is_f64() => visitor.visit_f64(num.as_f64().unwrap()),
            ConstValue::Number(num) if num.is_u64() => visitor.visit_u64(num.as_u64().unwrap()),
            ConstValue::Number(num) => visitor.visit_i64(num.as_i64().unwrap()),
            ConstValue::String(s) => visitor.visit_str(s),
            ConstValue::Boolean(b) => visitor.visit_bool(*b),
            ConstValue::Binary(b) => visitor.visit_bytes(b),
            ConstValue::Enum(en) => visitor.visit_enum(en.as_str().into_deserializer()),
            ConstValue::List(v) => visitor.visit_seq(Sequence(v.iter())),
            ConstValue::Object(obj) => visitor.visit_map(Object {
                iter: obj.iter(),
                next_value: None,
            }),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0 {
            ConstValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }
}

struct Sequence<'de>(std::slice::Iter<'de, ConstValue>);

impl<'de> serde::de::SeqAccess<'de> for Sequence<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let Some(val) = self.0.next() else {
                return Ok(None);
            };
        seed.deserialize(ValueDeserializer(val)).map(Some)
    }
}

struct Object<'de> {
    iter: dynaql::indexmap::map::Iter<'de, Name, ConstValue>,
    next_value: Option<&'de ConstValue>,
}

impl<'de> serde::de::MapAccess<'de> for Object<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let Some((key, val)) = self.iter.next() else {
            return Ok(None);
        };
        self.next_value = Some(val);
        seed.deserialize(NameDeserializer(key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = self.next_value.take().unwrap();
        seed.deserialize(ValueDeserializer(value))
    }
}

#[derive(Debug)]
pub enum Error {
    Message(String),
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]

    use dynaql_parser::{parse_schema, types::TypeSystemDefinition};

    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct MyDirective {
        a_list: Vec<String>,
        a_nested_object: MyNestedObject,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct MyNestedObject {
        a_field: String,
    }

    #[test]
    fn test_from_directive() {
        let doc = parse_schema(
            r#"
                extend schema @mydirective(aList: ["hello", "there"], aNestedObject: {aField: "blah"})
            "#,
        )
        .unwrap();
        let TypeSystemDefinition::Schema(schema) = &doc.definitions[0] else {
            panic!("Expected a schema");
        };

        let directive = &schema.node.directives[0];

        insta::assert_debug_snapshot!(parse_directive::<MyDirective>(&directive.node).unwrap(), @r###"
        MyDirective {
            a_list: [
                "hello",
                "there",
            ],
            a_nested_object: MyNestedObject {
                a_field: "blah",
            },
        }
        "###);
    }
}
