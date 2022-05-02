use super::ResolverTrait;
use crate::registry::{resolvers::ResolverContext, variables::VariableResolveDefinition};
use crate::{Context, Error, Value};
use dynamodb::{DynamoDBBatchersData, QueryKey};
use dynomite::AttributeValue;
use std::collections::HashMap;
use std::hash::Hash;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash)]
pub enum DynamoResolver {
    /// A Query based on the PK and the SK
    QueryPKSK {
        pk: VariableResolveDefinition,
        sk: VariableResolveDefinition,
        /// Define if we need to query the FatIndex to get the associated nodes and edges.
        fat: bool,
    },
}

pub(crate) type QueryResult = HashMap<String, Vec<HashMap<String, AttributeValue>>>;

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

        // TODO: Here we ask from the Type definition the associated edges, but what
        // we should ask is the edges associated FROM the SelectedSet.
        let edges = ctx_ty.edges();
        let edges_len = edges.len();

        match self {
            DynamoResolver::QueryPKSK { pk, sk, fat } => {
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

                    return serde_json::to_value(dyna).map_err(|err| Error::new(err.to_string()));
                }

                let query_loader = if *fat { query_loader_fat } else { query_loader };
                let query_result: QueryResult = query_loader
                    .load_one(QueryKey::new(pk, {
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
                        edges
                    }))
                    .await?
                    .ok_or_else(|| {
                        Error::new("Internal Error: Failed to fetch the associated nodes.")
                    })?;

                // We get the actual requested edge.
                let dyna = if *fat {
                    serde_json::to_value(
                        query_result
                            .get(current_ty)
                            .ok_or_else(|| Error::new("Internal Error: Failed to fetch the node"))?
                            .clone(),
                    )
                } else {
                    serde_json::to_value(
                        query_result
                            .get(current_ty)
                            .map(|x| x.first())
                            .flatten()
                            .ok_or_else(|| Error::new("Internal Error: Failed to fetch the node"))?
                            .clone(),
                    )
                };

                if resolver_ctx.resolver_id.is_some() {
                    // TODO: Try a Entity type repartition shared cache instead of a Query result
                    // based Cache. It should reduce the complexity and be more future proof.
                    let key = format!(
                        "{}_resolver_query_edges",
                        resolver_ctx
                            .ty
                            .ok_or_else(|| {
                                Error::new(
                                    "Internal Error: Failed to process the associated nodes.",
                                )
                            })?
                            .name()
                            .to_lowercase()
                    );
                    ctx.resolver_data_insert(key, query_result);
                }

                dyna.map_err(Error::new_with_source)
            }
        }
    }
}
