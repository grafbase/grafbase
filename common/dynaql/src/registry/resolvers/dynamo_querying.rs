use super::ResolverTrait;
use crate::registry::{resolvers::ResolverContext, variables::VariableResolveDefinition};
use crate::{Context, Error, Value};
use dynamodb::{DynamoDBBatchersData, QueryKey, QueryTypeKey};
use dynomite::AttributeValue;
use serde_json::Map;
use std::collections::HashMap;
use std::hash::Hash;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash)]
pub enum DynamoResolver {
    /// A Query based on the PK and the SK
    ///
    /// We query dynamodb with PK and SK, we also query the edges based on the type infered.
    ///
    /// # Returns
    ///
    /// We expect this resolver to return a Value with this type, if for example.
    /// This resolver should ALWAYS be used for Unique Results.
    ///
    /// With a Blog example where Author would be an Edge:
    ///
    /// ```json
    /// {
    ///   "Blog": HashMap<String, AttributeValue>,
    ///   "Author": Vec<HashMap<String, AttributeValue>>,
    ///   "Edge2": Vec<HashMap<String, AttributeValue>>,
    /// }
    /// ```
    ///
    /// Event if the relation is unique, we'll return a Vec, it's the purpose of the EdgeResolver
    /// to determine if the schema is coherent and to fallback an error if it's not.
    ///
    /// The purpose of this resolver is only to resolve the n-1 level in an optimized way and to
    /// pass the results.
    QueryPKSK {
        pk: VariableResolveDefinition,
        sk: VariableResolveDefinition,
    },
    /// A Query based on the type of the entity.
    ///
    /// We query the reverted index by type to get a node and his edges.
    /// This Resolver is non-paginated, it means it's designed to get EVERY NODE AND EDGES.
    ///
    /// With the workers limits, it can fails.
    /// It should be used with items when we know that the items aren't too big.
    /// A sub-query checker will live that'll allow users to avoid too big queries with partial
    /// response.
    ///
    /// # Returns
    ///
    /// We expect this resolver to return a Value with this type, if for example.
    ///
    /// With a Blog example where Author would be an Edge:
    ///
    /// ```json
    /// [{
    ///   "Blog": HashMap<String, AttributeValue>,
    ///   "Author": Vec<HashMap<String, AttributeValue>>,
    ///   "Edge2": Vec<HashMap<String, AttributeValue>>,
    /// }]
    /// ```
    ListResultByType { r#type: VariableResolveDefinition },
}

pub(crate) type QueryResult = HashMap<String, Vec<HashMap<String, AttributeValue>>>;
pub(crate) type QueryTypeResult =
    HashMap<String, HashMap<String, Vec<HashMap<String, AttributeValue>>>>;

#[async_trait::async_trait]
impl ResolverTrait for DynamoResolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, Error> {
        let loader_item = &ctx.data::<DynamoDBBatchersData>()?.loader;
        let query_loader = &ctx.data::<DynamoDBBatchersData>()?.query;
        let query_loader_fat = &ctx.data::<DynamoDBBatchersData>()?.query_fat;

        let ctx_ty = resolver_ctx
            .ty
            .ok_or_else(|| Error::new("Internal Error: Failed process the associated schema."))?;
        let current_ty = ctx_ty.name();
        #[cfg(feature = "tracing_worker")]
        logworker::info!("", "Current TY: {}", &current_ty,);

        // TODO: Here we ask from the Type definition the associated edges, but what
        // we should ask is the edges associated FROM the SelectedSet.
        let edges = ctx_ty.edges();
        let edges_len = edges.len();

