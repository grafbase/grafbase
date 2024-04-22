use engine_parser::parse_selection_set;
use engine_value::ConstValue;
use graph_entities::{CompactValue, ResponseNodeId, ResponsePrimitive};
use serde_json::Value;

use super::{introspection, joins::resolve_joined_field, resolve_container, resolve_list};
use crate::{
    registry::{
        check_field_cache_tag,
        field_set::{all_fieldset_fields_are_present, FieldSetDisplay},
        resolvers::{ResolvedValue, ResolverContext},
        scalars::{DynamicScalar, PossibleScalar},
    },
    request::IntrospectionState,
    Context, ContextExt, ContextField, Error, ServerError,
};
use registry_v2::{resolvers::Resolver, MetaType, ScalarParser};

/// Resolves the field inside `ctx` within the type `root`
pub async fn resolve_field(
    ctx: &ContextField<'_>,
    parent_resolver_value: Option<ResolvedValue>,
) -> Result<ResponseNodeId, ServerError> {
    let introspection_enabled = match ctx.query_env.introspection_state {
        IntrospectionState::ForceEnabled => true,
        IntrospectionState::ForceDisabled => false,
        IntrospectionState::UserPreference => !ctx.schema_env.registry.disable_introspection,
    };

    if ctx.item.node.name.node == "__schema" {
        if introspection_enabled {
            return introspection::resolve_schema_field(ctx)
                .await
                .and_then(|opt| opt.ok_or_else(|| ServerError::new("Unknown internal introspection error", None)))
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
                .and_then(|opt| opt.ok_or_else(|| ServerError::new("Unknown internal introspection error", None)))
                .map_err(|error| ctx.set_error_path(error));
        } else {
            return Err(ServerError::new(
                "Unauthorized for introspection.",
                Some(ctx.item.node.name.pos),
            ));
        }
    }

    let Some(field) = ctx.parent_type.field(ctx.item.node.name.node.as_str()) else {
        return Err(ServerError::new(
            format!(
                "Could not find a field named {} on {}",
                ctx.item.node.name.node,
                ctx.parent_type.name()
            ),
            Some(ctx.item.node.name.pos),
        ));
    };

    let mut parent_resolver_value = parent_resolver_value.unwrap_or_default();

    if let Some(requires) = field.requires() {
        parent_resolver_value = resolve_requires_fieldset(parent_resolver_value, requires, ctx).await?;
    }

    let result = match CurrentResolverType::new(field, ctx) {
        CurrentResolverType::PRIMITIVE => resolve_primitive_field(ctx, field, parent_resolver_value).await,
        CurrentResolverType::CONTAINER => resolve_container_field(ctx, field, parent_resolver_value).await,
        CurrentResolverType::ARRAY => resolve_array_field(ctx, field, parent_resolver_value).await,
    }
    .map_err(|error| ctx.set_error_path(error));

    match result {
        Ok(result) => Ok(result),
        Err(e) if field.ty().is_nullable() => {
            ctx.add_error(e);
            Ok(ctx.response().await.insert_node(CompactValue::Null))
        }
        Err(error) => {
            // Propagate the error to parents who can add it to the response and null things out
            Err(error)
        }
    }
}

async fn resolve_primitive_field(
    ctx: &ContextField<'_>,
    field: registry_v2::MetaField<'_>,
    parent_resolver_value: ResolvedValue,
) -> Result<ResponseNodeId, ServerError> {
    let resolved_value = run_field_resolver(ctx, parent_resolver_value)
        .await
        .map_err(|err| err.into_server_error(ctx.item.pos));

    let result = match resolved_value {
        Ok(Some(result)) if result.data_resolved().is_null() => handle_null_primitive(field, ctx),
        Ok(None) => handle_null_primitive(field, ctx),
        Ok(Some(result)) => Ok(result.take()),
        Err(err) => return Err(err),
    }?;

    let field_type = field.ty().named_type();

    let parent_type_name = ctx.parent_type.name();

    let result = match field_type {
        MetaType::Scalar(scalar) => match scalar.parser() {
            ScalarParser::PassThrough => {
                let scalar_value: ConstValue = result
                    .try_into()
                    .map_err(|err: serde_json::Error| ServerError::new(err.to_string(), Some(ctx.item.pos)))?;

                check_field_cache_tag(ctx, parent_type_name, field.name(), Some(&scalar_value)).await;

                scalar_value
            }
            ScalarParser::BestEffort => match result {
                serde_json::Value::Null => ConstValue::Null,
                _ => {
                    let scalar_value = PossibleScalar::to_value(field.ty().named_type().name(), result)
                        .map_err(|err| err.into_server_error(ctx.item.pos))?;

                    check_field_cache_tag(ctx, parent_type_name, field.name(), Some(&scalar_value)).await;

                    scalar_value
                }
            },
        },
        registry_v2::MetaType::Enum { .. } => {
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

fn handle_null_primitive(field: registry_v2::MetaField<'_>, ctx: &ContextField<'_>) -> Result<Value, ServerError> {
    if field.ty().is_non_null() {
        tracing::warn!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "message": "Something went wrong here",
                "expected": serde_json::Value::String(field.ty().to_string()),
                "path": serde_json::Value::String(ctx.path.to_string()),
            }))
            .unwrap(),
        );
        return Err(ServerError::new(
            format!(
                "An error happened while fetching `{}`, expected a non null value but found a null",
                field.name()
            ),
            Some(ctx.item.pos),
        ));
    }

    Ok(serde_json::Value::Null)
}

