use readable::Readable;
use schema::InputValueSerdeError;
use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any,
};

use super::{VariableInputValue, VariableInputValueWalker};

impl<'de> serde::Deserializer<'de> for VariableInputValueWalker<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.item {
            VariableInputValue::Null => visitor.visit_none(),
            VariableInputValue::String(s) => visitor.visit_borrowed_str(s),
            VariableInputValue::EnumValue(id) => visitor.visit_borrowed_str(self.schema.walk(*id).name()),
            VariableInputValue::Int(n) => visitor.visit_i32(*n),
            VariableInputValue::BigInt(n) => visitor.visit_i64(*n),
            VariableInputValue::U64(n) => visitor.visit_u64(*n),
            VariableInputValue::Float(n) => visitor.visit_f64(*n),
            VariableInputValue::Boolean(b) => visitor.visit_bool(*b),
            VariableInputValue::List(ids) => {
                SeqDeserializer::new(self.variables[*ids].iter().map(|value| self.walk(value))).deserialize_any(visitor)
            }
            VariableInputValue::InputObject(ids) => {
                MapDeserializer::new(self.variables[*ids].iter().map(|(input_value_definition_id, value)| {
                    (self.schema.walk(*input_value_definition_id).name(), self.walk(value))
                }))
                .deserialize_any(visitor)
            }
            VariableInputValue::Map(ids) => MapDeserializer::new(
                self.variables[*ids]
                    .iter()
                    .map(|(key, value)| (key.as_str(), self.walk(value))),
            )
            .deserialize_any(visitor),
            VariableInputValue::DefaultValue(id) => id.read(self.schema).deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.item, VariableInputValue::Null) {
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for VariableInputValueWalker<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
