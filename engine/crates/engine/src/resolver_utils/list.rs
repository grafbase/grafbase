use std::future::Future;

use async_runtime::make_send_on_wasm;
use engine_scalars::{DynamicScalar, PossibleScalar};
use engine_value::Name;
use futures_util::future::BoxFuture;
use graph_entities::{CompactValue, QueryResponseNode, ResponseList, ResponseNodeId, ResponsePrimitive};

use crate::{
    extensions::ResolveInfo,
    parser::types::Field,
    registry::{resolvers::ResolvedValue, type_kinds::OutputType},
    resolver_utils::resolve_container,
    Context, ContextExt, ContextField, ContextList, ContextSelectionSetLegacy, ContextWithIndex, Error,
    LegacyOutputType, Positioned, ServerError, ServerResult, Value,
};

/// Resolve a list by executing each of the items concurrently.
pub async fn resolve_list<'a>(
    ctx: ContextList<'a>,
    field: &'a Positioned<Field>,
    inner_ty: registry_v2::MetaType<'a>,
    value: ResolvedValue,
) -> ServerResult<ResponseNodeId> {
    #[async_recursion::async_recursion]
    async fn inner(
        ctx: ContextList<'async_recursion>,
        field: &Positioned<Field>,
        ty: registry_v2::MetaType<'async_recursion>,
        value: ResolvedValue,
    ) -> Result<ResponseNodeId, ServerError> {
        // First we need to make sure our parent resolve data actually has a list
        // (or return null early if we're on a nullable list)
        let items = match value.data_resolved() {
            serde_json::Value::Null if ctx.list_is_nullable() => {
                return Ok(ctx.response().await.insert_node(CompactValue::Null));
            }
            serde_json::Value::Null => {
                return Err(ctx.set_error_path(ServerError::new(
                    format!(
                        "An error occurred while fetching `{}`, a non-nullable value was expected but no value was found.",
                        field.node.name.node
                    ),
                    Some(ctx.pos()),
                )));
            }
            serde_json::Value::Array(_) => value.item_iter().expect("we checked its an array").collect::<Vec<_>>(),
            value => {
                return Err(ctx.set_error_path(ServerError::new(
                    format!("Encountered a {} where we expected a list", json_kind_str(value)),
                    Some(ctx.pos()),
                )));
            }
        };

        let futures = items.into_iter().enumerate().map(|(idx, item)| -> BoxFuture<'_, _> {
            if item.data_resolved().is_null() {
                // If the current item is null we should just stop executing here and return null
                let ctx = ctx.clone();
                return Box::pin(async move { Ok(ctx.response().await.insert_node(CompactValue::Null)) });
            }

            match ctx.with_index(idx) {
                ContextWithIndex::Field(field_ctx) => {
                    Box::pin(async move { resolve_leaf_field(field_ctx, item).await })
                }
                ContextWithIndex::SelectionSet(selection_ctx) => {
                    Box::pin(async move { resolve_container(&selection_ctx, item).await })
                }
                ContextWithIndex::List(list_context) => {
                    let resolve_future = inner(list_context.clone(), field, ty, item);

                    if ctx.query_env().extensions.is_empty() {
                        Box::pin(resolve_future)
                    } else {
                        Box::pin(apply_extensions(list_context, field, ty, resolve_future))
                    }
                }
            }
        });

        let mut children = vec![];
        let contents_are_non_null = ctx
            .contents_are_non_null()
            .expect("only returns none if we're not a list and we definitely are here");
        let contents_are_nullable = !contents_are_non_null;

        for (index, result) in futures_util::future::join_all(futures).await.into_iter().enumerate() {
            // Now we need to handle error propagation and validate the nullability
            // of each of the list items
            match result {
                Ok(id) if contents_are_non_null => {
                    let found_null = match ctx.response().await.get_node(id) {
                        Some(QueryResponseNode::Primitive(value)) if value.is_null() => true,
                        None => true,
                        _ => false,
                    };
                    if found_null {
                        let mut error =
                            ServerError::new(
                                format!(
                                    "An error occurred while fetching `{}`, a non-nullable value was expected but no value was found.",
                                    field.node.name.node
                                ),
                                Some(ctx.pos()),
                            );

                        error.path = ctx.path.child(index).into_iter().collect();

                        return Err(error);
                    }
                    children.push(id);
                }
                Ok(id) => children.push(id),
                Err(error) if contents_are_nullable => {
                    ctx.add_error(error);
                    children.push(ctx.response().await.insert_node(CompactValue::Null));
                }
                Err(error) => return Err(error),
            }
        }

        Ok(ctx.response().await.insert_node(ResponseList::with_children(children)))
    }

    inner(ctx, field, inner_ty, value).await
}