        match self {
            DynamoResolver::ListResultByType { r#type } => {
                let pk = match r#type.param(ctx, last_resolver_value).expect("can't fail") {
                    Value::String(inner) => inner,
                    _ => {
                        return Err(Error::new("Internal Error: failed to infer key"));
                    }
                };

                // When we query a Node with the Query Dataloader, we have to indicate
                // which Edges should be getted with it because we are able to retreive
                // a Node with his edges in one network request.
                // We could also request to have only the node edges and not the node
                // data.
                //
                // We add the Node to the edges to also ask for the Node Data.
                let mut edges = edges
                    .iter()
                    .map(|(_, x)| x.0.to_string())
                    .collect::<Vec<_>>();
                edges.push(pk.clone());

                let query_result: QueryTypeResult = query_loader_fat
                    .load_one(QueryTypeKey::new(pk.clone(), edges))
                    .await?
                    .ok_or_else(|| {
                        Error::new("Internal Error: Failed to fetch the associated nodes.")
                    })?;

                let result: Vec<serde_json::Value> = query_result
                    .into_values()
                    .map(|edges| {
                        let len = edges.len();
                        let value: Map<String, serde_json::Value> = edges.into_iter().fold(
                            Map::with_capacity(len),
                            |mut acc, (edge_key, dyna_value)| {
                                let value = if edge_key == pk {
                                    serde_json::to_value(dyna_value.first())
                                } else {
                                    serde_json::to_value(dyna_value)
                                };

                                match value {
                                    Ok(value) => {
                                        acc.insert(edge_key, value);
                                    }
                                    Err(err) => {
                                        acc.insert(edge_key, serde_json::Value::Null);
                                        ctx.add_error(
                                            Error::new_with_source(err)
                                                .into_server_error(ctx.item.pos),
                                        );
                                    }
                                }
                                acc
                            },
                        );

                        serde_json::Value::Object(value)
                    })
                    .collect();

                Ok(serde_json::Value::Array(result))
            }
            DynamoResolver::QueryPKSK { pk, sk } => {
                let pk = match pk.param(ctx, last_resolver_value).expect("can't fail") {
                    Value::String(inner) => inner,
                    _ => {
                        return Err(Error::new("Internal Error: failed to infer key"));
                    }
                };

                let sk = match sk.param(ctx, last_resolver_value).expect("can't fail") {
                    Value::String(inner) => inner,
                    _ => {
                        return Err(Error::new("Internal Error: failed to infer key"));
                    }
                };

                if edges_len == 0 {
                    let dyna = loader_item
                        .load_one((pk.clone(), sk))
                        .await?
                        .ok_or_else(|| Error::new("Internal Error: Failed to fetch the node"))?;

                    let value =
                        serde_json::to_value(dyna).map_err(|err| Error::new(err.to_string()))?;
                    return Ok(serde_json::json!({
                        current_ty: value,
                    }));
                }

                // When we query a Node with the Query Dataloader, we have to indicate
                // which Edges should be getted with it because we are able to retreive
                // a Node with his edges in one network request.
                // We could also request to have only the node edges and not the node
                // data.
                //
                // We add the Node to the edges to also ask for the Node Data.
                let mut edges = edges
                    .iter()
                    .map(|(_, x)| x.0.to_string())
                    .collect::<Vec<_>>();
                edges.push(current_ty.to_string());

                let query_result: QueryResult = query_loader
                    .load_one(QueryKey::new(pk, edges))
                    .await?
                    .ok_or_else(|| {
                        Error::new("Internal Error: Failed to fetch the associated nodes.")
                    })?;

                let len = query_result.len();
                let result: Map<String, serde_json::Value> =
                    query_result
                        .into_iter()
                        .fold(Map::with_capacity(len), |mut acc, (a, b)| {
                            let value = if a == current_ty {
                                serde_json::to_value(b.first())
                            } else {
                                serde_json::to_value(b)
                            };

                            match value {
                                Ok(value) => {
                                    acc.insert(a, value);
                                }
                                Err(err) => {
                                    acc.insert(a, serde_json::Value::Null);
                                    ctx.add_error(
                                        Error::new_with_source(err).into_server_error(ctx.item.pos),
                                    );
                                }
                            }
                            acc
                        });

                Ok(serde_json::Value::Object(result))
            }
        }
    }
}
