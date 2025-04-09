use schema::InputValueSerdeError;
use serde::{
    de::{
        IntoDeserializer, Visitor,
        value::{MapDeserializer, SeqDeserializer},
    },
    forward_to_deserialize_any,
};
use walker::Walk;

use crate::{InputValueContext, QueryInputValueRecord, VariableDefinitionId};

use super::{QueryInputValue, QueryOrSchemaInputValue};

impl<'de> serde::Deserializer<'de> for QueryInputValue<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let QueryInputValue { ctx, ref_: value } = self;
        match value {
            QueryInputValueRecord::Null => visitor.visit_none(),
            QueryInputValueRecord::String(s) | QueryInputValueRecord::UnboundEnumValue(s) => {
                visitor.visit_borrowed_str(s)
            }
            QueryInputValueRecord::EnumValue(id) => visitor.visit_borrowed_str(id.walk(ctx.schema).name()),
            QueryInputValueRecord::Int(n) => visitor.visit_i32(*n),
            QueryInputValueRecord::I64(n) => visitor.visit_i64(*n),
            QueryInputValueRecord::U64(n) => visitor.visit_u64(*n),
            QueryInputValueRecord::Float(n) => visitor.visit_f64(*n),
            QueryInputValueRecord::Boolean(b) => visitor.visit_bool(*b),
            QueryInputValueRecord::List(ids) => SeqDeserializer::new(ids.walk(ctx)).deserialize_any(visitor),
            QueryInputValueRecord::InputObject(ids) => {
                MapDeserializer::new(ids.walk(ctx).filter_map(|(input_value_definition, value)| {
                    if value.is_undefined() {
                        input_value_definition
                            .default_value()
                            .map(|value| (input_value_definition.name(), QueryOrSchemaInputValue::Schema(value)))
                    } else {
                        Some((input_value_definition.name(), QueryOrSchemaInputValue::Query(value)))
                    }
                }))
                .deserialize_any(visitor)
            }
            QueryInputValueRecord::Map(ids) => MapDeserializer::new(ids.walk(ctx)).deserialize_any(visitor),
            QueryInputValueRecord::DefaultValue(id) => id.walk(ctx.schema).deserialize_any(visitor),
            QueryInputValueRecord::Variable(id) => {
                <VariableDefinitionId as Walk<InputValueContext<'de>>>::walk(*id, self.ctx).deserialize_any(visitor)
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.ref_, QueryInputValueRecord::Null) {
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for QueryInputValue<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::Deserializer<'de> for QueryOrSchemaInputValue<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            QueryOrSchemaInputValue::Query(value) => value.deserialize_any(visitor),
            QueryOrSchemaInputValue::Schema(value) => value.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            QueryOrSchemaInputValue::Query(value) => value.deserialize_option(visitor),
            QueryOrSchemaInputValue::Schema(value) => value.deserialize_option(visitor),
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for QueryOrSchemaInputValue<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
