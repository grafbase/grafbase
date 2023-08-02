use super::JsonMap;
use crate::{
    registry::{resolvers::atlas_data_api::normalize, variables::VariableResolveDefinition},
    Context, ServerResult,
};
use grafbase_runtime::search::Cursor;
use serde_json::{json, Value};
use std::sync::OnceLock;

pub(super) fn by(ctx: &Context<'_>) -> ServerResult<Value> {
    static BY_FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = BY_FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("by".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("by")?;
    let map = normalize::keys_and_values(ctx, map, input_type);

    Ok(Value::Object(map))
}

pub(super) fn before(ctx: &Context<'_>) -> Option<JsonMap> {
    static FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("before".to_string()));

    let before = resolve_definition
        .resolve::<Cursor>(ctx, Option::<Value>::None)
        .ok()
        .and_then(|cursor| String::from_utf8(cursor.into_bytes()).ok());

    match before {
        Some(before) => {
            let mut map = JsonMap::new();

            map.insert(
                "_id".to_string(),
                json!({
                    "$lt": { "$oid": before }
                }),
            );

            Some(map)
        }
        _ => None,
    }
}

pub(super) fn after(ctx: &Context<'_>) -> Option<JsonMap> {
    static FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("after".to_string()));

    let after = resolve_definition
        .resolve::<Cursor>(ctx, Option::<Value>::None)
        .ok()
        .and_then(|cursor| String::from_utf8(cursor.into_bytes()).ok());

    match after {
        Some(after) => {
            let mut map = JsonMap::new();

            map.insert(
                "_id".to_string(),
                json!({
                    "$gt": { "$oid": after }
                }),
            );

            Some(map)
        }
        _ => None,
    }
}

pub(super) fn filter(ctx: &Context<'_>) -> ServerResult<Value> {
    static FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("filter".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("filter")?;
    let map = normalize::flatten_keys(normalize::keys_and_values(ctx, map, input_type));

    let map = match before(ctx) {
        Some(before) => {
            let inner = map;

            let mut map = JsonMap::new();
            map.insert("$and".to_string(), json!([inner, before]));

            map
        }
        None => map,
    };

    let map = match after(ctx) {
        Some(after) => {
            let inner = map;

            let mut map = JsonMap::new();
            map.insert("$and".to_string(), json!([inner, after]));

            map
        }
        None => map,
    };

    Ok(Value::Object(map))
}

pub(super) fn input(ctx: &Context<'_>) -> ServerResult<Value> {
    static INPUT_FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = INPUT_FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("input".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("input")?;
    let map = normalize::keys_and_values(ctx, map, input_type);

    Ok(Value::Object(map))
}

pub(super) fn order_by(ctx: &Context<'_>) -> ServerResult<Option<Value>> {
    static ORDER_BY: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = ORDER_BY.get_or_init(|| VariableResolveDefinition::InputTypeName("orderBy".to_string()));

    match resolve_definition.resolve::<JsonMap>(ctx, Option::<Value>::None) {
        Ok(map) if !map.is_empty() => {
            let input_type = ctx.find_argument_type("orderBy")?;
            let map = normalize::keys_and_values(ctx, map, input_type);
            let map = normalize::flatten_keys(map);

            Ok(Some(Value::Object(map)))
        }
        _ => Ok(None),
    }
}

pub(super) fn limit(ctx: &Context<'_>) -> Value {
    static ORDER_BY: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = ORDER_BY.get_or_init(|| VariableResolveDefinition::InputTypeName("first".to_string()));

    match resolve_definition.resolve::<Value>(ctx, Option::<Value>::None) {
        Ok(value) if !value.is_null() => value,
        _ => Value::from(100),
    }
}

pub(super) fn skip(ctx: &Context<'_>) -> Option<Value> {
    static ORDER_BY: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = ORDER_BY.get_or_init(|| VariableResolveDefinition::InputTypeName("skip".to_string()));

    match resolve_definition.resolve::<Value>(ctx, Option::<Value>::None) {
        Ok(value) if !value.is_null() => Some(value),
        _ => None,
    }
}
