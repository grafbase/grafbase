// use schema::DataType;
// use serde::{de::DeserializeSeed, Deserialize};
//
// use crate::response::ResponseValue;
//
// pub struct DeserializableDataType(DataType);
//
// impl<'de> DeserializeSeed<'de> for DeserializableDataType {
//     type Value = ResponseValue;
//
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         match self.0 {
//             DataType::String => String::deserialize(deserializer).map(ResponseValue::String),
//             DataType::Float => f64::deserialize(deserializer).map(ResponseValue::Float),
//             DataType::Int => i64::deserialize(deserializer).map(ResponseValue::Int),
//             DataType::Boolean => bool::deserialize(deserializer).map(ResponseValue::Boolean),
//         }
//     }
// }
