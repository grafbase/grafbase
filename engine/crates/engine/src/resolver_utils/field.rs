use engine_value::ConstValue;
use graph_entities::{CompactValue, NodeID, ResponseNodeId, ResponsePrimitive};

use crate::{
    context::ResolverChainNode,
    registry::{
        resolvers::{ResolvedValue, ResolverContext},
        scalars::{DynamicScalar, PossibleScalar},
        type_kinds::OutputType,
        MetaField, MetaType, ScalarParser, TypeReference,
    },
    ContextField, Error, QueryPathSegment, ServerError,
};

use super::{introspection, resolve_container, resolve_list};

/// Resolves the field inside `ctx` within the type `root`
pub async fn resolve_field(
    ctx: &ContextField<'_>,
    parent_type: &MetaType,
    parent_resolver_value: Option<ResolvedValue>,
) -> Result<Option<ResponseNodeId>, ServerError> {
    let introspection_enabled = !ctx.schema_env.registry.disable_introspection && !ctx.query_env.disable_introspection;

    if ctx.item.node.name.node == "__schema" {
        if introspection_enabled {
            return introspection::resolve_schema_field(ctx)
                .await
                .map_err(|error| ctx.set_error_path(error));
        } else {
            return Err(ServerError::new(
                "Unauthorized for introspection.",
                Some(ctx.item.node.name.pos),
            ));
        }
    } else if ctx.item.node.name.node == "__type" {
        if introspection_enabled {
            return introspection::resolve_type_field(ctx)
                .await
                .map_err(|error| ctx.set_error_path(error));
        } else {
            return Err(ServerError::new(
                "Unauthorized for introspection.",
                Some(ctx.item.node.name.pos),
            ));
        }
    }

    let Some(field) = parent_type.field_by_name(ctx.item.node.name.node.as_str()) else {
        return Ok(None);
    };

    let result = match CurrentResolverType::new(&field, ctx) {
        CurrentResolverType::PRIMITIVE => resolve_primitive_field(ctx, parent_type, field, parent_resolver_value).await,
        CurrentResolverType::CONTAINER => resolve_container_field(ctx, field, parent_resolver_value).await,
        CurrentResolverType::ARRAY => resolve_array_field(ctx, field, parent_resolver_value).await,
    }
    .map_err(|error| ctx.set_error_path(error));

    match result {
        Ok(result) => Ok(Some(result)),
        Err(e) if field.ty.is_nullable() => {
            ctx.add_error(e);
            Ok(Some(ctx.response().await.insert_node(CompactValue::Null)))
        }
        Err(error) => {
            // Propagate the error to parents who can add it to the response and null things out
            Err(error)
        }
    }
}

async fn resolve_primitive_field(
    ctx: &ContextField<'_>,
    parent_type: &MetaType,
    field: &MetaField,
    parent_resolver_value: Option<ResolvedValue>,
) -> Result<ResponseNodeId, ServerError> {
    let resolver_node = ctx.resolver_node.as_ref().expect("shouldn't be null");
    let resolved_value = run_field_resolver(&ctx, resolver_node, parent_resolver_value)
        .await
        .map_err(|err| err.into_server_error(ctx.item.pos));

    let result = match resolved_value {
        Ok(result) => {
            if field.ty.is_non_null() && *result.data_resolved() == serde_json::Value::Null {
                #[cfg(feature = "tracing_worker")]
                logworker::warn!(
                    ctx.trace_id(),
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "message": "Something went wrong here",
                        "expected": serde_json::Value::String(field.ty.to_string()),
                        "path": serde_json::Value::String(ctx.path.to_string()),
                    }))
                    .unwrap(),
                );
                Err(ServerError::new(
                    format!(
                        "An error happened while fetching `{}`, expected a non null value but found a null",
                        field.name
                    ),
                    Some(ctx.item.pos),
                ))
            } else {
                Ok(result.take())
            }
        }
        Err(err) => return Err(err),
    }?;

    let field_type = ctx
        .registry()
        .lookup(&field.ty)
        .map_err(|error| error.into_server_error(ctx.item.pos))?;

    let parent_type_name = parent_type.name();

    let result = match field_type {
        OutputType::Scalar(scalar) => match scalar.parser {
            ScalarParser::PassThrough => {
                let scalar_value: ConstValue = result
                    .try_into()
                    .map_err(|err: serde_json::Error| ServerError::new(err.to_string(), Some(ctx.item.pos)))?;

                field
                    .check_cache_tag(ctx, &parent_type_name, &field.name, Some(&scalar_value))
                    .await;

                scalar_value
            }
            ScalarParser::BestEffort => match result {
                serde_json::Value::Null => ConstValue::Null,
                _ => {
                    let scalar_value = PossibleScalar::to_value(&field.ty.named_type().as_str(), result)
                        .map_err(|err| err.into_server_error(ctx.item.pos))?;

                    field
                        .check_cache_tag(ctx, &parent_type_name, &field.name, Some(&scalar_value))
                        .await;

                    scalar_value
                }
            },
        },
        OutputType::Enum { .. } => {
            ConstValue::from_json(result).map_err(|err| ServerError::new(err.to_string(), Some(ctx.item.pos)))?
        }
        _ => {
            return Err(ServerError::new(
                "Internal error: expected an enum or scalar type for a primitive",
                Some(ctx.item.pos),
            ));
        }
    };

    Ok(ctx.response().await.insert_node(ResponsePrimitive::new(result.into())))
}

