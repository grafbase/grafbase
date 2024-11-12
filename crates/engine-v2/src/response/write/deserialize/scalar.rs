use schema::ScalarType;
use serde::{de::DeserializeSeed, Deserialize};

use crate::response::ResponseValue;

pub(crate) struct ScalarTypeSeed(pub ScalarType);

impl<'de> DeserializeSeed<'de> for ScalarTypeSeed {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ScalarTypeSeed(ty) = self;

        match ty {
            ScalarType::String => String::deserialize(deserializer).map(Into::into),
            ScalarType::Float => f64::deserialize(deserializer).map(Into::into),
            ScalarType::Int => i32::deserialize(deserializer).map(Into::into),
            ScalarType::BigInt => i64::deserialize(deserializer).map(Into::into),
            ScalarType::JSON => Box::<serde_json::Value>::deserialize(deserializer).map(Into::into),
            ScalarType::Boolean => bool::deserialize(deserializer).map(Into::into),
        }
    }
}
