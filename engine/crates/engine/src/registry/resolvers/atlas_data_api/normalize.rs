use serde_json::Value;

use super::{value::MongoValue, JsonMap};
use crate::{
    registry::{
        resolvers::atlas_data_api::consts::OP_ELEM_MATCH, type_kinds::InputType, MetaInputValue, TypeReference,
    },
    ContextField,
};

/// Given the input keys, converts them to the names on MongoDB.
///
/// Has any effect if a field in the data model is having a `@map` directive.
pub(super) fn keys(ctx: &ContextField<'_>, map: JsonMap, input_type: InputType<'_>) -> JsonMap {
    let mut result = JsonMap::new();

    for (key, value) in map {
        let Some(meta_field) = input_type.field(&key) else {
            continue;
        };
        let key = meta_field.rename.clone().unwrap_or(key);
        let value = normalize(keys, ctx, value, meta_field);

        result.insert(key, value);
    }

    result
}

/// Given the input values, converts them to the extended JSON format
/// on MongoDB.
pub(super) fn values(ctx: &ContextField<'_>, map: JsonMap, input_type: InputType<'_>) -> JsonMap {
    let mut result = JsonMap::new();

    for (key, value) in map {
        let meta_field = input_type.field(&key).unwrap();
        let value = normalize(values, ctx, value, meta_field);
        let type_name = meta_field.ty.named_type();
        let value = MongoValue::from_json(type_name.as_str(), value).into();

        result.insert(key, value);
    }

    result
}

/// Given the input, converts the keys to the mapped variants in MongoDB,
/// and values to the extended JSON format.
pub(super) fn keys_and_values(ctx: &ContextField<'_>, map: JsonMap, input_type: InputType<'_>) -> JsonMap {
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
                Value::Object(input) if key == OP_ELEM_MATCH => {
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
                    Some(ref acc) => match output.get_mut(acc).and_then(serde_json::Value::as_object_mut) {
                        Some(object) => {
                            object.insert(key, value);
                        }
                        None => {
                            let mut inner = JsonMap::new();
                            inner.insert(key, value);
                            output.insert(acc.to_string(), inner.into());
                        }
                    },
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

fn normalize<F>(normalize: F, ctx: &ContextField<'_>, value: Value, input_meta: &MetaInputValue) -> Value
where
    F: Fn(&ContextField<'_>, JsonMap, InputType<'_>) -> JsonMap,
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
