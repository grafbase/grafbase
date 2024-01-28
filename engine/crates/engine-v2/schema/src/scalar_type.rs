use std::str::FromStr;

use serde::Deserialize;

/// Defines how a scalar should be represented and validated by the engine. They're almost the same
/// as scalars, but scalars like ID which have no own data format are just mapped to String.
/// https://the-guild.dev/graphql/scalars/docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScalarType {
    String,
    Float,
    Int,
    BigInt,
    JSON,
    Boolean,
}

pub enum ScalarValue {
    String(Box<str>),
    Float(f64),
    Int(i32),
    BigInt(i64),
    Json(Box<serde_json::Value>),
    Boolean(bool),
}

impl ScalarType {
    pub fn from_scalar_name(name: &str) -> ScalarType {
        ScalarType::from_str(name).ok().unwrap_or(match name {
            "ID" => ScalarType::String,
            _ => ScalarType::JSON,
        })
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for ScalarType {
    type Value = ScalarValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self {
            ScalarType::String => String::deserialize(deserializer).map(|s| ScalarValue::String(s.into_boxed_str())),
            ScalarType::Float => f64::deserialize(deserializer).map(ScalarValue::Float),
            ScalarType::Int => i32::deserialize(deserializer).map(ScalarValue::Int),
            ScalarType::BigInt => i64::deserialize(deserializer).map(ScalarValue::BigInt),
            ScalarType::JSON => {
                serde_json::Value::deserialize(deserializer).map(|json| ScalarValue::Json(Box::new(json)))
            }
            ScalarType::Boolean => bool::deserialize(deserializer).map(ScalarValue::Boolean),
        }
    }
}
