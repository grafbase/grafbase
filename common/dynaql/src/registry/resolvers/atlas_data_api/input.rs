pub(super) mod pagination;

use super::JsonMap;
use crate::{
    names::MONGODB_OUTPUT_FIELD_ID,
    registry::{resolvers::atlas_data_api::normalize, variables::VariableResolveDefinition},
    Context, ServerResult,
};
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::sync::OnceLock;

pub(super) fn by(ctx: &Context<'_>) -> ServerResult<JsonMap> {
    static BY_FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = BY_FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("by".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("by")?;
    let map = normalize::keys_and_values(ctx, map, input_type);

    Ok(map)
}

pub(super) fn filter(ctx: &Context<'_>) -> ServerResult<JsonMap> {
    static FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("filter".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("filter")?;
    let map = normalize::flatten_keys(normalize::keys_and_values(ctx, map, input_type));

    let map = match pagination::before(ctx)? {
        Some(before) => {
            let inner = map;

            let mut map = JsonMap::new();
            map.insert("$and".to_string(), json!([inner, before]));

            map
        }
        None => map,
    };

    let map = match pagination::after(ctx)? {
        Some(after) => {
            let inner = map;

            let mut map = JsonMap::new();
            map.insert("$and".to_string(), json!([inner, after]));

            map
        }
        None => map,
    };

    Ok(map)
}

pub(super) fn input(ctx: &Context<'_>) -> ServerResult<JsonMap> {
    static INPUT_FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = INPUT_FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("input".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;
    let input_type = ctx.find_argument_type("input")?;
    let map = normalize::keys_and_values(ctx, map, input_type);

    Ok(map)
}

pub(super) fn order_by(ctx: &Context<'_>) -> Option<Vec<JsonMap>> {
    static ORDER_BY: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = ORDER_BY.get_or_init(|| VariableResolveDefinition::InputTypeName("orderBy".to_string()));

    resolve_definition
        .resolve::<Vec<JsonMap>>(ctx, Option::<Value>::None)
        .ok()
}

pub(super) fn sort(ctx: &Context<'_>, definition: Option<&[JsonMap]>) -> ServerResult<Option<IndexMap<String, Value>>> {
    let last = last(ctx);

    match definition {
        Some(maps) if !maps.is_empty() => {
            let input_type = ctx.find_argument_type("orderBy")?;

            let mut order_by = IndexMap::new();

            for map in maps {
                let map = normalize::keys_and_values(ctx, map.clone(), input_type);
                let map = normalize::flatten_keys(map);

                order_by.extend(map);
            }

            if !order_by.contains_key(MONGODB_OUTPUT_FIELD_ID) {
                order_by.insert(MONGODB_OUTPUT_FIELD_ID.to_string(), Value::from(1));
            }

            if last.is_some() {
                order_by = order_by
                    .into_iter()
                    .map(|(key, value)| {
                        let value = match value {
                            Value::Number(number) if number.as_i64() == Some(-1) => Value::from(1),
                            Value::Number(number) if number.as_i64() == Some(1) => Value::from(-1),
                            value => value,
                        };

                        (key, value)
                    })
                    .collect();
            }

            Ok(Some(order_by))
        }
        _ => {
            if last.is_some() {
                let mut order_by = IndexMap::new();
                order_by.insert(MONGODB_OUTPUT_FIELD_ID.to_string(), Value::from(-1));

                Ok(Some(order_by))
            } else {
                Ok(None)
            }
        }
    }
}

pub(super) fn first(ctx: &Context<'_>) -> Option<usize> {
    static FIRST: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = FIRST.get_or_init(|| VariableResolveDefinition::InputTypeName("first".to_string()));

    match resolve_definition.resolve::<usize>(ctx, Option::<Value>::None) {
        Ok(value) => Some(value),
        _ => None,
    }
}

pub(super) fn last(ctx: &Context<'_>) -> Option<usize> {
    static LAST: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition = LAST.get_or_init(|| VariableResolveDefinition::InputTypeName("last".to_string()));

    match resolve_definition.resolve::<usize>(ctx, Option::<Value>::None) {
        Ok(value) => Some(value),
        _ => None,
    }
}
