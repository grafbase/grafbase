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

use crate::{
    SdkError,
    wit::{self, PgBoundValue},
};
pub use wit::PgType;

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
    fn into_value(self, base_idx: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>);

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
    fn into_bound_value(self, mut base_idx: u64) -> DatabaseValue
    where
        Self: Sized,
    {
        let (value, array_values) = self.into_value(&mut base_idx);

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
#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseValue {
    pub(crate) value: wit::PgBoundValue,
    pub(crate) array_values: Option<wit::PgValueTree>,
}

impl DatabaseValue {
    /// Creates an iterator over the values in this database value.
    ///
    /// If this is an array value, the iterator will yield each element in the array.
    /// If this is a scalar value, the iterator will yield only that value.
    ///
    /// # Returns
    ///
    /// A `DatabaseValueIterator` that iterates over the values.
    pub fn iter(self) -> DatabaseValueIterator {
        let r#type = self.value.type_;

        DatabaseValueIterator {
            value: Some(self.value),
            r#type,
            tree: self.array_values,
        }
    }

    /// Returns the value as a string slice if it is a string type, otherwise returns None.
    pub fn as_str(&self) -> Option<&str> {
        match self.value.value {
            wit::PgValue::String(ref s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns whether this database value is null.
    pub fn is_null(&self) -> bool {
        matches!(self.value.value, wit::PgValue::Null)
    }

    /// Converts the database value to a vector of individual database values if it's an array type.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<DatabaseValue>)` if this is an array type, containing each element as a separate `DatabaseValue`
    /// - `None` if this is not an array type
    pub fn to_list(&self) -> Option<Vec<DatabaseValue>> {
        match self.value.value {
            wit::PgValue::Array(ref array) => {
                let tree = self.array_values.as_ref()?;
                let mut result = Vec::with_capacity(array.len());

                for idx in array {
                    result.push(Self {
                        value: wit::PgBoundValue {
                            value: tree[(*idx) as usize].clone(),
                            type_: self.value.type_,
                            is_array: false,
                        },
                        array_values: None,
                    });
                }

                Some(result)
            }
            _ => None,
        }
    }

    /// Converts a JSON input value to a database value with a specified PostgreSQL type.
    ///
    /// This is meant for converting an input value to a database value matching the underlying
    /// Postgres type.
    ///
    /// # Parameters
    ///
    /// * `value` - The JSON input value to convert
    /// * `column_type` - The PostgreSQL type of the column this input value is either written or
    ///   filtered against.
    pub fn from_json_input(
        value: serde_json::Value,
        column_type: impl Into<wit::PgType>,
        column_is_array: bool,
    ) -> Result<Self, SdkError> {
        let column_type = column_type.into();

        let value = match value {
            serde_json::Value::Null => DatabaseValue {
                value: wit::PgBoundValue {
                    value: wit::PgValue::Null,
                    type_: column_type,
                    is_array: column_is_array,
                },
                array_values: None,
            },
            serde_json::Value::Bool(b) => b.into_bound_value(0),
            serde_json::Value::Number(number) => {
                if let Some(i) = number.as_i64() {
                    return Ok(i.into_bound_value(0));
                }

                if let Some(f) = number.as_f64() {
                    return Ok(f.into_bound_value(0));
                }

                return Err(SdkError::from(format!("Number out of range: {}", number)));
            }
            serde_json::Value::String(s) => s.into_bound_value(0),
            serde_json::Value::Array(values) => {
                let mut array_values = Vec::with_capacity(values.len());
                let mut indices = Vec::with_capacity(values.len());

                for (i, value) in values.into_iter().enumerate() {
                    indices.push(i as u64);

                    match value {
                        serde_json::Value::Null => array_values.push(wit::PgValue::Null),
                        serde_json::Value::Bool(b) => array_values.push(wit::PgValue::Boolean(b)),
                        serde_json::Value::Number(number) => {
                            if let Some(i) = number.as_i64() {
                                array_values.push(wit::PgValue::Int64(i));
                            } else if let Some(f) = number.as_f64() {
                                array_values.push(wit::PgValue::Float64(f));
                            } else {
                                return Err(SdkError::from(format!("Number out of range: {}", number)));
                            }
                        }
                        serde_json::Value::String(s) => array_values.push(wit::PgValue::String(s)),
                        serde_json::Value::Array(_) => return Err(SdkError::from("Nested arrays are not supported")),
                        json @ serde_json::Value::Object(_) => {
                            array_values.push(wit::PgValue::Json(serde_json::to_string(&json).unwrap()))
                        }
                    }
                }

                Self {
                    value: wit::PgBoundValue {
                        value: wit::PgValue::Array(indices),
                        type_: column_type,
                        is_array: true,
                    },
                    array_values: Some(array_values),
                }
            }
            json @ serde_json::Value::Object(_) => json.into_bound_value(0),
        };

        Ok(value)
    }
}

/// An iterator over the values in a `DatabaseValue`.
///
/// This iterator yields references to `PgValue` instances contained within
/// a `DatabaseValue`. For array values, it iterates through each element
/// in the array. For scalar values, it yields just that single value.
pub struct DatabaseValueIterator {
    value: Option<wit::PgBoundValue>,
    r#type: wit::PgType,
    tree: Option<wit::PgValueTree>,
}

impl Iterator for DatabaseValueIterator {
    type Item = DatabaseValue;

    fn next(&mut self) -> Option<Self::Item> {
        let value = match (self.value.take(), self.tree.as_mut()) {
            (_, Some(array)) => array.pop(),
            (Some(value), _) => Some(value.value),
            _ => None,
        };

        Some(DatabaseValue {
            value: PgBoundValue {
                value: value?,
                type_: self.r#type,
                is_array: false,
            },
            array_values: None,
        })
    }
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
    fn into_value(self, base_idx: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
        match self {
            Some(value) => value.into_value(base_idx),
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
    fn into_value(self, base_idx: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
        let mut values = Vec::with_capacity(self.len());
        let mut value_tree = wit::PgValueTree::with_capacity(self.len());

        for value in self {
            let index = value_tree.len();
            values.push(index as u64 + *base_idx);

            *base_idx += 1;

            // here we don't really care for nested arrays. they don't really work
            // in postgres, they are very weird. it's all in one dimension even though
            // it looks like you can nest them
            let (value, _) = value.into_value(base_idx);
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Int32(self), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Int32
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for u32 {
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Int64(self as i64), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Int64
    }

    fn is_array() -> bool {
        false
    }
}

impl DatabaseType for i64 {
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
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

impl DatabaseType for Vec<u8> {
    fn into_value(self, _: &mut u64) -> (wit::PgValue, Option<wit::PgValueTree>) {
        (wit::PgValue::Bytes(self), None)
    }

    fn type_hint() -> wit::PgType {
        wit::PgType::Bytes
    }

    fn is_array() -> bool {
        false
    }
}
