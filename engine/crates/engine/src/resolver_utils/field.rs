use engine_parser::parse_selection_set;
use engine_value::ConstValue;
use graph_entities::{CompactValue, NodeID, ResponseNodeId, ResponsePrimitive};
use serde_json::Value;

use super::{introspection, joins::resolve_joined_field, resolve_container, resolve_list};
use crate::{
    registry::{
        resolvers::{ResolvedValue, Resolver, ResolverContext},
        scalars::{DynamicScalar, PossibleScalar},
        type_kinds::OutputType,
        FieldSet, MetaField, MetaType, ScalarParser, TypeReference,
    },
    Context, ContextExt, ContextField, Error, QueryPathSegment, ServerError,
};

/// Resolves the field inside `ctx` within the type `root`
pub async fn resolve_field(
    ctx: &ContextField<'_>,
    parent_resolver_value: Option<ResolvedValue>,
) -> Result<ResponseNodeId, ServerError> {
    let introspection_enabled = !ctx.schema_env.registry.disable_introspection && !ctx.query_env.disable_introspection;

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

    if let Some(requires) = &field.requires {
        parent_resolver_value = resolve_requires_fieldset(parent_resolver_value, requires, ctx).await?;
    }

    // TODO: Ok, so _here_
    // _if_ the current field has a Join resolver on it.
    // Then we don't bother going into any of the below functions.
    //
    // And instead we generate a _new_ SelectionSet out of the
    // join select with the queries select _inside_ and then resolve that.
    // and start resolving that instead?
    //
    // This would absolutely work but ofc would fuck any error paths up somewhat horribly.
    //
    // We could just translate these back to normal...?
    // _or_ we could go down a slightly different code path for joins.
    //
    // Which may be a better option?  Not sure....
    //
    // I guess we've essentially ruled out the presence of arrays in the join path.
    // We can have scalars, but not until the leaf field.
    // No spreads.
    // It's _mostly_ containers w/ one entry.
    // Ban __typenam
    // So the normal code path is basically:
    //
    // resolve_container
    // -> FieldExecutionSet::add_selection_set (w/ one field)
    // -> FieldExecutionSet::add_field
    // creates a future that does:
    //  -> resolve_field
    //  -> resolve_container_field
    //  -> run_field_resolver -> resolve_container
    //  -> resolve_container etc.
    // resolves all futures
    //
    // I may be missing something, but seems like it would be easy to bypass much of
    // that heirarchy and just do a `resolve_join` that takes the simple path we need...?
    //
    // Dumb join version:
    // fn resolve_join(
    //
    //   for field in the thing {
    //      let current_value = run_field_resolver(resolver, last_value);
    //      let context = dunno_but_do_this_somehow()
    //      let last_value = current_value;
    //   }
    // }
    //
    // at the end of the loop current_value _should_ be the value we use for the current field.
    // (e.g. the return value of run_field_resolver, assuming we were able to implement
    // this in there (can we - there's a question?))

    //

    let result = match CurrentResolverType::new(field, ctx) {
        CurrentResolverType::PRIMITIVE => resolve_primitive_field(ctx, field, parent_resolver_value).await,
        CurrentResolverType::CONTAINER => resolve_container_field(ctx, field, parent_resolver_value).await,
        CurrentResolverType::ARRAY => resolve_array_field(ctx, field, parent_resolver_value).await,
    }
    .map_err(|error| ctx.set_error_path(error));

    match result {
        Ok(result) => Ok(result),
        Err(e) if field.ty.is_nullable() => {
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
    field: &MetaField,
    parent_resolver_value: ResolvedValue,
) -> Result<ResponseNodeId, ServerError> {
    let resolved_value = run_field_resolver(ctx, parent_resolver_value)
        .await
        .map_err(|err| err.into_server_error(ctx.item.pos));

    let result = match resolved_value {
        Ok(result) => {
            if field.ty.is_non_null() && *result.data_resolved() == serde_json::Value::Null {
                log::warn!(
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

    let parent_type_name = ctx.parent_type.name();

    let result = match field_type {
        OutputType::Scalar(scalar) => match scalar.parser {
            ScalarParser::PassThrough => {
                let scalar_value: ConstValue = result
                    .try_into()
                    .map_err(|err: serde_json::Error| ServerError::new(err.to_string(), Some(ctx.item.pos)))?;

                field
                    .check_cache_tag(ctx, parent_type_name, &field.name, Some(&scalar_value))
                    .await;

                scalar_value
            }
            ScalarParser::BestEffort => match result {
                serde_json::Value::Null => ConstValue::Null,
                _ => {
                    let scalar_value = PossibleScalar::to_value(field.ty.named_type().as_str(), result)
                        .map_err(|err| err.into_server_error(ctx.item.pos))?;

                    field
                        .check_cache_tag(ctx, parent_type_name, &field.name, Some(&scalar_value))
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
    parent_resolver_value: ResolvedValue,
) -> Result<ResponseNodeId, ServerError> {
    // If there is a resolver associated to the container we execute it before
    // asking to resolve the other fields
    let resolved_value = run_field_resolver(ctx, parent_resolver_value)
        .await
        .map_err(|err| err.into_server_error(ctx.item.pos))?;

    if resolved_value.data_resolved().is_null() {
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
        .node_id(field_type.name())
        .and_then(|x| NodeID::from_owned(x).ok());

    let type_name = field_type.name().to_string();

    let selection_ctx = ctx.with_selection_set(&ctx.item.node.selection_set);

    match resolve_container(&selection_ctx, node_id, resolved_value).await {
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
    parent_resolver_value: ResolvedValue,
) -> Result<ResponseNodeId, ServerError> {
    let registry = ctx.registry();
    let container_type = registry
        .lookup_expecting::<&MetaType>(&field.ty)
        .map_err(|error| error.into_server_error(ctx.item.pos))?;

    let resolved_value = run_field_resolver(ctx, parent_resolver_value)
        .await
        .map_err(|err| err.into_server_error(ctx.item.pos))?;

    field
        .check_cache_tag(ctx, container_type.name(), &field.name, None)
        .await;

    let list_ctx = ctx.to_list_context();
    resolve_list(list_ctx, ctx.item, container_type, resolved_value).await
}

pub(super) async fn run_field_resolver(
    ctx: &ContextField<'_>,
    parent_resolver_value: ResolvedValue,
) -> Result<ResolvedValue, Error> {
    let resolver = &ctx.field.resolver;

    if let Some(QueryPathSegment::Index(idx)) = ctx.path.last() {
        // Items in lists don't have resolvers - we just look them up by index
        return Ok(parent_resolver_value.get_index(*idx).unwrap_or_default());
    }

    match resolver {
        Resolver::Parent => {
            // Some fields just pass their parents data down to their children (or have no data at all).
            // For those, we early return with the parent data
            return Ok(parent_resolver_value);
        }
        Resolver::Join(join) => {
            return resolve_joined_field(ctx, join, parent_resolver_value).await;
        }
        _ => {}
    }

    resolver
        .resolve(ctx, &ResolverContext::new(ctx), Some(parent_resolver_value))
        .await
}

async fn resolve_requires_fieldset(
    parent_resolver_value: ResolvedValue,
    requires: &FieldSet,
    ctx: &ContextField<'_>,
) -> Result<ResolvedValue, ServerError> {
    let all_fields_present = match parent_resolver_value.data_resolved() {
        Value::Object(object) => requires.all_fields_are_present(object),
        _ => false,
    };

    if all_fields_present {
        return Ok(parent_resolver_value);
    }

    let selection_set_string = format!("{{ {requires} }}");
    let selection_set = parse_selection_set(&selection_set_string).map_err(|error| {
        log::error!(
            ctx.trace_id(),
            "Could not parse require string `{selection_set_string} as selection set: {error}"
        );
        ServerError::new("Internal error processing @requires", None)
    })?;

    let require_context = ctx.with_requires_selection_set(&selection_set);

    let node_id = resolve_container(&require_context, None, parent_resolver_value.clone()).await?;

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
