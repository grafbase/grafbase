use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any,
};

use crate::{InputValueSerdeError, SchemaInputValue, SchemaInputValueWalker};

impl<'de> serde::Deserializer<'de> for SchemaInputValueWalker<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.item {
            SchemaInputValue::Null => visitor.visit_none(),
            SchemaInputValue::String(id) => visitor.visit_borrowed_str(&self.schema[*id]),
            SchemaInputValue::EnumValue(id) => visitor.visit_borrowed_str(self.walk(*id).name()),
            SchemaInputValue::Int(n) => visitor.visit_i32(*n),
            SchemaInputValue::BigInt(n) => visitor.visit_i64(*n),
            SchemaInputValue::U64(n) => visitor.visit_u64(*n),
            SchemaInputValue::Float(n) => visitor.visit_f64(*n),
            SchemaInputValue::Boolean(b) => visitor.visit_bool(*b),
            SchemaInputValue::List(ids) => {
                let mut deserializer = SeqDeserializer::new(self.schema[*ids].iter().map(|value| self.walk(value)));
                let seq = visitor.visit_seq(&mut deserializer)?;
                deserializer.end()?;
                Ok(seq)
            }
            SchemaInputValue::InputObject(ids) => {
                let mut deserializer =
                    MapDeserializer::new(self.schema[*ids].iter().map(|(input_value_definition_id, value)| {
                        (self.walk(*input_value_definition_id).name(), self.walk(value))
                    }));
                let map = visitor.visit_map(&mut deserializer)?;
                deserializer.end()?;
                Ok(map)
            }
            SchemaInputValue::Map(ids) => {
                let mut deserializer = MapDeserializer::new(
                    self.schema[*ids]
                        .iter()
                        .map(|(key, value)| (self.schema[*key].as_str(), self.walk(value))),
                );
                let map = visitor.visit_map(&mut deserializer)?;
                deserializer.end()?;
                Ok(map)
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.item, SchemaInputValue::Null) {
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for SchemaInputValueWalker<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