fn apply_extensions<'a>(
    ctx: ContextList<'a>,
    field: &'a Positioned<Field>,
    inner_ty: registry_v2::MetaType<'a>,
    resolve_fut: impl Future<Output = Result<ResponseNodeId, ServerError>> + Send + 'a,
) -> impl Future<Output = Result<ResponseNodeId, ServerError>> + 'a {
    let ctx = ctx.clone();
    let type_name = inner_ty.name();
    async move {
        let ctx_field = ctx.field_context;
        let meta_field = ctx_field
            .schema_env
            .registry
            .lookup_type(type_name)
            .and_then(|ty| ty.field(field.node.name.node.as_str()));

        let parent_type = format!("[{type_name}]");
        // let return_type = format!("{type_name}!").into();
        let return_type = todo!("is this even called anymore?");
        let args_values: Vec<(Positioned<Name>, Option<Value>)> = ctx_field
            .item
            .node
            .arguments
            .clone()
            .into_iter()
            .map(|(key, val)| (key, ctx_field.resolve_input_value(val).ok()))
            .collect();

        let resolve_info = ResolveInfo {
            path: ctx.path.clone(),
            parent_type: &parent_type,
            return_type,
            name: field.node.name.node.as_str(),
            alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
            required_operation: meta_field.and_then(|f| f.required_operation().cloned()),
            auth: meta_field.and_then(|f| f.auth()),
            input_values: args_values,
        };

        let resolve_fut = async move { Ok(Some(resolve_fut.await?)) };
        futures_util::pin_mut!(resolve_fut);
        ctx.query_env()
            .extensions
            .resolve(resolve_info, &mut resolve_fut)
            .await
            .map(|value| value.expect("You definitely encountered a bug!"))
    }
}

async fn resolve_leaf_field(ctx: ContextField<'_>, item: ResolvedValue) -> Result<ResponseNodeId, ServerError> {
    let mut result = Value::try_from(item.take()).map_err(|err| Error::new(format!("{err:?}")));

    // Yes it's ugly...
    if let OutputType::Scalar(scalar) = ctx.field_base_type() {
        result = result.and_then(|value| resolve_scalar(value, scalar.name()));
    }

    let item = result.map_err(|error| ctx.set_error_path(error.into_server_error(ctx.item.pos)))?;

    Ok(ctx.response().await.insert_node(ResponsePrimitive::new(item.into())))
}

fn resolve_scalar(value: Value, base_type_name: &str) -> Result<Value, Error> {
    if value == Value::Null {
        return Ok(value);
    }
    match value {
        Value::Null => Ok(Value::Null),
        Value::List(list) => list
            .into_iter()
            .map(|value| resolve_scalar(value, base_type_name))
            .collect::<Result<Vec<Value>, Error>>()
            .map(Value::List),
        value => PossibleScalar::to_value(
            base_type_name,
            serde_json::to_value(value).expect("ConstValue can always be transformed into a json"),
        )
        .map_err(|err| Error::new(err.0)),
    }
}

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list_native<'a, T: LegacyOutputType + 'a>(
    ctx: &ContextSelectionSetLegacy<'a>,
    field: &Positioned<Field>,
    iter: impl IntoIterator<Item = T>,
    len: Option<usize>,
) -> ServerResult<ResponseNodeId> {
    let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
    for (idx, item) in iter.into_iter().enumerate() {
        let ctx_idx = ctx.with_index(idx);
        futures.push(make_send_on_wasm(async move {
            LegacyOutputType::resolve(&item, &ctx_idx, field)
                .await
                .map_err(|err| ctx_idx.set_error_path(err))
        }));
    }

    let children = futures_util::future::try_join_all(futures).await?;

    let node_id = ctx.response().await.insert_node(ResponseList::with_children(children));

    Ok(node_id)
}

fn json_kind_str(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "list",
        serde_json::Value::Object(_) => "object",
    }
}
