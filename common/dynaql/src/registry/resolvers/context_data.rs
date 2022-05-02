use futures_util::Future;

use super::dynamo_querying::{DynamoResolver, QueryResult};
use super::ResolverTrait;
use crate::registry::is_array_basic_type;
use crate::registry::resolvers::ResolverContext;
use crate::registry::transformers::{Transformer, TransformerTrait};
use crate::registry::variables::VariableResolveDefinition;
use crate::{context::resolver_data_get_opt_ref, Context, Error, Value};
use std::hash::Hash;
use std::pin::Pin;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash)]
pub enum ContextDataResolver {
    /// Key based Resolver for ResolverContext
    Key { key: String },
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
    Edge { key: String, is_node: bool },
}

#[async_trait::async_trait]
impl ResolverTrait for ContextDataResolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
    ) -> Result<serde_json::Value, Error> {
        let current_ty = resolver_ctx.ty.unwrap().name();

        match self {
            ContextDataResolver::Key { key } => {
                let store = ctx
                    .resolvers_data
                    .read()
                    .map_err(|_| Error::new("Internal error"))?;
                let ctx_value = resolver_data_get_opt_ref::<Value>(&store, key)
                    .map(std::clone::Clone::clone)
                    .unwrap_or(Value::Null);
                Ok(serde_json::to_value(ctx_value)?)
            }
            ContextDataResolver::Edge { key, is_node } => {
                let is_array = is_array_basic_type(&resolver_ctx.field.unwrap().ty);

                let mut ctx_value = Vec::new();
                {
                    let store = ctx
                        .resolvers_data
                        .read()
                        .map_err(|_| Error::new("Internal error"))?;
                    if let Some(value) = resolver_data_get_opt_ref::<QueryResult>(&store, key)
                        .and_then(|x| x.get(current_ty))
                    {
                        ctx_value = value.clone();
                    }
                }

                if !is_array && ctx_value.len() > 1 {
                    ctx.add_error(Error::new("An issue occured while resolving this field. Reason: Incoherent schema.").into_server_error(ctx.item.pos));
                }

                if !is_array {
                    let result = ctx_value
                        .first()
                        .map(serde_json::to_value)
                        .transpose()
                        .map(|x| x.unwrap_or(serde_json::Value::Null))
                        .map_err(|_| Error::new("Internal error while manipulating data"))?;

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
                                return Ok(serde_json::Value::Null);
                            }
                        };
                        let sk = VariableResolveDefinition::DebugString(sk);

                        let result = DynamoResolver::QueryPKSK { pk: sk.clone(), sk }
                            .resolve(ctx, resolver_ctx)
                            .await?;

                        return Ok(result);
                    }

                    Ok(result)
                } else {
                    let array = ctx_value
                        .into_iter()
                        .map(serde_json::to_value)
                        .map(|x| x.ok().unwrap_or(serde_json::Value::Null))
                        .collect::<Vec<_>>();

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
                            .transform(result)?;
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
                                    .resolve(ctx, resolver_ctx)
                                    .await
                            });

                            array_result.push(result);
                        }
                        let arr = futures_util::future::try_join_all(array_result).await?;

                        return Ok(serde_json::Value::Array(arr));
                    }

                    Ok(serde_json::Value::Array(array))
                }
            }
        }
    }
}
