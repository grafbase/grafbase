use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any,
};

use crate::{InputValue, InputValueSerdeError, InputValuesContext};

pub(super) struct InputValueDeserializer<'de, Str, Ctx> {
    pub ctx: Ctx,
    pub value: &'de InputValue<Str>,
}

impl<'de, Str, Ctx> serde::Deserializer<'de> for InputValueDeserializer<'de, Str, Ctx>
where
    Ctx: InputValuesContext<'de, Str>,
    Str: 'de,
{
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            InputValue::Null => visitor.visit_unit(),
            InputValue::String(s) | InputValue::UnknownEnumValue(s) => visitor.visit_borrowed_str(self.ctx.get_str(s)),
            InputValue::EnumValue(id) => visitor.visit_borrowed_str(self.ctx.schema_walker().walk(*id).name()),
            InputValue::Int(n) => visitor.visit_i32(*n),
            InputValue::BigInt(n) => visitor.visit_i64(*n),
            InputValue::U64(n) => visitor.visit_u64(*n),
            InputValue::Float(n) => visitor.visit_f64(*n),
            InputValue::Boolean(b) => visitor.visit_bool(*b),
            &InputValue::List(ids) => {
                let ctx = self.ctx;
                let mut deserializer = SeqDeserializer::new(ids.iter().map(move |id| InputValueDeserializer {
                    value: &ctx.input_values()[id],
                    ctx,
                }));
                let seq = visitor.visit_seq(&mut deserializer)?;
                deserializer.end()?;
                Ok(seq)
            }
            InputValue::InputObject(ids) => {
                let ctx = self.ctx;
                let mut deserializer = MapDeserializer::new(ids.iter().map(move |id| {
                    let (id, value) = &ctx.input_values()[id];
                    (
                        ctx.schema_walker().walk(*id).name(),
                        InputValueDeserializer { value, ctx },
                    )
                }));
                let map = visitor.visit_map(&mut deserializer)?;
                deserializer.end()?;
                Ok(map)
            }
            InputValue::Map(ids) => {
                let ctx = self.ctx;
                let mut deserializer = MapDeserializer::new(ids.iter().map(move |id| {
                    let (key, value) = &ctx.input_values()[id];
                    (self.ctx.get_str(key), InputValueDeserializer { value, ctx })
                }));
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
        if matches!(self.value, InputValue::Null) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, Str, Ctx> IntoDeserializer<'de, InputValueSerdeError> for InputValueDeserializer<'de, Str, Ctx>
where
    Ctx: InputValuesContext<'de, Str>,
    Str: 'de,
{
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
