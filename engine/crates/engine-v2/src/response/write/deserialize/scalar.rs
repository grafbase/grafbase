use serde::{de::DeserializeSeed, Deserialize, Deserializer};

use crate::response::ResponseValue;

pub struct IntSeed;

impl<'de> DeserializeSeed<'de> for IntSeed {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        i32::deserialize(deserializer).map(|value| ResponseValue::Int { value, nullable: false })
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct BigIntSeed;

impl<'de> DeserializeSeed<'de> for BigIntSeed {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        i64::deserialize(deserializer).map(|value| ResponseValue::BigInt { value, nullable: false })
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct FloatSeed;

impl<'de> DeserializeSeed<'de> for FloatSeed {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        f64::deserialize(deserializer).map(|value| ResponseValue::Float { value, nullable: false })
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct JSONSeed;

impl<'de> DeserializeSeed<'de> for JSONSeed {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde_json::Value::deserialize(deserializer).map(|value| ResponseValue::Json {
            value: Box::new(value),
            nullable: false,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct StringSeed;

impl<'de> DeserializeSeed<'de> for StringSeed {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| ResponseValue::String {
            value: value.into_boxed_str(),
            nullable: false,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct BooleanSeed;

impl<'de> DeserializeSeed<'de> for BooleanSeed {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        bool::deserialize(deserializer).map(|value| ResponseValue::Boolean { value, nullable: false })
    }
}
