use schema::InputValueSerdeError;
use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any,
};

use crate::response::{value::NULL, ResponseListId, ResponseObjectId, ResponseValue};

use super::{OldResponseObjectView, OldResponseObjectsView, ResponseValueWalker};

impl<'de> serde::Deserializer<'de> for OldResponseObjectsView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        SeqDeserializer::new(self.into_iter()).deserialize_any(visitor)
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

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }
}

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for OldResponseObjectsView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::Deserializer<'de> for OldResponseObjectView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        MapDeserializer::new(self.ctx.response_views[self.selection_set].iter().map(|selection| {
            let key = self.ctx.schema[selection.name].as_str();
            let value = ResponseValueWalker {
                ctx: self.ctx,
                value: self.response_object.find_required_field(selection.id).unwrap_or(&NULL),
                selection_set: selection.subselection,
            };
            (key, value)
        }))
        .deserialize_any(visitor)
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

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }
}

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for OldResponseObjectView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::Deserializer<'de> for ResponseValueWalker<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ResponseValue::Null => visitor.visit_none(),
            ResponseValue::Boolean { value, .. } => visitor.visit_bool(*value),
            ResponseValue::Int { value, .. } => visitor.visit_i32(*value),
            ResponseValue::BigInt { value, .. } => visitor.visit_i64(*value),
            ResponseValue::Float { value, .. } => visitor.visit_f64(*value),
            ResponseValue::String { value, .. } => visitor.visit_borrowed_str(value),
            ResponseValue::StringId { id, .. } => visitor.visit_borrowed_str(&self.ctx.schema[*id]),
            ResponseValue::Json { value, .. } => value
                .as_ref()
                .deserialize_any(visitor)
                .map_err(|err| InputValueSerdeError::Message(err.to_string())),
            &ResponseValue::List {
                part_id,
                offset,
                length,
                ..
            } => {
                let values = &self.ctx.response[ResponseListId {
                    part_id,
                    offset,
                    length,
                }];

                SeqDeserializer::new(values.iter().map(|value| ResponseValueWalker {
                    ctx: self.ctx,
                    value,
                    selection_set: self.selection_set,
                }))
                .deserialize_any(visitor)
            }
            &ResponseValue::Object { part_id, index, .. } => OldResponseObjectView {
                ctx: self.ctx,
                response_object: &self.ctx.response[ResponseObjectId { part_id, index }],
                selection_set: self.selection_set,
            }
            .deserialize_any(visitor),
        }
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

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }
}

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for ResponseValueWalker<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
