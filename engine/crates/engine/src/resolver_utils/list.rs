use std::{future::Future, iter::Peekable};

use engine_value::Name;
use graph_entities::{CompactValue, QueryResponseNode, ResponseList, ResponseNodeId, ResponsePrimitive};

use crate::{
    extensions::ResolveInfo,
    parser::types::Field,
    registry::{
        resolvers::ResolvedValue,
        scalars::{DynamicScalar, PossibleScalar},
        MetaFieldType, MetaType, WrappingType, WrappingTypeIter,
    },
    resolver_utils::resolve_container,
    ContextExt, ContextSelectionSet, Error, LegacyOutputType, Positioned, ServerError, ServerResult, Value,
};

/// Resolve a list by executing each of the items concurrently.
pub async fn resolve_list<'a>(
    ctx: ContextSelectionSet<'a>,
    field: &'a Positioned<Field>,
    field_ty: &MetaFieldType,
    inner_ty: &'a MetaType,
    value: ResolvedValue,
) -> ServerResult<ResponseNodeId> {
    #[async_recursion::async_recursion]
    async fn inner(
        ctx: ContextSelectionSet<'async_recursion>,
        field: &Positioned<Field>,
        list_kinds: &[ListKind],
        ty: &MetaType,
        value: ResolvedValue,
    ) -> Result<ResponseNodeId, ServerError> {
        let Some(list_kind) = list_kinds.first() else {
            // If there's no list_kind then we've reached the innermost item and should resolve that
            return resolve_item(ctx, field, ty, value).await;
        };

        // First we need to make sure our parent resolve data actually has a list
        // (or return null early if we're on a nullable list)
        let items = match (list_kind, value.data_resolved()) {
            (ListKind::NullableList(_), serde_json::Value::Null) => {
                return Ok(ctx.response().await.insert_node(CompactValue::Null));
            }
            (ListKind::NonNullList(_), serde_json::Value::Null) => {
                return Err(ctx.set_error_path(ServerError::new(
                    format!(
                        "An error occurred while fetching `{}`, a non-nullable value was expected but no value was found.",
                        field.node.name.node
                    ),
                    Some(ctx.item.pos),
                )));
            }
            (_, serde_json::Value::Array(_)) => value.item_iter().expect("we checked its an array").collect::<Vec<_>>(),
            (_, value) => {
                return Err(ctx.set_error_path(ServerError::new(
                    format!("Encountered a {} where we expected a list", json_kind_str(value)),
                    Some(ctx.item.pos),
                )));
            }
        };

        let futures = items.into_iter().enumerate().map(|(idx, item)| {
            let ctx_idx = ctx.with_index(idx, Some(&ctx.item.node));
            let resolve_future = inner(ctx_idx.clone(), field, &list_kinds[1..], ty, item);

            if ctx.query_env.extensions.is_empty() {
                resolve_future
            } else {
                Box::pin(apply_extensions(ctx_idx, field, ty, resolve_future))
            }
        });

        let mut children = vec![];
        for (index, result) in futures_util::future::join_all(futures).await.into_iter().enumerate() {
            // Now we need to handle error propagation and validate the nullability
            // of each of the list items
            match result {
                Ok(id) if list_kind.has_non_null_item() => {
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
                                Some(ctx.item.pos),
                            );

                        error.path = ctx.path.child(index).into_iter().collect();

                        return Err(error);
                    }
                    children.push(id);
                }
                Ok(id) => children.push(id),
                Err(error) if list_kind.has_nullable_item() => {
                    ctx.add_error(error);
                    children.push(ctx.response().await.insert_node(CompactValue::Null));
                }
                Err(error) => return Err(error),
            }
        }

        Ok(ctx.response().await.insert_node(ResponseList::with_children(children)))
    }

    inner(
        ctx,
        field,
        &ListNullabilityIter::new(field_ty).collect::<Vec<_>>(),
        inner_ty,
        value,
    )
    .await
}

fn apply_extensions<'a>(
    ctx: ContextSelectionSet<'a>,
    field: &'a Positioned<Field>,
    inner_ty: &'a MetaType,
    resolve_fut: impl Future<Output = Result<ResponseNodeId, ServerError>> + Send + 'a,
) -> impl Future<Output = Result<ResponseNodeId, ServerError>> + 'a {
    let ctx = ctx.clone();
    let type_name = inner_ty.name();
    async move {
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
            path: ctx.path.clone(),
            parent_type: &parent_type,
            return_type: &return_type,
            name: field.node.name.node.as_str(),
            alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
            required_operation: meta_field.and_then(|f| f.required_operation),
            auth: meta_field.and_then(|f| f.auth.as_ref()),
            input_values: args_values,
        };

        let resolve_fut = async move { Ok(Some(resolve_fut.await?)) };
        futures_util::pin_mut!(resolve_fut);
        ctx.query_env
            .extensions
            .resolve(resolve_info, &mut resolve_fut)
            .await
            .map(|value| value.expect("You definitely encountered a bug!"))
    }
}

