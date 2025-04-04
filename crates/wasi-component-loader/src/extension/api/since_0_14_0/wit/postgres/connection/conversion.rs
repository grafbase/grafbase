use sqlx::{
    Postgres,
    postgres::{
        PgArguments,
        types::{PgInterval, PgPoint},
    },
    query::Query,
};

use crate::extension::api::since_0_14_0::wit::postgres::{PgType, PgValue, PgValueTree};

pub fn bind_value<'a>(
    query: Query<'a, Postgres, PgArguments>,
    value: PgValue,
    r#type: PgType,
    tree: &PgValueTree,
    is_array: bool,
) -> Query<'a, Postgres, PgArguments> {
    match value {
        PgValue::Null => match r#type {
            PgType::Boolean if is_array => query.bind(Option::<Vec<bool>>::None),
            PgType::Int16 if is_array => query.bind(Option::<Vec<i16>>::None),
            PgType::Int32 if is_array => query.bind(Option::<Vec<i32>>::None),
            PgType::Int64 if is_array => query.bind(Option::<Vec<i64>>::None),
            PgType::Float32 if is_array => query.bind(Option::<Vec<f32>>::None),
            PgType::Float64 if is_array => query.bind(Option::<Vec<f64>>::None),
            PgType::String if is_array => query.bind(Option::<Vec<&str>>::None),
            PgType::Bytes if is_array => query.bind(Option::<Vec<Vec<u8>>>::None),
            PgType::Point if is_array => query.bind(Option::<Vec<PgPoint>>::None),
            PgType::Interval if is_array => query.bind(Option::<Vec<PgInterval>>::None),
            PgType::Boolean => query.bind(Option::<bool>::None),
            PgType::Int16 => query.bind(Option::<i16>::None),
            PgType::Int32 => query.bind(Option::<i32>::None),
            PgType::Int64 => query.bind(Option::<i64>::None),
            PgType::Float32 => query.bind(Option::<f32>::None),
            PgType::Float64 => query.bind(Option::<f64>::None),
            PgType::String => query.bind(Option::<&str>::None),
            PgType::Bytes => query.bind(Option::<Vec<u8>>::None),
            PgType::Point => query.bind(Option::<PgPoint>::None),
            PgType::Interval => query.bind(Option::<PgInterval>::None),
        },
        PgValue::Boolean(value) => query.bind(value),
        PgValue::Int16(value) => query.bind(value),
        PgValue::Int32(value) => query.bind(value),
        PgValue::Int64(value) => query.bind(value),
        PgValue::Float32(value) => query.bind(value),
        PgValue::Float64(value) => query.bind(value),
        PgValue::String(value) => query.bind(value),
        PgValue::Bytes(value) => query.bind(value),
        PgValue::Uuid(value) => query.bind(value),
        PgValue::Json(value) => query.bind(value),
        PgValue::Timestamp(value) => query.bind(value),
        PgValue::Date(value) => query.bind(value),
        PgValue::Time(value) => query.bind(value),
        PgValue::DateTime(value) => query.bind(value),
        PgValue::Array(indices) => {
            // Handle different array types based on the PgType
            match r#type {
                PgType::Boolean => {
                    let values: Vec<bool> = indices
                        .iter()
                        .filter_map(|&index| match &tree[index as usize] {
                            PgValue::Boolean(val) => Some(*val),
                            _ => None,
                        })
                        .collect();

                    query.bind(values)
                }
                PgType::Int16 => {
                    let values: Vec<i16> = indices
                        .iter()
                        .filter_map(|&index| match &tree[index as usize] {
                            PgValue::Int16(val) => Some(*val),
                            _ => None,
                        })
                        .collect();

                    query.bind(values)
                }
                PgType::Int32 => {
                    let values: Vec<i32> = indices
                        .iter()
                        .filter_map(|&index| match &tree[index as usize] {
                            PgValue::Int32(val) => Some(*val),
                            _ => None,
                        })
                        .collect();

                    query.bind(values)
                }
                PgType::Int64 => {
                    let values: Vec<i64> = indices
                        .iter()
                        .filter_map(|&index| match &tree[index as usize] {
                            PgValue::Int64(val) => Some(*val),
                            _ => None,
                        })
                        .collect();

                    query.bind(values)
                }
                PgType::Float32 => {
                    let values: Vec<f32> = indices
                        .iter()
                        .filter_map(|&index| match &tree[index as usize] {
                            PgValue::Float32(val) => Some(*val),
                            _ => None,
                        })
                        .collect();

                    query.bind(values)
                }
                PgType::Float64 => {
                    let values: Vec<f64> = indices
                        .iter()
                        .filter_map(|&index| match &tree[index as usize] {
                            PgValue::Float64(val) => Some(*val),
                            _ => None,
                        })
                        .collect();

                    query.bind(values)
                }
                PgType::String => {
                    let values: Vec<String> = indices
                        .iter()
                        .filter_map(|&index| match &tree[index as usize] {
                            PgValue::String(val) => Some(val.to_string()),
                            _ => None,
                        })
                        .collect();

                    query.bind(values)
                }
                PgType::Bytes => {
                    let values: Vec<Vec<u8>> = indices
                        .iter()
                        .filter_map(|&index| match &tree[index as usize] {
                            PgValue::Bytes(val) => Some(val.clone()),
                            _ => None,
                        })
                        .collect();

                    query.bind(values)
                }
                // Handle other types or fall back to a default implementation
                _ => {
                    // For types we don't explicitly handle, convert to strings
                    let values: Vec<String> = indices
                        .iter()
                        .map(|&index| format!("{:?}", &tree[index as usize]))
                        .collect();

                    query.bind(values)
                }
            }
        }
        PgValue::Numeric(value) => query.bind(value),
        PgValue::Point(value) => query.bind(PgPoint { x: value.0, y: value.1 }),
        PgValue::Interval(value) => query.bind(PgInterval {
            months: value.0,
            days: value.1,
            microseconds: value.2,
        }),
        PgValue::Inet(value) => query.bind(value),
        PgValue::Macaddr(value) => query.bind(value),
        PgValue::Bit(value) => query.bind(value),
        PgValue::Money(value) => query.bind(value),
        PgValue::Xml(value) => query.bind(value),
    }
}
