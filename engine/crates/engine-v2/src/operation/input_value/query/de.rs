use schema::InputValueSerdeError;
use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any,
};
use walker::Walk;

use super::{QueryInputValue, QueryInputValueWalker};

impl<'de> serde::Deserializer<'de> for QueryInputValueWalker<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let input_values = &self.operation.query_input_values;
        match self.item {
            QueryInputValue::Null => visitor.visit_none(),
            QueryInputValue::String(s) | QueryInputValue::UnboundEnumValue(s) => visitor.visit_borrowed_str(s),
            QueryInputValue::EnumValue(id) => visitor.visit_borrowed_str(self.schema.walk(*id).name()),
            QueryInputValue::Int(n) => visitor.visit_i32(*n),
            QueryInputValue::BigInt(n) => visitor.visit_i64(*n),
            QueryInputValue::U64(n) => visitor.visit_u64(*n),
            QueryInputValue::Float(n) => visitor.visit_f64(*n),
            QueryInputValue::Boolean(b) => visitor.visit_bool(*b),
            QueryInputValue::List(ids) => {
                SeqDeserializer::new(input_values[*ids].iter().map(|value| self.walk(value))).deserialize_any(visitor)
            }
            QueryInputValue::InputObject(ids) => {
                MapDeserializer::new(
                    input_values[*ids]
                        .iter()
                        .filter_map(|(input_value_definition_id, value)| {
                            let value = self.walk(value);
                            // https://spec.graphql.org/October2021/#sec-Input-Objects.Input-Coercion
                            if value.is_undefined() {
                                None
                            } else {
                                Some((self.schema.walk(*input_value_definition_id).name(), value))
                            }
                        }),
                )
                .deserialize_any(visitor)
            }
            QueryInputValue::Map(ids) => MapDeserializer::new(
                input_values[*ids]
                    .iter()
                    .map(|(key, value)| (key.as_str(), self.walk(value))),
            )
            .deserialize_any(visitor),
            QueryInputValue::DefaultValue(id) => id.walk(self.schema).deserialize_any(visitor),
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
