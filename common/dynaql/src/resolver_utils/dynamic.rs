use std::borrow::Cow;

use dynaql_value::{ConstValue, Name};
use graph_entities::{ResponseNodeId, ResponsePrimitive};

use crate::dynamic::{DynamicFieldContext, DynamicFieldKind};
use crate::graph::selection_set_into_node;
use crate::model::{__Schema, __Type};
use crate::registry::resolvers::{ResolvedContainer, ResolvedValue};
use crate::registry::{MetaInputValue, MetaType, MetaTypeName, Registry};
use crate::resolver_utils::{resolve_container, resolve_list};
use crate::{Context, Error, OutputType, ServerError, ServerResult};

pub async fn resolve_introspection_field(
    ctx: &Context<'_>,
    root_type: &MetaType,
) -> ServerResult<Option<ResponseNodeId>> {
    if !ctx.schema_env.registry.disable_introspection && !ctx.query_env.disable_introspection {
        if ctx.item.node.name.node == "__schema" {
            let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
            let visible_types = ctx.schema_env.registry.find_visible_types(&ctx);
            let a = selection_set_into_node(
                OutputType::resolve(
                    &__Schema::new(&ctx.schema_env.registry, &visible_types),
                    &ctx_obj,
                    ctx.item,
                )
                .await?,
                &ctx_obj.to_dynamic(root_type),
            )
            .await;

            return Ok(Some(a));
        } else if ctx.item.node.name.node == "__type" {
            let (_, type_name) = ctx.param_value::<String>("name", None)?;
            let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
            let visible_types = ctx.schema_env.registry.find_visible_types(&ctx);
            let a = selection_set_into_node(
                OutputType::resolve(
                    &ctx.schema_env
                        .registry
                        .types
                        .get(&type_name)
                        .filter(|_| visible_types.contains(type_name.as_str()))
                        .map(|ty| __Type::new_simple(&ctx.schema_env.registry, &visible_types, ty)),
                    &ctx_obj,
                    ctx.item,
                )
                .await?,
                &ctx_obj.to_dynamic(root_type),
            )
            .await;
            return Ok(Some(a));
        }
    }
    Err(ServerError::new(
        format!("Unknown field '{}'", ctx.item.node.name.node),
        Some(ctx.item.pos),
    ))
}

pub async fn resolve_field<'ctx>(
    ctx_field: &DynamicFieldContext<'ctx>,
    maybe_parent_resolved_container: Option<&ResolvedValue<'ctx>>,
) -> ServerResult<Option<ResponseNodeId>> {
    let meta_field_resolver_result =
        run_meta_field_resolver(ctx_field, maybe_parent_resolved_container).await;
    let result = match ctx_field.kind() {
        DynamicFieldKind::PRIMITIVE => {
            resolve_primitive(ctx_field, meta_field_resolver_result).await
        }
        DynamicFieldKind::OBJECT => resolve_object(ctx_field, meta_field_resolver_result).await,
        DynamicFieldKind::ARRAY => resolve_array(ctx_field, meta_field_resolver_result).await,
    };

    Ok(Some(result.map_err(|err| ctx_field.set_error_path(err))?))
}

async fn resolve_primitive<'ctx>(
    ctx_field: &DynamicFieldContext<'ctx>,
    meta_field_resolver_result: ServerResult<ResolvedValue<'ctx>>,
) -> ServerResult<ResponseNodeId> {
    let result = match meta_field_resolver_result {
        Ok(result) => {
            if ctx_field.meta.is_required() && result.value.is_null() {
                #[cfg(feature = "tracing_worker")]
                logworker::warn!(
                        ctx_field.data_unchecked::<dynamodb::DynamoDBContext>().trace_id,
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "message": "Something went wrong here",
                            "expected": serde_json::Value::String(ctx_field.meta.ty.clone()),
                            "path": serde_json::Value::String(ctx_field.path_node.map(|p| p.to_string()).unwrap_or_default()),
                        }))
                        .unwrap(),
                    );
                Err(ServerError::new(
                    format!(
                        "An error happened while fetching {:?}",
                        ctx_field.item.node.name
                    ),
                    Some(ctx_field.item.pos),
                ))
            } else {
                Ok(result.value.into_owned())
            }
        }
        Err(err) => {
            if ctx_field.meta.is_required() {
                Err(err)
            } else {
                // FIXME: Inconsistent with array & object error management, do we want to keep
                // it?
                ctx_field.add_error(err);
                Ok(serde_json::Value::Null)
            }
        }
    }?;

    let result = ConstValue::from_json(result)
        .map_err(|err| ServerError::new(err.to_string(), Some(ctx_field.item.pos)))?;

    Ok(ctx_field
        .response_graph
        .write()
        .await
        .new_node_unchecked(ResponsePrimitive::new(result).into()))
}

