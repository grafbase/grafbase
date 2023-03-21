use std::collections::HashMap;
use std::net::IpAddr;

use chrono::{serde::ts_milliseconds, NaiveDate};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

mod naive_date {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.format(FORMAT).to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScalarValue {
    URL(String),
    Email(String),
    PhoneNumber(String),
    String(String),
    Int(i64),
    Float(f64),
    Timestamp(#[serde(with = "ts_milliseconds")] DateTime<Utc>),
    Date(#[serde(with = "naive_date")] NaiveDate),
    DateTime(#[serde(with = "ts_milliseconds")] DateTime<Utc>),
    Boolean(bool),
    IPAddress(IpAddr),
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub indices: HashMap<String, IndexConfig>,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct IndexConfig {
    pub schema: Schema,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub fields: HashMap<String, FieldEntry>,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct FieldEntry {
    pub ty: FieldType,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum FieldType {
    URL(FieldOptions),
    Email(FieldOptions),
    PhoneNumber(FieldOptions),
    String(FieldOptions),
    Date(FieldOptions),
    DateTime(FieldOptions),
    Timestamp(FieldOptions),
    Int(FieldOptions),
    Float(FieldOptions),
    Boolean(FieldOptions),
    IPAddress(FieldOptions),
}

#[derive(Clone, Eq, PartialEq, Default, Hash, Debug, Serialize, Deserialize)]
pub struct FieldOptions {
    pub nullable: bool,
}

impl FieldType {
    pub fn is_nullable(&self) -> bool {
        match self {
            FieldType::URL(opts)
            | FieldType::Email(opts)
            | FieldType::PhoneNumber(opts)
            | FieldType::String(opts)
            | FieldType::Date(opts)
            | FieldType::DateTime(opts)
            | FieldType::Timestamp(opts)
            | FieldType::Int(opts)
            | FieldType::Float(opts)
            | FieldType::Boolean(opts)
            | FieldType::IPAddress(opts) => opts.nullable,
        }
    }
}
