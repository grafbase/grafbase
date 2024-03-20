use schema::{InputValueSerdeError, RawInputValuesContext};
use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any,
};

use super::{QueryInputValue, QueryInputValueWalker};

impl<'de> serde::Deserializer<'de> for QueryInputValueWalker<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.item {
            QueryInputValue::Null | QueryInputValue::Undefined => visitor.visit_none(),
            QueryInputValue::String(s) => visitor.visit_borrowed_str(s),
            QueryInputValue::EnumValue(id) => visitor.visit_borrowed_str(self.schema_walker.walk(*id).name()),
            QueryInputValue::Int(n) => visitor.visit_i32(*n),
            QueryInputValue::BigInt(n) => visitor.visit_i64(*n),
            QueryInputValue::U64(n) => visitor.visit_u64(*n),
            QueryInputValue::Float(n) => visitor.visit_f64(*n),
            QueryInputValue::Boolean(b) => visitor.visit_bool(*b),
            QueryInputValue::List(ids) => {
                let mut deserializer = SeqDeserializer::new(ids.map(move |id| self.walk(&self.operation[id])));
                let seq = visitor.visit_seq(&mut deserializer)?;
                deserializer.end()?;
                Ok(seq)
            }
            QueryInputValue::InputObject(ids) => {
                let mut deserializer = MapDeserializer::new(ids.filter_map(move |id| {
                    let (input_value_definition_id, value) = &self.operation[id];
                    let value = self.walk(value);
                    if value.is_undefined() {
                        None
                    } else {
                        Some((self.schema_walker.walk(*input_value_definition_id).name(), value))
                    }
                }));
                let map = visitor.visit_map(&mut deserializer)?;
                deserializer.end()?;
                Ok(map)
            }
            QueryInputValue::Map(ids) => {
                let mut deserializer = MapDeserializer::new(ids.map(move |id| {
                    let (key, value) = &self.operation[id];
                    let value = self.walk(value);
                    (key.as_str(), value)
                }));
                let map = visitor.visit_map(&mut deserializer)?;
                deserializer.end()?;
                Ok(map)
            }
            QueryInputValue::DefaultValue(id) => {
                RawInputValuesContext::walk(&self.schema_walker, *id).deserialize_any(visitor)
            }
            QueryInputValue::Variable(id) => self.walk(*id).deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.item, QueryInputValue::Null) {
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for QueryInputValueWalker<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
