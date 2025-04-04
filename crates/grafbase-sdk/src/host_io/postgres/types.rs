//! # Database Type conversions for Postgres
//!
//! This module provides traits and implementations for converting Rust types
//! to Postgres database values.
//!
//! ## Key Components
//!
//! - `DatabaseType` trait: Defines how types can be converted to database values
//! - `DatabaseValue`: Represents a value bound to a SQL query parameter
//! - Type implementations: Conversions for Rust primitives and common types
use std::time::SystemTime;

use chrono::Utc;
use uuid::Uuid;

use crate::wit;

/// A trait for types that can be converted to database values.
///
/// This trait allows Rust types to be converted into database values that can be
/// used in SQL queries. It provides methods to convert values with or without
/// explicit type hints.
pub trait DatabaseType {
    /// Converts this value into a database value.
    ///
    /// This method is used when binding parameters to SQL queries.
    /// It transforms a Rust value into the appropriate database representation.
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>);

    /// Returns the PostgreSQL data type for this value.
    ///
    /// This method provides type information that can be used
    /// for explicit type casting in SQL queries.
    fn type_hint() -> wit::PgType;

    /// Returns whether this value represents an array type.
    ///
    /// This method indicates if the type is an array of values
    /// rather than a single scalar value.
    fn is_array() -> bool;

    /// Converts this value into a bound database value with type information.
    ///
    /// This method creates a complete database value binding that includes both
    /// the actual value and its associated PostgreSQL type information,
    /// which can be used for parameter binding in SQL queries.
    fn into_bound_value(self) -> DatabaseValue
    where
        Self: Sized,
    {
        let (value, array_values) = self.into_value();

        let value = wit::PgBoundValue {
            value,
            type_: Self::type_hint(),
            is_array: Self::is_array(),
        };

        DatabaseValue { value, array_values }
    }
}

/// A struct representing a value that can be bound to a SQL query parameter.
///
/// This holds both the value itself and optional array-related value information
/// when the value represents an array of values.
pub struct DatabaseValue {
    pub(crate) value: wit::PgBoundValue,
    pub(crate) array_values: Option<wit::PgValueTree>,
}

/// A struct representing a 2D point with x and y coordinates.
///
/// This structure is used to represent geometric points in a two-dimensional space,
/// typically for use with PostgreSQL's geometric point type.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct Point {
    /// The x-coordinate of the point.
    pub x: f64,
    /// The y-coordinate of the point.
    pub y: f64,
}

/// A struct representing a time interval.
///
/// This structure corresponds to PostgreSQL's interval type, storing
/// a period of time decomposed into months, days, and microseconds.
/// It can be used for date/time arithmetic operations.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct Interval {
    /// The number of months in the interval.
    pub months: i32,
    /// The number of days in the interval.
    pub days: i32,
    /// The number of microseconds in the interval.
    pub microseconds: i64,
}

impl<T> DatabaseType for Option<T>
where
    T: DatabaseType,
{
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        match self {
            Some(value) => value.into_value(),
            None => (wit::PgValue::Null, None),
        }
    }

    fn type_hint() -> wit::PgType {
        T::type_hint()
    }

    fn is_array() -> bool {
        T::is_array()
    }
}

impl<T> DatabaseType for Vec<T>
where
    T: DatabaseType,
{
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        let mut values = Vec::with_capacity(self.len());
        let mut value_tree = wit::PgValueTree::with_capacity(self.len());

        for value in self {
            let index = value_tree.len();
            values.push(index as u64);

            // here we don't really care for nested arrays. they don't really work
            // in postgres, they are very weird. it's all in one dimension even though
            // it looks like you can nest them
            let (value, _) = value.into_value();
            value_tree.push(value);
        }

        (wit::PgValue::Array(values), Some(value_tree))
    }

    fn type_hint() -> wit::PgType {
        T::type_hint()
    }

    fn is_array() -> bool {
        true
    }
}

impl DatabaseType for String {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::String(self), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::String
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for i16 {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Int16(self), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Int16
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for i32 {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Int32(self), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Int32
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for i64 {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Int64(self), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Int64
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for f32 {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Float32(self), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Float32
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for f64 {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Float64(self), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Float64
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for bool {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Boolean(self), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Boolean
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for SystemTime {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        let ts = self.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros() as i64;

        (wit::PgValue::Timestamp(ts), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Int64
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for serde_json::Value {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        let json_str = serde_json::to_string(&self).unwrap();

        (wit::PgValue::Json(json_str), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::String
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for Uuid {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Uuid(self.to_string()), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::String
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for chrono::NaiveDate {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Date(self.to_string()), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::String
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for chrono::NaiveTime {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Time(self.to_string()), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::String
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for chrono::DateTime<Utc> {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Timestamp(self.timestamp_micros()), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Int64
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for rust_decimal::Decimal {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Numeric(self.to_string()), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::String
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for Point {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Point((self.x, self.y)), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Point
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for Interval {
    fn into_value(self) -> (wit::PgValue, Option<wit::PgValueTree>) {
        let value = wit::PgValue::Interval((self.months, self.days, self.microseconds));

        (value, None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Interval
    }

    fn is_array() -> bool {
        false
    }
}
