use chrono::{DateTime, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::JsonMap;

/// A value representation in MongoDB. Extends over JSON with a few special types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MongoValue {
    ObjectId(String),
    Date(String),
    DateTime(String),
    Timestamp(u64),
    Decimal(String),
    Bytes(String),
    BigInt(i64),
    String(String),
    PosInt(u64),
    NegInt(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<MongoValue>),
    Object(JsonMap),
    Null,
}

impl MongoValue {
    pub fn is_id(&self) -> bool {
        matches!(self, MongoValue::ObjectId(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, MongoValue::Null)
    }

    pub fn from_json(type_name: &str, value: Value) -> Self {
        match (type_name, value) {
            ("ID", Value::String(value)) => Self::ObjectId(value),
            ("Date", Value::String(value)) => Self::Date(value),
            ("DateTime", Value::String(value)) => Self::DateTime(value),
            ("Timestamp", Value::Number(value)) => {
                Self::Timestamp(value.as_u64().expect("Timestamp not stored as u64."))
            }
            ("Decimal", Value::String(value)) => Self::Decimal(value),
            ("Bytes", Value::String(value)) => Self::Bytes(value),
            ("BigInt", Value::Number(value)) => Self::BigInt(value.as_i64().expect("BigInt not stored as i64.")),
            ("MongoOrderByDirection", Value::String(value)) => Self::NegInt(if value == "ASC" { 1 } else { -1 }),
            (_, Value::Null) => Self::Null,
            (_, Value::Bool(value)) => Self::Boolean(value),
            (_, Value::Number(value)) => value
                .as_u64()
                .map(Self::PosInt)
                .or_else(|| value.as_i64().map(Self::NegInt))
                .unwrap_or_else(|| value.as_f64().map(Self::Float).unwrap()),
            (_, Value::String(value)) => Self::String(value),
            (_, Value::Array(values)) => {
                let inner = values
                    .into_iter()
                    .map(|value| MongoValue::from_json(type_name, value))
                    .collect();

                Self::Array(inner)
            }
            (_, Value::Object(value)) => Self::Object(value),
        }
    }
}

impl From<MongoValue> for Value {
    fn from(value: MongoValue) -> Self {
        match value {
            MongoValue::ObjectId(value) => {
                json!({ "$oid": value })
            }
            MongoValue::Date(value) => {
                let date = NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                    .ok()
                    .map(|date| date.signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()))
                    .as_ref()
                    .map(Duration::num_milliseconds)
                    .map(|milliseconds| milliseconds.to_string())
                    .unwrap_or(value);

                json!({ "$date": { "$numberLong": date } })
            }
            MongoValue::DateTime(value) => {
                let date = DateTime::parse_from_rfc3339(&value)
                    .ok()
                    .as_ref()
                    .map(DateTime::timestamp_millis)
                    .map(|date| date.to_string())
                    .unwrap_or(value);

                json!({ "$date": { "$numberLong": date } })
            }
            MongoValue::Timestamp(value) => {
                json!({ "$timestamp": { "t": value, "i": 1 }})
            }
            MongoValue::Decimal(value) => {
                json!({ "$numberDecimal": value })
            }
            MongoValue::Bytes(value) => {
                json!({ "$binary": { "base64": value, "subType": "05" } })
            }
            MongoValue::BigInt(value) => {
                json!({ "$numberLong": value.to_string() })
            }
            MongoValue::String(value) => Value::String(value),
            MongoValue::PosInt(value) => Value::from(value),
            MongoValue::NegInt(value) => Value::from(value),
            MongoValue::Float(value) => Value::from(value),
            MongoValue::Boolean(value) => Value::Bool(value),
            MongoValue::Array(values) => {
                let inner = values.into_iter().map(Value::from).collect();
                Value::Array(inner)
            }
            MongoValue::Object(value) => Value::Object(value),
            MongoValue::Null => Value::Null,
        }
    }
}