async fn resolve_container_field(
    ctx: &ContextField<'_>,
    field: registry_v2::MetaField<'_>,
    parent_resolver_value: ResolvedValue,
) -> Result<ResponseNodeId, ServerError> {
    // If there is a resolver associated to the container we execute it before
    // asking to resolve the other fields
    let resolved_value = run_field_resolver(ctx, parent_resolver_value)
        .await
        .map_err(|err| err.into_server_error(ctx.item.pos))?;

    if resolved_value.is_none() {
        if field.ty().is_non_null() {
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
    let resolved_value = resolved_value.unwrap();

    let field_type = field.ty().named_type();

    let type_name = field_type.name().to_string();

    let selection_ctx = ctx.with_selection_set(&ctx.item.node.selection_set);

    match resolve_container(&selection_ctx, resolved_value).await {
        result @ Ok(_) => {
            check_field_cache_tag(ctx, &type_name, field.name(), None).await;
            result
        }
        Err(err) => {
            if field.ty().is_non_null() {
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
    field: registry_v2::MetaField<'_>,
    parent_resolver_value: ResolvedValue,
) -> Result<ResponseNodeId, ServerError> {
    let registry = ctx.registry();
    let container_type = field.ty().named_type();

    let resolved_value = run_field_resolver(ctx, parent_resolver_value)
        .await
        .map_err(|err| err.into_server_error(ctx.item.pos))?
        .unwrap_or_default();

    check_field_cache_tag(ctx, container_type.name(), field.name(), None).await;

    let list_ctx = ctx.to_list_context();
    resolve_list(list_ctx, ctx.item, container_type, resolved_value).await
}

pub(super) async fn run_field_resolver(
    ctx: &ContextField<'_>,
    parent_resolver_value: ResolvedValue,
) -> Result<Option<ResolvedValue>, Error> {
    let resolver = &ctx.field.resolver();

    match resolver {
        Resolver::Parent => {
            // Some fields just pass their parents data down to their children (or have no data at all).
            // For those we early return with the parent data
            return Ok(Some(parent_resolver_value));
        }
        Resolver::Join(join) => {
            return resolve_joined_field(ctx, join, parent_resolver_value).await.map(Some);
        }
        _ => {}
    }

    let resolved_value = crate::registry::resolvers::run_resolver(
        resolver,
        ctx,
        &ResolverContext::new(ctx),
        Some(parent_resolver_value),
    )
    .await?;

    if resolved_value.data_resolved().is_null() {
        // Convert nulls into `None` which will stop us executing child fields
        return Ok(None);
    }

    Ok(Some(resolved_value))
}

async fn resolve_requires_fieldset(
    parent_resolver_value: ResolvedValue,
    requires: &registry_v2::FieldSet,
    ctx: &ContextField<'_>,
) -> Result<ResolvedValue, ServerError> {
    let all_fields_present = match parent_resolver_value.data_resolved() {
        Value::Object(object) => all_fieldset_fields_are_present(requires, object),
        _ => false,
    };

    if all_fields_present {
        return Ok(parent_resolver_value);
    }

    let selection_set_string = format!("{{ {} }}", FieldSetDisplay(requires));
    let selection_set = parse_selection_set(&selection_set_string).map_err(|error| {
        tracing::error!("Could not parse require string `{selection_set_string} as selection set: {error}");
        ServerError::new("Internal error processing @requires", None)
    })?;

    let require_context = ctx.with_requires_selection_set(&selection_set);

    let node_id = resolve_container(&require_context, parent_resolver_value.clone()).await?;

    let data = ctx
        .response()
        .await
        .take_node_into_compact_value(node_id)
        .expect("this has to work");

    Ok(ResolvedValue::new(data.into()))
}

#[derive(Debug)]
enum CurrentResolverType {
    PRIMITIVE,
    ARRAY,
    CONTAINER,
}

impl CurrentResolverType {
    fn new(current_field: registry_v2::MetaField<'_>, ctx: &ContextField<'_>) -> Self {
        if current_field.ty().is_list() {
            return CurrentResolverType::ARRAY;
        }

        // This seems... unreliable
        match &ctx.item.node.selection_set.node.items.is_empty() {
            true => CurrentResolverType::PRIMITIVE,
            false => CurrentResolverType::CONTAINER,
        }
    }
}
