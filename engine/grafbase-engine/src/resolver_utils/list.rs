use grafbase_engine_value::Name;
use graph_entities::{ResponseList, ResponseNodeId, ResponsePrimitive};

use crate::{
    extensions::ResolveInfo,
    parser::types::Field,
    registry::{
        resolvers::ResolvedValue,
        scalars::{DynamicScalar, PossibleScalar},
        MetaType,
    },
    resolver_utils::resolve_container,
    ContextSelectionSet, Error, LegacyOutputType, Positioned, ServerError, ServerResult, Value,
};

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list<'a>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    ty: &'a MetaType,
    values: Vec<ResolvedValue>,
) -> ServerResult<ResponseNodeId> {
    let extensions = &ctx.query_env.extensions;
    if !extensions.is_empty() {
        let mut futures = Vec::with_capacity(values.len());
        for (idx, item) in values.into_iter().enumerate() {
            futures.push({
                let ctx = ctx.clone();
                let type_name = ty.name();
                async move {
                    let ctx_idx = ctx.with_index(idx, Some(&ctx.item.node));
                    let extensions = &ctx.query_env.extensions;

                    let ctx_field = ctx.with_field(field, None, Some(&ctx.item.node));
                    let meta_field = ctx_field
                        .schema_env
                        .registry
                        .types
                        .get(type_name)
                        .and_then(|ty| ty.field_by_name(field.node.name.node.as_str()));

                    let parent_type = format!("[{type_name}]");
                    let return_type = format!("{type_name}!").into();
                    let args_values: Vec<(Positioned<Name>, Option<Value>)> = ctx_field
                        .item
                        .node
                        .arguments
                        .clone()
                        .into_iter()
                        .map(|(key, val)| (key, ctx_field.resolve_input_value(val).ok()))
                        .collect();

                    let resolve_info = ResolveInfo {
                        path_node: ctx_idx.path_node.as_ref().unwrap(),
                        parent_type: &parent_type,
                        return_type: &return_type,
                        name: field.node.name.node.as_str(),
                        alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                        required_operation: meta_field.and_then(|f| f.required_operation),
                        auth: meta_field.and_then(|f| f.auth.as_ref()),
                        input_values: args_values,
                    };

                    let resolve_fut = async {
                        match ty {
                            MetaType::Scalar(_) | MetaType::Enum(_) => {
                                let mut result = Value::try_from(item.take()).map_err(|err| {
                                    ctx_idx.set_error_path(ServerError::new(format!("{err:?}"), Some(field.pos)))
                                })?;
                                // Yes it's ugly...
                                if let MetaType::Scalar(_) = ty {
                                    result = resolve_scalar(result, type_name)
                                        .map_err(|err| err.into_server_error(field.pos))?;
                                }
                                Ok(Some(
                                    ctx_idx
                                        .response_graph
                                        .write()
                                        .await
                                        .insert_node(ResponsePrimitive::new(result.into())),
                                ))
                            }
                            _ => resolve_container(&ctx_idx, ty, None, Some(item))
                                .await
                                .map(Option::Some)
                                .map_err(|err| ctx_idx.set_error_path(err)),
                        }
                    };
                    futures_util::pin_mut!(resolve_fut);
                    extensions
                        .resolve(resolve_info, &mut resolve_fut)
                        .await
                        .map(|value| value.expect("You definitely encountered a bug!"))
                }
            });
        }
        let node = ResponseList::with_children(futures_util::future::try_join_all(futures).await?);

        Ok(ctx.response_graph.write().await.insert_node(node))
    } else {
        let mut futures = Vec::with_capacity(values.len());
        for (idx, item) in values.into_iter().enumerate() {
            let ctx_idx = ctx.with_index(idx, Some(&ctx.item.node));
            futures.push(async move {
                match ty {
                    MetaType::Scalar { .. } | MetaType::Enum { .. } => {
                        let result = Value::try_from(item.take()).map_err(|err| {
                            ctx_idx.set_error_path(ServerError::new(format!("{err:?}"), Some(field.pos)))
                        })?;

                        Ok(ctx_idx
                            .response_graph
                            .write()
                            .await
                            .insert_node(ResponsePrimitive::new(result.into())))
                    }
                    _ => resolve_container(&ctx_idx, ty, None, Some(item))
                        .await
                        .map_err(|err| ctx_idx.set_error_path(err)),
                }
            });
        }

        let node = ResponseList::with_children(futures_util::future::try_join_all(futures).await?);

        Ok(ctx.response_graph.write().await.insert_node(node))
    }
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
        ),
    }
}

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list_native<'a, T: LegacyOutputType + 'a>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    iter: impl IntoIterator<Item = T>,
    len: Option<usize>,
) -> ServerResult<ResponseNodeId> {
    let extensions = &ctx.query_env.extensions;
    if !extensions.is_empty() {
        let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
        for (idx, item) in iter.into_iter().enumerate() {
            futures.push({
                let ctx = ctx.clone();
                async move {
                    let ctx_idx = ctx.with_index(idx, Some(&ctx.item.node));
                    let extensions = &ctx.query_env.extensions;

                    let type_name = <T>::type_name();
                    let ctx_field = ctx.with_field(field, None, Some(&ctx.item.node));
                    let meta_field = ctx_field
                        .schema_env
                        .registry
                        .types
                        .get(type_name.as_ref())
                        .and_then(|ty| ty.field_by_name(field.node.name.node.as_str()));

                    let resolve_info = ResolveInfo {
                        path_node: ctx_idx.path_node.as_ref().unwrap(),
                        parent_type: &Vec::<T>::type_name(),
                        return_type: &T::qualified_type_name(),
                        name: field.node.name.node.as_str(),
                        alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                        required_operation: meta_field.and_then(|f| f.required_operation),
                        auth: meta_field.and_then(|f| f.auth.as_ref()),
                        input_values: Vec::new(), // Isn't needed for static resolve
                    };
                    let resolve_fut = async {
                        LegacyOutputType::resolve(&item, &ctx_idx, field)
                            .await
                            .map(Option::Some)
                            .map_err(|err| ctx_idx.set_error_path(err))
                    };
                    futures_util::pin_mut!(resolve_fut);
                    extensions
                        .resolve(resolve_info, &mut resolve_fut)
                        .await
                        .map(|value| value.expect("You definitely encountered a bug!"))
                }
            });
        }
        let children = futures_util::future::try_join_all(futures).await?;

        let node_id = ctx
            .response_graph
            .write()
            .await
            .insert_node(ResponseList::with_children(children));

        Ok(node_id)
    } else {
        let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
        for (idx, item) in iter.into_iter().enumerate() {
            let ctx_idx = ctx.with_index(idx, Some(&ctx.item.node));
            futures.push(async move {
                LegacyOutputType::resolve(&item, &ctx_idx, field)
                    .await
                    .map_err(|err| ctx_idx.set_error_path(err))
            });
        }

        let children = futures_util::future::try_join_all(futures).await?;

        let node_id = ctx
            .response_graph
            .write()
            .await
            .insert_node(ResponseList::with_children(children));

        Ok(node_id)
    }
}
