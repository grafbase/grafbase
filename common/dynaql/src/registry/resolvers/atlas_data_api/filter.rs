use chrono::{DateTime, NaiveDate};
use serde_json::{json, Value};

use super::JsonMap;
use crate::{
    registry::{type_kinds::InputType, variables::VariableResolveDefinition, TypeReference},
    Context, ServerResult,
};
use std::sync::OnceLock;

pub(super) fn by(ctx: &Context<'_>) -> ServerResult<Value> {
    static BY_FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition =
        BY_FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("by".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("by")?;

    Ok(Value::Object(normalize(ctx, map, input_type)))
}

pub(super) fn input(ctx: &Context<'_>) -> ServerResult<Value> {
    static INPUT_FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition =
        INPUT_FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("input".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("input")?;

    Ok(Value::Object(normalize(ctx, map, input_type)))
}

fn normalize(ctx: &Context<'_>, map: JsonMap, input_type: InputType<'_>) -> JsonMap {
    let mut result = JsonMap::new();

    for (key, value) in map {
        let meta_field = input_type.field(&key).unwrap();
        let key = meta_field.rename.clone().unwrap_or(key);

        let nested_type = ctx
            .schema_env
            .registry
            .lookup(&meta_field.ty)
            .ok()
            .filter(InputType::is_input_object);

        let value = match (value, nested_type) {
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
        };

        let value = match meta_field.ty.named_type().as_str() {
            "ID" => json!({ "$oid": value }),
            "Date" => {
                json!({ "$date": { "$numberLong": date_to_timestamp(value) } })
            }
            "DateTime" => {
                json!({ "$date": { "$numberLong": datetime_to_timestamp(value) } })
            }
            "Timestamp" => json!({ "$timestamp": { "t": value, "i": 1 }}),
            "Decimal" => json!({ "$numberDecimal": value }),
            "Bytes" => json!({ "$binary": { "base64": value, "subType": "05", }}),
            "BigInt" => json!({ "$numberLong": value.to_string() }),
            _ => value,
        };

        result.insert(key, value);
    }

    result
}

fn date_to_timestamp(input: Value) -> Value {
    match input {
        Value::String(ref value) => {
            let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap();

            let duration = date
                .signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
                .num_milliseconds();

            Value::String(duration.to_string())
        }
        value => value,
    }
}

fn datetime_to_timestamp(input: Value) -> Value {
    match input {
        Value::String(ref value) => {
            let date = DateTime::parse_from_rfc3339(value).unwrap();
            Value::String(date.timestamp_millis().to_string())
        }
        value => value,
    }
}
