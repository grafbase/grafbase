use std::net::IpAddr;

use chrono::{serde::ts_milliseconds, DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::serde_utils::naive_date;

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

impl PartialOrd for ScalarValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (ScalarValue::URL(a), ScalarValue::URL(b))
            | (ScalarValue::Email(a), ScalarValue::Email(b))
            | (ScalarValue::PhoneNumber(a), ScalarValue::PhoneNumber(b))
            | (ScalarValue::String(a), ScalarValue::String(b)) => a.partial_cmp(b),
            (ScalarValue::Int(a), ScalarValue::Int(b)) => a.partial_cmp(b),
            (ScalarValue::Float(a), ScalarValue::Float(b)) => a.partial_cmp(b),
            (ScalarValue::Timestamp(a), ScalarValue::Timestamp(b))
            | (ScalarValue::DateTime(a), ScalarValue::DateTime(b)) => a.partial_cmp(b),
            (ScalarValue::Date(a), ScalarValue::Date(b)) => a.partial_cmp(b),
            (ScalarValue::IPAddress(a), ScalarValue::IPAddress(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl std::fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ScalarValue::*;
        match self {
            URL(value) | Email(value) | PhoneNumber(value) | String(value) => value.fmt(f),
            Int(value) => value.fmt(f),
            Float(value) => value.fmt(f),
            Timestamp(value) | DateTime(value) => value.fmt(f),
            Date(value) => value.fmt(f),
            Boolean(value) => value.fmt(f),
            IPAddress(value) => value.fmt(f),
        }
    }
}

impl From<&str> for ScalarValue {
    fn from(value: &str) -> Self {
        ScalarValue::String(value.to_string())
    }
}

impl From<f64> for ScalarValue {
    fn from(value: f64) -> Self {
        ScalarValue::Float(value)
    }
}

impl From<i64> for ScalarValue {
    fn from(value: i64) -> Self {
        ScalarValue::Int(value)
    }
}
impl From<bool> for ScalarValue {
    fn from(value: bool) -> Self {
        ScalarValue::Boolean(value)
    }
}
impl From<IpAddr> for ScalarValue {
    fn from(value: IpAddr) -> Self {
        ScalarValue::IPAddress(value)
    }
}
impl From<NaiveDate> for ScalarValue {
    fn from(value: NaiveDate) -> Self {
        ScalarValue::Date(value)
    }
}
impl From<DateTime<Utc>> for ScalarValue {
    fn from(value: DateTime<Utc>) -> Self {
        ScalarValue::DateTime(value)
    }
}
