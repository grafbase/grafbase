#![allow(deprecated)]

use super::dynamo_querying::DynamoResolver;
use super::{ResolvedPaginationInfo, ResolvedValue, ResolverTrait};
use crate::registry::resolvers::ResolverContext;
use crate::registry::transformers::{Transformer, TransformerTrait};
use crate::registry::variables::VariableResolveDefinition;
use crate::{context::resolver_data_get_opt_ref, Context, Error, Value};
use std::hash::Hash;
use std::sync::Arc;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum ContextDataResolver {
    /// Key based Resolver for ResolverContext
    #[deprecated = "Should not use Context anymore in SDL def"]
    Key { key: String },
    /// Key based Resolver for ResolverContext
    LocalKey { key: String },
    /// ContextDataResolver based on Edges.
    ///
    /// When we fetch a Node, we'll also fetch the Edges of that node if needed.
    /// We need to indicate in the ResolverChain than those fields will be Edges.
    ///
    /// The only side note is when your edge is also a Node:
    ///
    /// ```ignore
    ///     Fetch 1             Fetch 2
    ///  ◄──────────────────◄►──────────────►
    ///  ┌──────┐
    ///  │Node A├─┐
    ///  └──────┘ │ ┌────────┐
    ///           ├─┤ Edge 1 ├─┐
    ///           │ └────────┘ │ ┌──────────┐
    ///           │            └─┤ Edge 1.1 │
    ///           │              └──────────┘
    ///           │
    ///           │ ┌────────┐
    ///           └─┤ Edge 2 │
    ///             └────────┘
    /// ```
    ///
    /// When you got a structure like this, the Fetch 1 will allow you to fetch
    /// the Node and his Edges, but you'll also need the Edges from Edge 1 as
    /// it's also a Node.
    ///
    /// The issue is you can only get the first-depth relation in our Graph
    /// Modelization in one go.
    ///
    /// So when we manipulate an Edge which is also a Node, we need to tell the
    /// resolver it's a Node, so we'll know we need to check at request-time, if
    /// the sub-level edges are requested, and if they are, we'll need to perform
    /// a second query accross our database.
    SingleEdge { key: String, relation_name: String },
    EdgeArray {
        key: String,
        relation_name: String,
        /// Expected type output
        /// Used when you are fetching an Edge which doesn't require you fetch other
        /// Nodes
        expected_ty: String,
    },
    /// This resolver get the PaginationData
    PaginationData,
}

#[async_trait::async_trait]
impl ResolverTrait for ContextDataResolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        match self {
            ContextDataResolver::LocalKey { key } => Ok(ResolvedValue::new(Arc::new(
                // TODO: Think again with internal modelization
                last_resolver_value
                    .and_then(|x| x.data_resolved.get(key))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            ))),
            #[allow(deprecated)]
            ContextDataResolver::Key { key } => {
                let store = ctx
                    .resolvers_data
                    .read()
                    .map_err(|_| Error::new("Internal error"))?;
                let ctx_value = resolver_data_get_opt_ref::<Value>(&store, key)
                    .map(std::clone::Clone::clone)
                    .unwrap_or(Value::Null);

                Ok(ResolvedValue::new(Arc::new(serde_json::to_value(
                    ctx_value,
                )?)))
            }
            ContextDataResolver::PaginationData => {
                let pagination = last_resolver_value
                    .and_then(|x| x.pagination.as_ref())
                    .map(ResolvedPaginationInfo::output);
                Ok(ResolvedValue::new(Arc::new(serde_json::to_value(
                    pagination,
                )?)))
            }
            // TODO: look into loading single edges in the same query. This may be tricky as we can no longer differentiate
            // between the queried item and it's edges as a nested pagination will not have pk == sk
            ContextDataResolver::SingleEdge { key, relation_name } => {
                let old_val = match last_resolver_value.and_then(|x| x.data_resolved.get(key)) {
                    Some(serde_json::Value::Array(arr)) => {
                        // Check than the old_val is an array with only 1 element.
                        if arr.len() > 1 {
                            ctx.add_error(Error::new("An issue occured while resolving this field. Reason: Incoherent schema.").into_server_error(ctx.item.pos));
                        }

                        arr.first()
                            .map(std::clone::Clone::clone)
                            .unwrap_or(serde_json::Value::Null)
                    }
                    Some(val) => val.clone(),
                    _ => {
                        return Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null))
                            .with_early_return())
                    }
                };

                let sk = Transformer::DynamoSelect {
                    property: "__sk".to_string(),
                }
                .transform(old_val)?;

                let sk = match sk {
                    serde_json::Value::String(inner) => inner,
                    _ => {
                        ctx.add_error(Error::new("An issue occured while resolving this field. Reason: Incoherent schema.").into_server_error(ctx.item.pos));
                        return Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null)));
                    }
                };

                let result = DynamoResolver::QuerySingleRelation {
                    parent_pk: sk.clone(),
                    relation_name: relation_name.clone(),
                }
                .resolve(ctx, resolver_ctx, last_resolver_value)
                .await?;

                return Ok(result);
            }
            ContextDataResolver::EdgeArray {
                key,
                relation_name,
                expected_ty,
            } => {
                let old_val = match last_resolver_value.and_then(|x| x.data_resolved.get(key)) {
                    Some(serde_json::Value::Array(arr)) => {
                        // Check than the old_val is an array with only 1 element.
                        if arr.len() > 1 {
                            ctx.add_error(Error::new("An issue occured while resolving this field. Reason: Incoherent schema.").into_server_error(ctx.item.pos));
                        }

                        arr.first()
                            .map(std::clone::Clone::clone)
                            .unwrap_or(serde_json::Value::Null)
                    }
                    Some(val) => val.clone(),
                    _ => {
                        return Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null))
                            .with_early_return())
                    }
                };

                let sk = Transformer::DynamoSelect {
                    property: "__sk".to_string(),
                }
                .transform(old_val)?;

                let sk = match sk {
                    serde_json::Value::String(inner) => inner,
                    _ => {
                        ctx.add_error(Error::new("An issue occured while resolving this field. Reason: Incoherent schema.").into_server_error(ctx.item.pos));
                        return Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null)));
                    }
                };

                let result = DynamoResolver::ListResultByTypePaginated {
                    r#type: VariableResolveDefinition::DebugString(expected_ty.to_string()),
                    first: VariableResolveDefinition::InputTypeName("first".to_string()),
                    after: VariableResolveDefinition::InputTypeName("after".to_string()),
                    before: VariableResolveDefinition::InputTypeName("before".to_string()),
                    last: VariableResolveDefinition::InputTypeName("last".to_string()),
                    nested: Some((relation_name.clone(), sk.clone())),
                }
                .resolve(ctx, resolver_ctx, last_resolver_value)
                .await?;

                return Ok(result);
            }
        }
    }
}
