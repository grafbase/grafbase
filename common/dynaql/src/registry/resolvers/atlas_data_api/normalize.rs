use chrono::{DateTime, NaiveDate};
use serde_json::{json, Value};

use crate::{
    registry::{type_kinds::InputType, MetaInputValue, TypeReference},
    Context,
};

use super::JsonMap;

/// Given the input keys, converts them to the names on MongoDB.
///
/// Has any effect if a field in the data model is having a `@map` directive.
pub(super) fn keys(ctx: &Context<'_>, map: JsonMap, input_type: InputType<'_>) -> JsonMap {
    let mut result = JsonMap::new();

    for (key, value) in map {
        let meta_field = input_type.field(&key).unwrap();
        let key = meta_field.rename.clone().unwrap_or(key);
        let value = normalize(keys, ctx, value, &meta_field);

        result.insert(key, value);
    }

    result
}

/// Given the input values, converts them to the extended JSON format
/// on MongoDB.
pub(super) fn values(ctx: &Context<'_>, map: JsonMap, input_type: InputType<'_>) -> JsonMap {
    let mut result = JsonMap::new();

    for (key, value) in map {
        let meta_field = input_type.field(&key).unwrap();
        let value = normalize(values, ctx, value, &meta_field);
        let type_name = meta_field.ty.named_type();

        let value = match type_name.as_str() {
            "ID" => normalize_value(value, |value| json!({ "$oid": value })),
            "Date" => normalize_value(
                value,
                |value| json!({ "$date": { "$numberLong": date_to_timestamp(value) } }),
            ),
            "DateTime" => normalize_value(
                value,
                |value| json!({ "$date": { "$numberLong": datetime_to_timestamp(value) } }),
            ),
            "Timestamp" => normalize_value(
                value,
                |value| json!({ "$timestamp": { "t": datetime_to_timestamp(value), "i": 1 } }),
            ),
            "Decimal" => normalize_value(value, |value| json!({ "$numberDecimal": value })),
            "Bytes" => normalize_value(
                value,
                |value| json!({ "$binary": { "base64": value, "subType": "05" } }),
            ),
            "BigInt" => normalize_value(value, |value| json!({ "$numberLong": value.to_string() })),
            "MongoOrderByDirection" => match value.as_str() {
                Some("ASC") => Value::from(1),
                Some("DESC") => Value::from(-1),
                _ => value,
            },
            _ => value,
        };

        result.insert(key, value);
    }

    result
}

/// Given the input, converts the keys to the mapped variants in MongoDB,
/// and values to the extended JSON format.
pub(super) fn keys_and_values(
    ctx: &Context<'_>,
    map: JsonMap,
    input_type: InputType<'_>,
) -> JsonMap {
    let map = values(ctx, map, input_type);
    keys(ctx, map, input_type)
}

/// For filters and projection, fields in nested objects need to be flattened.
/// E.g. if we filter `{ address: { city: { eq: "Helsinki" }}}`, the filter we
/// send to MongoDB must be in the form `{ "address.city": { "$eq": "Helsinki" } }`.
pub(super) fn flatten_keys(input: JsonMap) -> JsonMap {
    fn recurse(input: JsonMap, output: &mut JsonMap, acc: Option<String>) {
        for (key, value) in input {
            match value {
                // Combine the nested keys into a single key, delimited with
                // a period.
                Value::Object(input) if !key.starts_with('$') => {
                    let acc = match acc {
                        None => Some(key),
                        Some(ref acc) => Some(format!("{acc}.{key}")),
                    };

                    recurse(input, output, acc);
                }
                Value::Object(input) if key == "$elemMatch" => {
                    let elem_match = flatten_keys(input);

                    match acc {
                        None => {
                            output.insert(key, elem_match.into());
                        }
                        Some(ref acc) => {
                            let mut inner = JsonMap::new();
                            inner.insert(key, elem_match.into());
                            output.insert(acc.to_string(), inner.into());
                        }
                    }
                }
                // a special case, if we use any filter functions in the nested
                // object, the function must be added as a separate object instead
                // of flattening.
                value if key.starts_with('$') => match acc {
                    None => {
                        output.insert(key, value);
                    }
                    Some(ref acc) => {
                        let mut inner = JsonMap::new();
                        inner.insert(key, value);
                        output.insert(acc.to_string(), inner.into());
                    }
                },
                value => {
                    let key = match acc {
                        None => key,
                        Some(ref acc) => format!("{acc}.{key}"),
                    };

                    output.insert(key, value);
                }
            }
        }
    }

    let mut result = JsonMap::new();
    recurse(input, &mut result, None);
    result
}

fn normalize<F>(normalize: F, ctx: &Context<'_>, value: Value, input_meta: &MetaInputValue) -> Value
where
    F: Fn(&Context<'_>, JsonMap, InputType<'_>) -> JsonMap,
{
    let nested_type = ctx
        .schema_env
        .registry
        .lookup(&input_meta.ty)
        .ok()
        .filter(InputType::is_input_object);

    match (value, nested_type) {
        (Value::Object(value), Some(nested_type)) => {
            let value = normalize(ctx, value, nested_type);
            Value::Object(value)
        }
        (Value::Array(values), Some(nested_type)) => {
            let values = values
                .into_iter()
                .map(|value| match value {
                    Value::Object(value) => {
                        let value = normalize(ctx, value, nested_type);
                        Value::Object(value)
                    }
                    value => value,
                })
                .collect();

            Value::Array(values)
        }
        (value, _) => value,
    }
}

fn normalize_value(input: Value, f: impl Fn(Value) -> Value) -> Value {
    match input {
        Value::Array(values) => {
            let mapped = values.into_iter().map(f).collect();
            Value::Array(mapped)
        }
        value => f(value),
    }
}

fn date_to_timestamp(input: Value) -> Value {
    input
        .as_str()
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .map(|date| {
            date.signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
                .num_milliseconds()
                .to_string()
        })
        .map(Value::String)
        .unwrap_or(input)
}

fn datetime_to_timestamp(input: Value) -> Value {
    input
        .as_str()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|date| Value::String(date.timestamp_millis().to_string()))
        .unwrap_or(input)
}
