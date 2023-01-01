use std::borrow::Cow;

use crate::dynamic::{DynamicFieldContext, DynamicSelectionSetContext};
use crate::extensions::ResolveInfo;
use crate::graph::selection_set_into_node;
use crate::parser::types::Field;
use crate::registry::resolvers::{ResolvedContainer, ResolvedValue};
use crate::registry::MetaType;
use crate::resolver_utils::resolve_container;
use crate::{ContextSelectionSet, OutputType, Positioned, ServerError, ServerResult};
use dynaql_value::{ConstValue, Name};
use graph_entities::{QueryResponseNode, ResponseList, ResponseNodeId, ResponsePrimitive};

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list<'ctx>(
    ctx_field: &DynamicFieldContext<'ctx>,
    parent_resolved_list: Cow<'ctx, Vec<serde_json::Value>>,
) -> ServerResult<ResponseNodeId> {
    let ctx_field_selection = ctx_field.get_selection();
    let extensions = &ctx_field.query_env.extensions;
    let field_item = ctx_field.item;
    let meta_field = ctx_field.meta;
    if !extensions.is_empty() {
        let mut futures = Vec::with_capacity(parent_resolved_list.len());
        for (idx, parent_resolved_element) in parent_resolved_list.iter().enumerate() {
            let ctx_index = ctx_field_selection.dynamic_with_index(idx);
            futures.push(async move {
                let args_values: Vec<(Positioned<Name>, Option<ConstValue>)> = field_item
                    .node
                    .arguments
                    .clone()
                    .into_iter()
                    .map(|(key, val)| (key, ctx_field.resolve_input_value(val).ok()))
                    .collect();

                let type_name = ctx_field.base_type.name();
                let resolve_info = ResolveInfo {
                    path_node: ctx_index.path_node.as_ref().unwrap(),
                    parent_type: &format!("[{type_name}]"),
                    return_type: &format!("{type_name}!"),
                    name: field_item.node.name.node.as_str(),
                    alias: field_item
                        .node
                        .alias
                        .as_ref()
                        .map(|alias| alias.node.as_str()),
                    required_operation: meta_field.required_operation,
                    auth: meta_field.auth.as_ref(),
                    input_values: args_values,
                };
                let resolve_fut = async {
                    resolve_list_element(
                        &ctx_index,
                        ctx_field.base_type,
                        Cow::Borrowed(parent_resolved_element),
                    )
                    .await
                    .map(Some)
                };
                futures_util::pin_mut!(resolve_fut);
                extensions
                    .resolve(resolve_info, &mut resolve_fut)
                    .await
                    .map(|value| value.expect("You definitely encountered a bug!"))
            });
        }
        let node = QueryResponseNode::List(ResponseList::with_children(
            futures_util::future::try_join_all(futures).await?,
        ));

        Ok(ctx_field
            .response_graph
            .write()
            .await
            .new_node_unchecked(node))
    } else {
        let mut futures = Vec::with_capacity(parent_resolved_list.len());
        for (idx, parent_resolved_element) in parent_resolved_list.iter().enumerate() {
            let ctx_index = ctx_field_selection.dynamic_with_index(idx);
            futures.push(async move {
                resolve_list_element(
                    &ctx_index,
                    ctx_field.base_type,
                    Cow::Borrowed(parent_resolved_element),
                )
                .await
            });
        }

        let node = QueryResponseNode::List(ResponseList::with_children(
            futures_util::future::try_join_all(futures).await?,
        ));

        Ok(ctx_field
            .response_graph
            .write()
            .await
            .new_node_unchecked(node))
    }
}

async fn resolve_list_element<'ctx>(
    ctx_index: &DynamicSelectionSetContext<'ctx>,
    base_type: &MetaType,
    parent_resolved_element: Cow<'ctx, serde_json::Value>,
) -> ServerResult<ResponseNodeId> {
    match base_type {
        MetaType::Scalar { .. } | MetaType::Enum { .. } => {
            let result =
                ConstValue::try_from(parent_resolved_element.into_owned()).map_err(|err| {
                    ctx_index.set_error_path(ServerError::new(format!("{err:?}"), None))
                })?;

            Ok(ctx_index
                .response_graph
                .write()
                .await
                .new_node_unchecked(QueryResponseNode::Primitive(ResponsePrimitive::new(result))))
        }
        // TODO: node_step
        _ => {
            let parent_resolved_container =
                ResolvedContainer::new(base_type, ResolvedValue::new(parent_resolved_element));
            resolve_container(&ctx_index, Some(parent_resolved_container))
                .await
                .map_err(|err| ctx_index.set_error_path(err))
        }
    }
}

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list_native<'a, T: OutputType + 'a>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    iter: impl IntoIterator<Item = T>,
    len: Option<usize>,
) -> ServerResult<ConstValue> {
    let extensions = &ctx.query_env.extensions;
    if !extensions.is_empty() {
        let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
        for (idx, item) in iter.into_iter().enumerate() {
            futures.push({
                let ctx = ctx.clone();
                async move {
                    let ctx_idx = ctx.with_index(idx);
                    let extensions = &ctx.query_env.extensions;

                    let type_name = <T>::type_name();
                    let ctx_field = ctx.with_field(field);
                    let meta_field = ctx_field
                        .schema_env
                        .registry
                        .types
                        .get(type_name.as_ref())
                        .and_then(|ty| ty.field_by_name(field.node.name.node.as_str()));
                    let ty = ctx_field.schema_env.registry.types.get(type_name.as_ref());

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
                        let a = selection_set_into_node(
                            OutputType::resolve(&item, &ctx_idx, field)
                                .await
                                .map(Option::Some)
                                .map_err(|err| ctx_idx.set_error_path(err))?
                                .unwrap_or_default(),
                            &ctx_idx.to_dynamic(ty.unwrap()),
                        )
                        .await;
                        Ok(Some(a))
                    };
                    futures_util::pin_mut!(resolve_fut);
                    extensions
                        .resolve(resolve_info, &mut resolve_fut)
                        .await
                        .map(|value| value.expect("You definitely encountered a bug!"))
                }
            });
        }
        let a = futures_util::future::try_join_all(futures).await?;
        let node = QueryResponseNode::List(ResponseList::with_children(a));
        let response_graph = ctx.response_graph.read().await;
        let result = response_graph
            .transform_node_to_const_value(&node)
            .map_err(|_| {
                ctx.set_error_path(ServerError::new("JSON serialization failure.", None))
            })?;
        Ok(result)
    } else {
        let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
        for (idx, item) in iter.into_iter().enumerate() {
            let ctx_idx = ctx.with_index(idx);
            futures.push(async move {
                OutputType::resolve(&item, &ctx_idx, field)
                    .await
                    .map_err(|err| ctx_idx.set_error_path(err))
            });
        }
        Ok(ConstValue::List(
            futures_util::future::try_join_all(futures).await?,
        ))
    }
}
