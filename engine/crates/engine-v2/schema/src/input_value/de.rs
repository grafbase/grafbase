use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any,
};

use crate::{InputValueSerdeError, RawInputValue, RawInputValueWalker, RawInputValuesContext};

impl<'de, Ctx> serde::Deserializer<'de> for RawInputValueWalker<'de, Ctx>
where
    Ctx: RawInputValuesContext<'de>,
{
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            RawInputValue::Null | RawInputValue::Undefined => visitor.visit_none(),
            RawInputValue::String(s) | RawInputValue::UnknownEnumValue(s) => {
                visitor.visit_borrowed_str(self.ctx.get_str(s))
            }
            RawInputValue::EnumValue(id) => visitor.visit_borrowed_str(self.ctx.schema_walker().walk(*id).name()),
            RawInputValue::Int(n) => visitor.visit_i32(*n),
            RawInputValue::BigInt(n) => visitor.visit_i64(*n),
            RawInputValue::U64(n) => visitor.visit_u64(*n),
            RawInputValue::Float(n) => visitor.visit_f64(*n),
            RawInputValue::Boolean(b) => visitor.visit_bool(*b),
            RawInputValue::List(ids) => {
                let mut deserializer = SeqDeserializer::new(ids.map(move |id| self.ctx.walk(id)));
                let seq = visitor.visit_seq(&mut deserializer)?;
                deserializer.end()?;
                Ok(seq)
            }
            RawInputValue::InputObject(ids) => {
                let mut deserializer = MapDeserializer::new(ids.filter_map(move |id| {
                    let (input_value_definition_id, value) = &self.ctx.input_values()[id];
                    let value = self.walk(value);
                    if value.is_undefined() {
                        None
                    } else {
                        Some((self.ctx.schema_walker().walk(*input_value_definition_id).name(), value))
                    }
                }));
                let map = visitor.visit_map(&mut deserializer)?;
                deserializer.end()?;
                Ok(map)
            }
            RawInputValue::Map(ids) => {
                let mut deserializer = MapDeserializer::new(ids.filter_map(move |id| {
                    let (key, value) = &self.ctx.input_values()[id];
                    let value = self.walk(value);
                    if value.is_undefined() {
                        None
                    } else {
                        Some((self.ctx.get_str(key), value))
                    }
                }));
                let map = visitor.visit_map(&mut deserializer)?;
                deserializer.end()?;
                Ok(map)
            }
            RawInputValue::Ref(id) => self.ctx.walk(*id).deserialize_any(visitor),
            RawInputValue::SchemaRef(id) => self.ctx.schema_walk(*id).deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.value, RawInputValue::Null | RawInputValue::Undefined) {
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

impl<'de, Ctx> IntoDeserializer<'de, InputValueSerdeError> for RawInputValueWalker<'de, Ctx>
where
    Ctx: RawInputValuesContext<'de>,
{
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
