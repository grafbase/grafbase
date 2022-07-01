#![allow(deprecated)]

use super::dynamo_querying::DynamoResolver;
use super::{ResolvedPaginationInfo, ResolvedValue, ResolverTrait};
use crate::registry::is_array_basic_type;
use crate::registry::resolvers::ResolverContext;
use crate::registry::transformers::{Transformer, TransformerTrait};
use crate::registry::variables::VariableResolveDefinition;
use crate::{context::resolver_data_get_opt_ref, Context, Error, Value};
use futures_util::Future;
use std::hash::Hash;
use std::pin::Pin;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash)]
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
    Edge {
        key: String,
        /// Is the actual edge also a Node
        /// It means that the edge will also have edges to be fetched.
        is_node: bool,
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
            ContextDataResolver::LocalKey { key } => Ok(ResolvedValue::new(
                // TODO: Think again with internal modelization
                last_resolver_value
                    .and_then(|x| x.data_resolved.get(key))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            )),
            #[allow(deprecated)]
            ContextDataResolver::Key { key } => {
                let store = ctx
                    .resolvers_data
                    .read()
                    .map_err(|_| Error::new("Internal error"))?;
                let ctx_value = resolver_data_get_opt_ref::<Value>(&store, key)
                    .map(std::clone::Clone::clone)
                    .unwrap_or(Value::Null);

                Ok(ResolvedValue::new(serde_json::to_value(ctx_value)?))
            }
            ContextDataResolver::PaginationData => {
                let pagination = last_resolver_value
                    .and_then(|x| x.pagination.as_ref())
                    .map(ResolvedPaginationInfo::output);
                Ok(ResolvedValue::new(serde_json::to_value(pagination)?))
            }
            ContextDataResolver::Edge {
                key,
                is_node,
                expected_ty,
            } => {
                // As we are in an Edge, the result from ancestor should be an array.
                let old_val = match last_resolver_value.and_then(|x| x.data_resolved.get(key)) {
                    Some(serde_json::Value::Array(arr)) => arr,
                    _ => return Ok(ResolvedValue::new(serde_json::Value::Null).with_early_return()),
                };

                let is_expecting_array = is_array_basic_type(&resolver_ctx.field.unwrap().ty);

                // Check than the old_val is an array with only 1 element.
                if !is_expecting_array && old_val.len() > 1 {
                    ctx.add_error(Error::new("An issue occured while resolving this field. Reason: Incoherent schema.").into_server_error(ctx.item.pos));
                }

                if !is_expecting_array {
                    let result = old_val
                        .first()
                        .map(std::clone::Clone::clone)
                        .unwrap_or(serde_json::Value::Null);

                    // If we do have a node, we'll need to fetch the Edges linked to this node.
                    // TODO: Optimize fetch by only fetching REQUESTED edges.
                    if *is_node {
                        let sk = Transformer::DynamoSelect {
                            property: "__sk".to_string(),
                        }
                        .transform(result)?;

                        let sk = match sk {
                            serde_json::Value::String(inner) => inner,
                            _ => {
                                ctx.add_error(Error::new("An issue occured while resolving this field. Reason: Incoherent schema.").into_server_error(ctx.item.pos));
                                return Ok(ResolvedValue::new(serde_json::Value::Null));
                            }
                        };
                        let sk = VariableResolveDefinition::DebugString(sk);

                        let result = DynamoResolver::QueryPKSK { pk: sk.clone(), sk }
                            .resolve(ctx, resolver_ctx, last_resolver_value)
                            .await?;

                        return Ok(result);
                    }

                    Ok(ResolvedValue::new(serde_json::json!({
                        expected_ty: result
                    })))
                } else {
                    let array = old_val;

                    // If we do have a node, we'll need to fetch the Edges linked to this node.
                    // TODO: Optimize fetch by only fetching REQUESTED edges.
                    if *is_node {
                        let mut array_result: Vec<
                            Pin<Box<dyn Future<Output = Result<serde_json::Value, Error>> + Send>>,
                        > = Vec::with_capacity(array.len());

                        for result in array {
                            let sk = Transformer::DynamoSelect {
                                property: "__sk".to_string(),
                            }
                            .transform(result.clone())?;
                            let sk = match sk {
                                serde_json::Value::String(inner) => inner,
                                _ => {
                                    ctx.add_error(Error::new("An issue occured while resolving this field. Reason: Incoherent schema.").into_server_error(ctx.item.pos));
                                    array_result.push(Box::pin(futures_util::future::ok(
                                        serde_json::Value::Null,
                                    )));
                                    continue;
                                }
                            };
                            let sk = VariableResolveDefinition::DebugString(sk);
                            let result = Box::pin(async move {
                                DynamoResolver::QueryPKSK { pk: sk.clone(), sk }
                                    .resolve(ctx, resolver_ctx, last_resolver_value)
                                    .await
                                    .map(|x| x.data_resolved)
                            });

                            array_result.push(result);
                        }
                        let arr = futures_util::future::try_join_all(array_result).await?;

                        return Ok(ResolvedValue::new(serde_json::Value::Array(arr)));
                    }

                    // If we do not have a Node, it means we need to return an array like
                    //
                    // ```json
                    // {
                    //   data: [{
                    //     "Blog": <Value>,
                    //     "<Node>": <Value>,
                    //   }]
                    // }
                    // ```
                    Ok(ResolvedValue::new(serde_json::Value::Array(
                        array
                            .iter()
                            .map(|x| serde_json::json!({ expected_ty: x }))
                            .collect(),
                    )))
                }
            }
        }
    }
}
