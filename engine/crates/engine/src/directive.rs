use engine_parser::{types::Directive, Pos};
use engine_value::{Name, Variables};

use crate::ServerError;

pub struct DirectiveDeserializer<'a> {
    directive: &'a Directive,
    variables: &'a Variables,
    current_index: usize,
}

impl<'a> DirectiveDeserializer<'a> {
    pub fn new(directive: &'a Directive, variables: &'a Variables) -> Self {
        DirectiveDeserializer {
            directive,
            variables,
            current_index: 0,
        }
    }
}

impl<'a> serde::de::Deserializer<'static> for DirectiveDeserializer<'a> {
    type Error = DirectiveParseError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'static>,
    {
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        <V: Visitor<'static>>
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'a> serde::de::MapAccess<'static> for DirectiveDeserializer<'a> {
    type Error = DirectiveParseError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'static>,
    {
        if self.current_index >= self.directive.arguments.len() {
            return Ok(None);
        }
        let name = &self.directive.arguments[self.current_index].0.node;

        return seed.deserialize(NameDeserializer(name)).map(Some).map_err(|error| {
            DirectiveParseError::ParsingName(error.to_string(), self.directive.arguments[self.current_index].0.pos)
        });
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'static>,
    {
        let name = &self.directive.arguments[self.current_index].0.node;
        let value = &self.directive.arguments[self.current_index].1;
        let current_pos = self.directive.arguments[self.current_index].1.pos;
        self.current_index += 1;

        seed.deserialize(value.node.clone().into_const_with(|variable_name| {
            self.variables
                .get(&variable_name)
                .cloned()
                .ok_or_else(|| DirectiveParseError::UnknownVariable(variable_name, current_pos))
        })?)
        .map_err(|err| DirectiveParseError::MalformedArgumeent(name.clone(), err.to_string(), current_pos))
    }
}

struct NameDeserializer<'de>(&'de Name);

impl<'a> serde::de::Deserializer<'static> for NameDeserializer<'a> {
    type Error = engine_value::DeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'static>,
    {
        visitor.visit_str(self.0.as_str())
    }

    serde::forward_to_deserialize_any! {
        <V: Visitor<'static>>
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DirectiveParseError {
    #[error("{0}")]
    ParsingName(String, Pos),
    #[error("unknown variable {0}")]
    UnknownVariable(Name, Pos),
    #[error("{1} for the argument `{0}`")]
    MalformedArgumeent(Name, String, Pos),
    #[error("unexpected error parsing directive: {0}")]
    Unknown(String),
}

impl DirectiveParseError {
    pub fn into_server_error(self, directive_name: &str, directive_pos: Pos) -> ServerError {
        let pos = match self {
            DirectiveParseError::ParsingName(_, pos) => pos,
            DirectiveParseError::UnknownVariable(_, pos) => pos,
            DirectiveParseError::MalformedArgumeent(_, _, pos) => pos,
            DirectiveParseError::Unknown(_) => directive_pos,
        };

        ServerError::new(format!("Error interpreting @{directive_name}: {self}"), Some(pos))
    }
}

impl serde::de::Error for DirectiveParseError {
    #[inline]
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DirectiveParseError::Unknown(msg.to_string())
    }
}
