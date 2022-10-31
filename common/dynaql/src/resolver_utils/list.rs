use crate::extensions::ResolveInfo;
use crate::parser::types::Field;
use crate::registry::MetaType;
use crate::resolver_utils::resolve_container;
use crate::{ContextSelectionSet, OutputType, Positioned, ServerError, ServerResult, Value};

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list<'a>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    ty: &'a MetaType,
    values: Vec<serde_json::Value>,
) -> ServerResult<Value> {
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

                    let parent_type = format!("[{}]", type_name);
                    let return_type = format!("{}!", type_name);
                    let resolve_info = ResolveInfo {
                        path_node: ctx_idx.path_node.as_ref().unwrap(),
                        parent_type: &parent_type,
                        return_type: &return_type,
                        name: field.node.name.node.as_str(),
                        alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                        required_operation: meta_field.and_then(|f| f.required_operation),
                        auth: meta_field.and_then(|f| f.auth.as_ref()),
                    };

                    let resolve_fut = async {
                        match ty {
                            MetaType::Scalar { .. } | MetaType::Enum { .. } => {
                                Value::try_from(item).map(Some).map_err(|err| {
                                    ctx_idx.set_error_path(ServerError::new(
                                        format!("{:?}", err),
                                        Some(field.pos),
                                    ))
                                })
                            }
                            _ => resolve_container(&ctx_idx, ty)
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
        Ok(Value::List(
            futures_util::future::try_join_all(futures).await?,
        ))
    } else {
        let mut futures = Vec::with_capacity(values.len());
        for (idx, item) in values.into_iter().enumerate() {
            let ctx_idx = ctx.with_index(idx, Some(&ctx.item.node));
            futures.push(async move {
                match ty {
                    MetaType::Scalar { .. } | MetaType::Enum { .. } => Value::try_from(item)
                        .map_err(|err| {
                            ctx_idx.set_error_path(ServerError::new(
                                format!("{:?}", err),
                                Some(field.pos),
                            ))
                        }),
                    _ => resolve_container(&ctx_idx, ty)
                        .await
                        .map_err(|err| ctx_idx.set_error_path(err)),
                }
            });
        }
        Ok(Value::List(
            futures_util::future::try_join_all(futures).await?,
        ))
    }
}

/// Resolve an list by executing each of the items concurrently.
pub async fn resolve_list_native<'a, T: OutputType + 'a>(
    ctx: &ContextSelectionSet<'a>,
    field: &Positioned<Field>,
    iter: impl IntoIterator<Item = T>,
    len: Option<usize>,
) -> ServerResult<Value> {
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
                    };
                    let resolve_fut = async {
                        OutputType::resolve(&item, &ctx_idx, field)
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
        Ok(Value::List(
            futures_util::future::try_join_all(futures).await?,
        ))
    } else {
        let mut futures = len.map(Vec::with_capacity).unwrap_or_default();
        for (idx, item) in iter.into_iter().enumerate() {
            let ctx_idx = ctx.with_index(idx, Some(&ctx.item.node));
            futures.push(async move {
                OutputType::resolve(&item, &ctx_idx, field)
                    .await
                    .map_err(|err| ctx_idx.set_error_path(err))
            });
        }
        Ok(Value::List(
            futures_util::future::try_join_all(futures).await?,
        ))
    }
}
