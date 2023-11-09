pub(super) mod pagination;

use indexmap::IndexMap;
use serde_json::{json, Value};

use super::{
    consts::{CURRENT_DATE, CURRENT_DATETIME, CURRENT_TIMESTAMP, OP_AND, OP_UNSET},
    JsonMap,
};
use crate::{
    names::MONGODB_OUTPUT_FIELD_ID,
    registry::resolvers::atlas_data_api::{
        consts::{TYPE, TYPE_DATE, TYPE_TIMESTAMP},
        normalize,
    },
    ContextField, ServerResult,
};

pub(super) fn by(ctx: &ContextField<'_>) -> ServerResult<JsonMap> {
    let map = ctx.input_by_name("by")?;
    let input_type = ctx.find_argument_type("by")?;
    let map = normalize::keys_and_values(ctx, map, input_type);

    Ok(map)
}

pub(super) fn filter(ctx: &ContextField<'_>) -> ServerResult<JsonMap> {
    let map = ctx.input_by_name("filter")?;
    let input_type = ctx.find_argument_type("filter")?;
    let map = normalize::flatten_keys(normalize::keys_and_values(ctx, map, input_type));

    let map = match pagination::before(ctx)? {
        Some(before) => {
            let inner = map;

            let mut map = JsonMap::new();
            map.insert(OP_AND.to_string(), json!([inner, before]));

            map
        }
        None => map,
    };

    let map = match pagination::after(ctx)? {
        Some(after) => {
            let inner = map;

            let mut map = JsonMap::new();
            map.insert(OP_AND.to_string(), json!([inner, after]));

            map
        }
        None => map,
    };

    Ok(map)
}

pub(super) fn input(ctx: &ContextField<'_>) -> ServerResult<JsonMap> {
    let map = ctx.input_by_name("input")?;
    let input_type = ctx.find_argument_type("input")?;
    let map = normalize::keys_and_values(ctx, map, input_type);

    Ok(map)
}

pub(super) fn input_many(ctx: &ContextField<'_>) -> ServerResult<Vec<JsonMap>> {
    let maps: Vec<JsonMap> = ctx.input_by_name("input")?;
    let input_type = ctx.find_argument_type("input")?;

    let result = maps
        .into_iter()
        .map(|map| normalize::keys_and_values(ctx, map, input_type))
        .collect();

    Ok(result)
}

pub(super) fn order_by(ctx: &ContextField<'_>) -> Option<Vec<JsonMap>> {
    ctx.input_by_name("orderBy").ok()
}

pub(super) fn sort(
    ctx: &ContextField<'_>,
    definition: Option<&[JsonMap]>,
) -> ServerResult<Option<IndexMap<String, Value>>> {
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

pub(super) fn first(ctx: &ContextField<'_>) -> Option<usize> {
    ctx.input_by_name("first").ok()
}

pub(super) fn last(ctx: &ContextField<'_>) -> Option<usize> {
    ctx.input_by_name("last").ok()
}

pub(super) fn update(ctx: &ContextField<'_>) -> ServerResult<JsonMap> {
    let input: JsonMap = ctx.input_by_name("input")?;
    let input_type = ctx.find_argument_type("input")?;
    let input = normalize::keys_and_values(ctx, input, input_type);
    let input = normalize::flatten_keys(input);

    let mut update = JsonMap::new();

    for (field, statement) in input {
        let object = match statement {
            Value::Object(object) => object,
            _ => continue,
        };

        for (query_name, query) in object {
            let is_date_time_query = query_name.starts_with("$current");

            if is_date_time_query && !query.as_bool().unwrap_or_default() {
                continue;
            }

            if query_name == OP_UNSET && !query.as_bool().unwrap_or_default() {
                continue;
            }

            let (query_name, query) = match query_name.as_str() {
                CURRENT_TIMESTAMP => (String::from(CURRENT_DATE), json!({ TYPE: TYPE_TIMESTAMP })),
                CURRENT_DATE | CURRENT_DATETIME => (String::from(CURRENT_DATE), json!({ TYPE: TYPE_DATE })),
                _ => (query_name, query),
            };

            let entry = update.entry(query_name).or_insert(Value::Object(JsonMap::new()));

            let object = match entry.as_object_mut() {
                Some(object) => object,
                _ => continue,
            };

            object.insert(field.clone(), query);
        }
    }

    Ok(update)
}