async fn resolve_item(
    ctx_idx: ContextSelectionSet<'_>,
    field: &Positioned<Field>,
    ty: &MetaType,
    item: ResolvedValue,
) -> Result<ResponseNodeId, ServerError> {
    match ty {
        MetaType::Scalar(_) | MetaType::Enum(_) => {
            let mut result = Value::try_from(item.take()).map_err(|err| Error::new(format!("{err:?}")));

            // Yes it's ugly...
            if let MetaType::Scalar(_) = ty {
                result = result.and_then(|value| resolve_scalar(value, ty.name()));
            }

            let item = result.map_err(|error| ctx_idx.set_error_path(error.into_server_error(field.pos)))?;

            Ok(ctx_idx
                .response()
                .await
                .insert_node(ResponsePrimitive::new(item.into())))
        }
        _ => resolve_container(&ctx_idx, ty, None, Some(item))
            .await
            .map_err(|err| ctx_idx.set_error_path(err)),
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
                        path: ctx_idx.path.clone(),
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

        let node_id = ctx.response().await.insert_node(ResponseList::with_children(children));

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

        let node_id = ctx.response().await.insert_node(ResponseList::with_children(children));

        Ok(node_id)
    }
}

/// An iterator over the nullability of lists in a type string
struct ListNullabilityIter<'a>(Peekable<WrappingTypeIter<'a>>);

impl<'a> ListNullabilityIter<'a> {
    pub fn new(ty: &'a MetaFieldType) -> Self {
        ListNullabilityIter(ty.wrapping_types().peekable())
    }
}

/// The nullability of a list _and_ its contents
#[derive(Debug, PartialEq, Clone, Copy)]
enum ListKind {
    NullableList(ListInner),
    NonNullList(ListInner),
}

impl ListKind {
    pub fn has_nullable_item(self) -> bool {
        matches!(self.inner_nullablity(), ListInner::Nullable)
    }

    pub fn has_non_null_item(self) -> bool {
        matches!(self.inner_nullablity(), ListInner::NonNullable)
    }

    fn inner_nullablity(self) -> ListInner {
        match self {
            ListKind::NullableList(inner) => inner,
            ListKind::NonNullList(inner) => inner,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ListInner {
    Nullable,
    NonNullable,
}

impl Iterator for ListNullabilityIter<'_> {
    type Item = ListKind;

    fn next(&mut self) -> Option<Self::Item> {
        let mut nullable = true;
        loop {
            match self.0.next()? {
                WrappingType::NonNull => {
                    nullable = false;
                    continue;
                }
                WrappingType::List if nullable => {
                    return Some(ListKind::NullableList(match self.0.peek() {
                        Some(WrappingType::NonNull) => ListInner::NonNullable,
                        _ => ListInner::Nullable,
                    }))
                }
                WrappingType::List => {
                    return Some(ListKind::NonNullList(match self.0.peek() {
                        Some(WrappingType::NonNull) => ListInner::NonNullable,
                        _ => ListInner::Nullable,
                    }))
                }
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn list_nullability(ty: &str) -> Vec<ListKind> {
        ListNullabilityIter::new(&ty.into()).collect::<Vec<_>>()
    }

    #[test]
    fn test_list_nullability_iter() {
        assert_eq!(list_nullability("String"), vec![]);
        assert_eq!(list_nullability("String!"), vec![]);
        assert_eq!(
            list_nullability("[String!]"),
            vec![ListKind::NullableList(ListInner::NonNullable)]
        );
        assert_eq!(
            list_nullability("[String!]!"),
            vec![ListKind::NonNullList(ListInner::NonNullable)]
        );
        assert_eq!(
            list_nullability("[String]!"),
            vec![ListKind::NonNullList(ListInner::Nullable)]
        );
        assert_eq!(
            list_nullability("[[String!]!]"),
            vec![
                ListKind::NullableList(ListInner::NonNullable),
                ListKind::NonNullList(ListInner::NonNullable)
            ]
        );
        assert_eq!(
            list_nullability("[[String!]]!"),
            vec![
                ListKind::NonNullList(ListInner::Nullable),
                ListKind::NullableList(ListInner::NonNullable)
            ]
        );
        assert_eq!(
            list_nullability("[[[String]]!]"),
            vec![
                ListKind::NullableList(ListInner::NonNullable),
                ListKind::NonNullList(ListInner::Nullable),
                ListKind::NullableList(ListInner::Nullable)
            ]
        );
    }
}
