use schema::InputValueSerdeError;
use serde::{
    de::{
        IntoDeserializer, Visitor,
        value::{MapDeserializer, SeqDeserializer},
    },
    forward_to_deserialize_any,
};
use walker::Walk;

use crate::VariableInputValueRecord;

use super::{VariableInputValue, VariableValue};

impl<'de> serde::Deserializer<'de> for VariableValue<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            VariableValue::Undefined => visitor.visit_none(),
            VariableValue::Provided(walker) => walker.deserialize_any(visitor),
            VariableValue::DefaultValue(walker) => walker.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            VariableValue::Undefined => visitor.visit_none(),
            VariableValue::Provided(walker) => walker.deserialize_option(visitor),
            VariableValue::DefaultValue(walker) => walker.deserialize_option(visitor),
        }
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

impl<'de> serde::Deserializer<'de> for VariableInputValue<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let VariableInputValue { ctx, ref_: value } = self;
        match value {
            VariableInputValueRecord::Null => visitor.visit_none(),
            VariableInputValueRecord::String(s) => visitor.visit_borrowed_str(s),
            VariableInputValueRecord::EnumValue(id) => visitor.visit_borrowed_str(id.walk(ctx.schema).name()),
            VariableInputValueRecord::Int(n) => visitor.visit_i32(*n),
            VariableInputValueRecord::BigInt(n) => visitor.visit_i64(*n),
            VariableInputValueRecord::U64(n) => visitor.visit_u64(*n),
            VariableInputValueRecord::Float(n) => visitor.visit_f64(*n),
            VariableInputValueRecord::Boolean(b) => visitor.visit_bool(*b),
            VariableInputValueRecord::List(ids) => SeqDeserializer::new(ids.walk(ctx)).deserialize_any(visitor),
            VariableInputValueRecord::InputObject(ids) => MapDeserializer::new(
                ids.walk(ctx)
                    .map(|(input_value_definition, value)| (input_value_definition.name(), value)),
            )
            .deserialize_any(visitor),
            VariableInputValueRecord::Map(ids) => MapDeserializer::new(ids.walk(ctx)).deserialize_any(visitor),
            VariableInputValueRecord::DefaultValue(id) => id.walk(ctx.schema).deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.ref_, VariableInputValueRecord::Null) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for VariableInputValue<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
