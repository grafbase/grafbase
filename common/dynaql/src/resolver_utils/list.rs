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
    len: usize,
) -> ServerResult<Value> {
    let extensions = &ctx.query_env.extensions;
    if !extensions.is_empty() {
        let mut futures = Vec::with_capacity(len);
        for idx in 0..len {
            futures.push({
                let ctx = ctx.clone();
                let type_name = ty.name();
                async move {
                    let ctx_idx = ctx.with_index(idx);
                    let extensions = &ctx.query_env.extensions;

                    let resolve_info = ResolveInfo {
                        path_node: ctx_idx.path_node.as_ref().unwrap(),
                        parent_type: &type_name,
                        return_type: match ctx_idx
                            .schema_env
                            .registry
                            .types
                            .get(type_name)
                            .and_then(|ty| ty.field_by_name(field.node.name.node.as_str()))
                            .map(|field| &field.ty)
                        {
                            Some(ty) => &ty,
                            None => {
                                return Err(ServerError::new(
                                    r#"An internal error happened"#.to_string(),
                                    Some(ctx_idx.item.pos),
                                ));
                            }
                        },
                        name: field.node.name.node.as_str(),
                        alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                    };
                    let resolve_fut = async {
                        resolve_container(&ctx_idx, ty)
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
        let mut futures = Vec::with_capacity(len);
        for idx in 0..len {
            let ctx_idx = ctx.with_index(idx);
            futures.push(async move {
                resolve_container(&ctx_idx, ty)
                    .await
                    .map_err(|err| ctx_idx.set_error_path(err))
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
                    let ctx_idx = ctx.with_index(idx);
                    let extensions = &ctx.query_env.extensions;

                    let resolve_info = ResolveInfo {
                        path_node: ctx_idx.path_node.as_ref().unwrap(),
                        parent_type: &Vec::<T>::type_name(),
                        return_type: &T::qualified_type_name(),
                        name: field.node.name.node.as_str(),
                        alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
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
            let ctx_idx = ctx.with_index(idx);
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