async fn resolve_object<'ctx>(
    ctx_field: &DynamicFieldContext<'ctx>,
    meta_field_resolver_result: ServerResult<ResolvedValue<'ctx>>,
) -> ServerResult<ResponseNodeId> {
    let resolved_value = meta_field_resolver_result?;
    if resolved_value.value.is_null() {
        if ctx_field.meta.is_required() {
            return Err(ServerError::new(
                format!(
                    "An error occurred while fetching `{}`, a non-nullable value was expected but no value was found.",
                    ctx_field.item.node.name.node
                ),
                Some(ctx_field.item.pos),
            ));
        } else {
            return Ok(ctx_field
                .response_graph
                .write()
                .await
                .new_node_unchecked(ResponsePrimitive::new(ConstValue::Null).into()));
        }
    }
    let resolved_container = ResolvedContainer::new(ctx_field.base_type, resolved_value);
    // Allows nested fields to use the resolved_value
    match resolve_container(&ctx_field.get_selection(), Some(resolved_container)).await {
        result @ Ok(_) => result,
        Err(err) => {
            if ctx_field.meta.is_required() {
                Err(err)
            } else {
                ctx_field.add_error(err);
                Ok(ctx_field
                    .response_graph
                    .write()
                    .await
                    .new_node_unchecked(ResponsePrimitive::new(ConstValue::Null).into()))
            }
        }
    }
}

async fn resolve_array<'ctx>(
    ctx_field: &DynamicFieldContext<'ctx>,
    meta_field_resolver_result: ServerResult<ResolvedValue<'ctx>>,
) -> ServerResult<ResponseNodeId> {
    let resolved_list = meta_field_resolver_result?.value;
    let list_ref = match resolved_list.as_ref() {
        // FIXME: Inconsistent with object/primitive, is it really what we want?
        serde_json::Value::Null => Cow::Owned(Vec::new()),
        serde_json::Value::Array(ref arr) => Cow::Borrowed(arr),
        _ => {
            return Err(ServerError::new(
                "An internal error happened",
                Some(ctx_field.item.pos),
            ));
        }
    };

    match resolve_list(&ctx_field, list_ref).await {
        result @ Ok(_) => result,
        Err(err) => {
            if ctx_field.meta.is_required() {
                Err(err)
            } else {
                ctx_field.add_error(err);
                Ok(ctx_field
                    .response_graph
                    .write()
                    .await
                    .new_node_unchecked(ResponsePrimitive::new(ConstValue::Null).into()))
            }
        }
    }
}

async fn run_meta_field_resolver<'ctx>(
    ctx_field: &DynamicFieldContext<'ctx>,
    maybe_parent_value: Option<&ResolvedValue<'ctx>>,
) -> ServerResult<ResolvedValue<'ctx>> {
    let resolver = ctx_field.meta.resolver.as_ref().ok_or_else(|| {
        ServerError::new(
            format!("Undefined resolver for {:?}", ctx_field.path_node),
            Some(ctx_field.item.pos),
        )
    })?;
    resolver
        .resolve_dynamic(ctx_field, maybe_parent_value)
        .await
}

pub fn resolve_input(
    ctx_field: &DynamicFieldContext<'_>,
    meta_input_value: &MetaInputValue,
    value: ConstValue,
) -> ServerResult<serde_json::Value> {
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
) -> Result<serde_json::Value, Error> {
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
                    Ok(serde_json::Value::Array(arr))
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
                            let mut map = serde_json::Map::new();
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
                                // Not adding null fields makes serde_json::Value easier to work with, as
                                // Value::get(key) would return Some(Null) instead of None.
                                if !field_value.is_null() {
                                    map.insert(name.to_string(), field_value);
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
                                Ok(serde_json::Value::Object(map))
                            }
                        } else {
                            Err(input_error("Expected an Object", path))
                        }
                    }
                    MetaType::Enum { .. } | MetaType::Scalar { .. } => {
                        Ok(serde_json::to_value(value)?)
                    }
                    _ => Err(input_error(
                        &format!("Internal Error: Unsupported input type {type_name}"),
                        path,
                    )),
                }
            }
        }
    } else {
        match default_value {
            Some(v) => Ok(serde_json::to_value(v)?),
            None => match MetaTypeName::create(&ty) {
                MetaTypeName::NonNull(_) => Err(input_error("Unexpected null value", path)),
                _ => Ok(serde_json::Value::Null),
            },
        }
    }
}

fn input_error(expected: &str, path: &[String]) -> Error {
    Error::new(format!("{expected} for {}", path.join(".")))
}
