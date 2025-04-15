use cynic_parser::ConstValue;
use serde::de::{
    Deserializer, IntoDeserializer,
    value::{MapDeserializer, SeqDeserializer},
};

pub(crate) struct ConstValueArgumentsDeserializer<'a>(pub Option<ConstValue<'a>>);

impl<'de> serde::Deserializer<'de> for ConstValueArgumentsDeserializer<'de> {
    type Error = ConstValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if let Some(fields) = self.0.and_then(|value| value.as_fields()) {
            MapDeserializer::new(fields.map(|field| (field.name(), ConstValueDeserializer(field.value()))))
                .deserialize_any(visitor)
        } else {
            serde_json::json!({})
                .deserialize_any(visitor)
                .map_err(|err| ConstValueDeserializerError::Message(err.to_string()))
        }
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier option
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

impl<'de> IntoDeserializer<'de, ConstValueDeserializerError> for ConstValueArgumentsDeserializer<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

pub(crate) struct ConstValueDeserializer<'a>(pub ConstValue<'a>);

impl<'de> Deserializer<'de> for ConstValueDeserializer<'de> {
    type Error = ConstValueDeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0 {
            ConstValue::Int(int_value) => visitor.visit_i64(int_value.value()),
            ConstValue::Float(float_value) => visitor.visit_f64(float_value.value()),
            ConstValue::String(string_value) => visitor.visit_borrowed_str(string_value.value()),
            ConstValue::Boolean(boolean_value) => visitor.visit_bool(boolean_value.value()),
            ConstValue::Null(_) => visitor.visit_none(),
            ConstValue::Enum(enum_value) => visitor.visit_borrowed_str(enum_value.name()),
            ConstValue::List(const_list) => {
                SeqDeserializer::new(const_list.into_iter().map(ConstValueDeserializer)).deserialize_any(visitor)
            }
            ConstValue::Object(const_object) => MapDeserializer::new(
                const_object
                    .into_iter()
                    .map(|field| (field.name(), ConstValueDeserializer(field.value()))),
            )
            .deserialize_any(visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if matches!(self.0, ConstValue::Null(_)) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ConstValueDeserializerError {
    #[error("{0}")]
    Message(String),
}

impl serde::de::Error for ConstValueDeserializerError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        ConstValueDeserializerError::Message(msg.to_string())
    }
}

impl<'de> IntoDeserializer<'de, ConstValueDeserializerError> for ConstValueDeserializer<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
