use serde_json::{json, Value};

use super::JsonMap;
use crate::{
    registry::{variables::VariableResolveDefinition, MetaType},
    Context, ServerResult,
};
use std::sync::OnceLock;

pub(super) fn by(ctx: &Context<'_>) -> ServerResult<Value> {
    static BY_FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition =
        BY_FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("by".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("by")?;

    Ok(Value::Object(normalize(map, input_type)))
}

pub(super) fn input(ctx: &Context<'_>) -> ServerResult<Value> {
    static INPUT_FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition =
        INPUT_FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("input".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("input")?;

    Ok(Value::Object(normalize(map, input_type)))
}

fn normalize(map: JsonMap, input_type: &MetaType) -> JsonMap {
    let mut result = JsonMap::new();

    for (key, value) in map {
        let meta_field = input_type.get_input_field(&key).unwrap();
        let key = meta_field.rename.clone().unwrap_or(key);

        let value = match meta_field.ty.as_str() {
            "ID" => json!({ "$oid": value }),
            "Date" | "DateTime" => json!({ "$date": value }),
            "Timestamp" => json!({ "$timestamp": { "t": value, "i": 1 }}),
            "Decimal" => json!({ "$numberDecimal": value }),
            "Binary" => json!({ "$binary": { "base64": value, "subType": "05", }}),
            _ => value,
        };

        result.insert(key, value);
    }

    result
}
