use std::{collections::HashMap, fmt::Display};

use engine::{Name, Pos};
use engine_parser::types::ConstDirective;
use engine_value::ConstValue;
use serde::de::{Error as _, IntoDeserializer};

use crate::{dynamic_string::DynamicString, rules::visitor::RuleError};

/// Parses a ConstDirective into a type that impls Deserialize
///
/// This will automatically interpolate environment variables into any type that deserializes
/// as a String (but not neccesarily any string that is present in the SDL)
pub fn parse_directive<D: serde::de::DeserializeOwned>(
    directive: &ConstDirective,
    environment_variables: &HashMap<String, String>,
) -> Result<D, RuleError> {
    D::deserialize(DirectiveDeserializer {
        directive,
        current_index: 0,
        environment_variables,
    })
    .map_err(|Error::Message(msg, pos)| RuleError::new(pos.into_iter().collect(), msg))
}

struct DirectiveDeserializer<'de> {
    environment_variables: &'de HashMap<String, String>,
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
            .map(Some)
            .map_err(|Error::Message(msg, pos)| {
                Error::Message(msg, pos.or(Some(self.directive.arguments[self.current_index].0.pos)))
            });
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = &self.directive.arguments[self.current_index].1;
        let current_pos = self.directive.arguments[self.current_index].1.pos;
        self.current_index += 1;

        seed.deserialize(ValueDeserializer::new(&value.node, self.environment_variables))
            .map_err(|err| Error::Message(err.to_string(), Some(current_pos)))
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

struct ValueDeserializer<'de> {
    value: &'de ConstValue,
    environment_variables: &'de HashMap<String, String>,
}

impl<'de> ValueDeserializer<'de> {
    fn new(value: &'de ConstValue, environment_variables: &'de HashMap<String, String>) -> Self {
        ValueDeserializer {
            value,
            environment_variables,
        }
    }
}

impl<'de> serde::de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            ConstValue::Null => visitor.visit_none(),
            ConstValue::Number(num) if num.is_f64() => visitor.visit_f64(num.as_f64().unwrap()),
            ConstValue::Number(num) if num.is_u64() => visitor.visit_u64(num.as_u64().unwrap()),
            ConstValue::Number(num) => visitor.visit_i64(num.as_i64().unwrap()),
            ConstValue::String(_) => self.deserialize_str(visitor),
            ConstValue::Boolean(b) => visitor.visit_bool(*b),
            ConstValue::Binary(b) => visitor.visit_bytes(b),
            ConstValue::Enum(en) => visitor.visit_str(en.as_str()), // Internally tagged.
            ConstValue::List(v) => visitor.visit_seq(Sequence {
                values: v.iter(),
                environment_variables: self.environment_variables,
            }),
            ConstValue::Object(obj) => visitor.visit_map(Object {
                iter: obj.iter(),
                next_value: None,
                environment_variables: self.environment_variables,
            }),
        }
    }

    // Externally tagged.
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            ConstValue::Enum(en) => visitor.visit_enum(en.as_str().into_deserializer()),
            ConstValue::String(str) => {
                // Technically we're not meant to deserialize strings as enums in GraphQL.
                // But some values we use as enum keys (e.g. "2.3") aren't valid as GraphQL enums
                // so I'm going to ignore that rule and just go for it here.
                visitor.visit_enum(str.as_str().into_deserializer())
            }
            _ => Err(Error::custom(format!(
                "attempted to deserialize {:?} as enum '{name}'",
                self.value
            ))),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            ConstValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            ConstValue::String(s) => {
                let mut dynamic_string = s.parse::<DynamicString>()?;
                dynamic_string.partially_evaluate(self.environment_variables)?;
                match dynamic_string.into_fully_evaluated_str() {
                    Some(evaluated_string) => visitor.visit_string(evaluated_string),
                    None => {
                        // Pretty sure this shouldn't happen at the moment, but if we add
                        // any runtime variable support it might.  Should probably change
                        // this to return a partially evaluated string if that happens.
                        //
                        // For now I'm just going to return an error to users.
                        Err(Error::custom(
                            "DynamicString was not fully evaluated.  Please contact support",
                        ))
                    }
                }
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
}

struct Sequence<'de> {
    values: std::slice::Iter<'de, ConstValue>,
    environment_variables: &'de HashMap<String, String>,
}

impl<'de> serde::de::SeqAccess<'de> for Sequence<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let Some(value) = self.values.next() else {
            return Ok(None);
        };
        seed.deserialize(ValueDeserializer::new(value, self.environment_variables))
            .map(Some)
    }
}

struct Object<'de> {
    iter: engine::indexmap::map::Iter<'de, Name, ConstValue>,
    next_value: Option<&'de ConstValue>,
    environment_variables: &'de HashMap<String, String>,
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
        seed.deserialize(ValueDeserializer::new(value, self.environment_variables))
    }
}

#[derive(Debug)]
enum Error {
    Message(String, Option<Pos>),
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string(), None)
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg, _) => formatter.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<engine::ServerError> for Error {
    fn from(value: engine::ServerError) -> Self {
        Error::Message(value.message, value.locations.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]

    use engine_parser::{parse_schema, types::TypeSystemDefinition};

    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct MyDirective {
        a_list: Vec<String>,
        a_nested_object: Option<MyNestedObject>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct MyNestedObject {
        a_field: String,
    }

    fn directive_test<Directive>(
        schema: &str,
        environment_variables: &HashMap<String, String>,
    ) -> Result<Directive, RuleError>
    where
        Directive: serde::de::DeserializeOwned,
    {
        let doc = parse_schema(schema).unwrap();

        let TypeSystemDefinition::Schema(schema) = &doc.definitions[0] else {
            panic!("Expected a schema");
        };

        parse_directive(&schema.node.directives[0].node, environment_variables)
    }

    #[test]
    fn test_from_directive() {
        insta::assert_debug_snapshot!(
            directive_test::<MyDirective>(
                r#"
                    extend schema @mydirective(
                        aList: ["hello", "there", "Bearer {{ env.BLAH }}"],
                        aNestedObject: {aField: "blah"},
                    )
                "#,
                &maplit::hashmap!{"BLAH".to_string() => "OH_LOOK_AN_ENV_VAR".to_string()}
            ).unwrap(),
            @r###"
        MyDirective {
            a_list: [
                "hello",
                "there",
                "Bearer OH_LOOK_AN_ENV_VAR",
            ],
            a_nested_object: Some(
                MyNestedObject {
                    a_field: "blah",
                },
            ),
        }
        "###);
    }

    #[test]
    fn test_missing_env_var() {
        insta::assert_snapshot!(
            directive_test::<MyDirective>(
                r#"
                    extend schema @mydirective(
                        aList: ["Bearer {{ env.BLAH }}"],
                    )
                "#,
                &HashMap::default()
            )
            .unwrap_err().to_string(),
            @"[3:32] undefined variable `BLAH`"
        );
    }
}
