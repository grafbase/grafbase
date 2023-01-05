use dynaql_value::{ConstValue, Name};
use indexmap::IndexMap;

use crate::registry::scalars::{DynamicScalar, PossibleScalar};
use crate::registry::{MetaInputValue, MetaType, MetaTypeName, Registry};

use crate::{Context, Error, ServerResult};

pub fn resolve_input(
    ctx_field: &Context<'_>,
    meta_input_value: &MetaInputValue,
    value: ConstValue,
) -> ServerResult<ConstValue> {
    // We do keep serde_json::Value::Null here contrary to resolver_input_inner
    // as it allows casting to either T or Option<T> later.
    resolve_input_inner(
        &ctx_field.schema_env.registry,
        &mut Vec::new(),
        &meta_input_value.ty,
        value,
        meta_input_value.default_value.as_ref(),
    )
    .map_err(|err| err.into_server_error(ctx_field.item.pos))
}

fn resolve_input_inner(
    registry: &Registry,
    path: &mut Vec<String>,
    ty: &str,
    value: ConstValue,
    default_value: Option<&ConstValue>,
) -> Result<ConstValue, Error> {
    if value != ConstValue::Null {
        match MetaTypeName::create(&ty) {
            MetaTypeName::List(type_name) => {
                if let ConstValue::List(list) = value {
                    let mut arr = Vec::new();
                    for (idx, element) in list.into_iter().enumerate() {
                        path.push(idx.to_string());
                        arr.push(resolve_input_inner(
                            registry, path, &type_name, element, None,
                        )?);
                        path.pop();
                    }
                    Ok(ConstValue::List(arr))
                } else {
                    Err(input_error("Expected a List", path))
                }
            }
            MetaTypeName::NonNull(type_name) => {
                resolve_input_inner(registry, path, &type_name, value, None)
            }
            MetaTypeName::Named(type_name) => {
                match registry
                    .types
                    .get(type_name)
                    .expect("Registry has already been validated")
                {
                    MetaType::InputObject {
                        input_fields,
                        oneof,
                        ..
                    } => {
                        if let ConstValue::Object(mut fields) = value {
                            let mut map = IndexMap::with_capacity(input_fields.len());
                            for (name, meta_input_value) in input_fields {
                                path.push(name.clone());
                                let field_value = resolve_input_inner(
                                    registry,
                                    path,
                                    &meta_input_value.ty,
                                    fields.remove(&Name::new(name)).unwrap_or(ConstValue::Null),
                                    meta_input_value.default_value.as_ref(),
                                )?;
                                path.pop();
                                // Not adding NULLs for now makes it easier to work with later.
                                // TODO: Keep NULL, they might be relevant in the future. Currently
                                // it's just not ideal with how we manipulate @oneof inputs
                                if field_value != ConstValue::Null {
                                    map.insert(Name::new(name), field_value);
                                }
                            }
                            if *oneof && map.len() != 1 {
                                Err(input_error(
                                    &format!(
                                        "Expected exactly one fields (@oneof), got {}",
                                        map.len()
                                    ),
                                    path,
                                ))
                            } else {
                                Ok(ConstValue::Object(map))
                            }
                        } else {
                            Err(input_error("Expected an Object", path))
                        }
                    }
                    MetaType::Enum { .. } => Ok(value),
                    // TODO: this conversion ConstValue -> serde_json -> ConstValue is sad...
                    // we need an intermediate representation between the database & dynaql
                    MetaType::Scalar { .. } => Ok(ConstValue::from_json(
                        PossibleScalar::parse(type_name, value)
                            .map_err(|err| Error::new(err.message()))?,
                    )?),
                    _ => Err(input_error(
                        &format!("Internal Error: Unsupported input type {type_name}"),
                        path,
                    )),
                }
            }
        }
    } else {
        match default_value {
            Some(v) => Ok(v.clone()),
            None => match MetaTypeName::create(&ty) {
                MetaTypeName::NonNull(_) => Err(input_error("Unexpected null value", path)),
                _ => Ok(ConstValue::Null),
            },
        }
    }
}

fn input_error(expected: &str, path: &[String]) -> Error {
    Error::new(format!("{expected} for {}", path.join(".")))
}