async fn resolve_container_field(
    ctx: &ContextField<'_>,
    field: &MetaField,
    parent_resolver_value: Option<ResolvedValue>,
) -> Result<ResponseNodeId, ServerError> {
    // If there is a resolver associated to the container we execute it before
    // asking to resolve the other fields
    let resolved_value = if let Some(resolver_node) = &ctx.resolver_node {
        let resolved_value = run_field_resolver(&ctx, resolver_node, parent_resolver_value)
            .await
            .map_err(|err| err.into_server_error(ctx.item.pos))?;

        if resolved_value.is_early_returned() {
            if field.ty.is_non_null() {
                return Err(ServerError::new(
                        format!(
                            "An error occured while fetching `{}`, a non-nullable value was expected but no value was found.",
                            ctx.item.node.name.node
                        ),
                        Some(ctx.item.pos),
                    ));
            } else {
                return Ok(ctx
                    .response()
                    .await
                    .insert_node(ResponsePrimitive::new(CompactValue::Null)));
            }
        }
        Some(resolved_value)
    } else {
        None
    };

    let field_type = ctx
        .registry()
        .lookup_expecting::<&MetaType>(&field.ty)
        .map_err(|error| error.into_server_error(ctx.item.pos))?;

    // TEMP: Hack
    // We can check from the schema definition if it's a node, if it is, we need to
    // have a way to get it
    // temp: Little hack here, we know that `ResolvedValue` are bound to have a format
    // of:
    // ```
    // {
    //   "Node": {
    //     "__sk": {
    //       "S": "node_id"
    //     }
    //   }
    // }
    // ```
    // We use that fact without checking it here.
    //
    // This have to be removed when we rework registry & engine to have a proper query
    // planning.
    let node_id: Option<NodeID<'_>> = resolved_value
        .as_ref()
        .and_then(|x| x.node_id(field_type.name()))
        .and_then(|x| NodeID::from_owned(x).ok());

    let type_name = field_type.name().to_string();

    let selection_ctx = ctx.with_selection_set(&ctx.item.node.selection_set);

    match resolve_container(&selection_ctx, field_type, node_id, resolved_value).await {
        result @ Ok(_) => {
            field.check_cache_tag(ctx, &type_name, &field.name, None).await;
            result
        }
        Err(err) => {
            if field.ty.is_non_null() {
                Err(err)
            } else {
                ctx.add_error(err);
                Ok(ctx
                    .response()
                    .await
                    .insert_node(ResponsePrimitive::new(CompactValue::Null)))
            }
        }
    }
}

async fn resolve_array_field(
    ctx: &ContextField<'_>,
    field: &MetaField,
    parent_resolver_value: Option<ResolvedValue>,
) -> Result<ResponseNodeId, ServerError> {
    let registry = ctx.registry();
    let container_type = registry
        .lookup_expecting::<&MetaType>(&field.ty)
        .map_err(|error| error.into_server_error(ctx.item.pos))?;

    let resolver_node = ctx.resolver_node.as_ref().expect("shouldn't be null");
    let resolved_value = run_field_resolver(&ctx, resolver_node, parent_resolver_value)
        .await
        .map_err(|err| err.into_server_error(ctx.item.pos))?;

    let selection_ctx = ctx.with_selection_set(&ctx.item.node.selection_set);

    field
        .check_cache_tag(ctx, container_type.name(), &field.name, None)
        .await;

    resolve_list(selection_ctx, ctx.item, &field.ty, container_type, resolved_value).await
}

async fn run_field_resolver(
    ctx: &ContextField<'_>,
    resolver_node: &ResolverChainNode<'_>,
    parent_resolver_value: Option<ResolvedValue>,
) -> Result<ResolvedValue, Error> {
    let mut final_result = parent_resolver_value.unwrap_or_default();

    if let Some(QueryPathSegment::Index(idx)) = ctx.path.last() {
        // If we are in an index segment, it means we do not have a current resolver (YET).
        final_result = final_result.get_index(*idx).unwrap_or_default();
    } else if let Some(resolver) = resolver_node.resolver {
        // Avoiding the early return when we're just propagating downwards data. Container
        // fields used as namespaces have no value (so Null) but their fields have resolvers.
        if !resolver.is_parent() {
            let current_ctx = ResolverContext::new(&resolver_node.execution_id)
                .with_ty(resolver_node.ty)
                .with_selection_set(resolver_node.selections)
                .with_field(resolver_node.field);

            final_result = resolver.resolve(ctx, &current_ctx, Some(&final_result)).await?;

            if final_result.data_resolved().is_null() {
                final_result = final_result.with_early_return();
            }
        }
    }

    Ok(final_result)
}

#[derive(Debug)]
enum CurrentResolverType {
    PRIMITIVE,
    ARRAY,
    CONTAINER,
}

impl CurrentResolverType {
    fn new(current_field: &MetaField, ctx: &ContextField<'_>) -> Self {
        if current_field.ty.is_list() {
            return CurrentResolverType::ARRAY;
        }

        // This seems... unreliable
        match &ctx.item.node.selection_set.node.items.is_empty() {
            true => CurrentResolverType::PRIMITIVE,
            false => CurrentResolverType::CONTAINER,
        }
    }
}
