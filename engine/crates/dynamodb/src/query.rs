use std::{collections::HashMap, sync::Arc, time::Duration};

use dataloader::{DataLoader, Loader, LruCache};
use dynomite::{Attribute, DynamoDbExt};
use futures_util::TryStreamExt;
use graph_entities::{NodeID, ID};
use indexmap::IndexMap;
use itertools::Itertools;
use rusoto_dynamodb::QueryInput;
#[cfg(feature = "tracing")]
use tracing::{info_span, Instrument};

use crate::{
    constant::{OWNED_BY, PK, RELATION_NAMES, SK, TYPE},
    paginated::QueryResult,
    DynamoDBContext, DynamoDBRequestedIndex, OperationAuthorization, OperationAuthorizationError,
};
#[cfg(not(feature = "wasm"))]
#[derive(Debug, Clone, thiserror::Error)]
pub enum QueryLoaderError {
    #[error("An internal error happened")]
    UnknownError,
    #[error("An internal error happened while fetching a list of entities")]
    QueryError,
    #[error("{0}")]
    Unauthorized(
        #[from]
        #[source]
        OperationAuthorizationError,
    ),
}

pub struct QueryLoader {
    ctx: Arc<DynamoDBContext>,
    index: DynamoDBRequestedIndex,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct QueryKey {
    pk: String,
    edges: Vec<String>,
}

impl QueryKey {
    pub fn new(pk: String, mut edges: Vec<String>) -> Self {
        Self {
            pk,
            edges: {
                edges.sort();
                edges
            },
        }
    }
}

#[async_trait::async_trait]
impl Loader<QueryKey> for QueryLoader {
    type Value = QueryResult;
    type Error = QueryLoaderError;

    async fn load(&self, keys: &[QueryKey]) -> Result<HashMap<QueryKey, Self::Value>, Self::Error> {
        log::debug!(self.ctx.trace_id, "Query Dataloader invoked {:?}", keys);
        let mut h = HashMap::new();
        let mut concurrent_f = vec![];
        let owned_by = match self.ctx.authorize_operation(crate::RequestedOperation::Get)? {
            OperationAuthorization::OwnerBased(owned_by) => Some(owned_by),
            _ => None,
        };
        for query_key in keys {
            // TODO: Handle this when dealing with Custom ID
            let Ok(pk) = NodeID::from_borrowed(&query_key.pk) else {
                h.insert(query_key.clone(), QueryResult::default());
                continue;
            };
            let mut exp = dynomite::attr_map! {
                ":pk" => pk.to_string(),
            };
            let edges_len = query_key.edges.len();

            let mut exp_attr = HashMap::with_capacity(3);
            exp_attr.insert("#pk".to_string(), self.index.pk());

            if edges_len > 0 {
                exp_attr.insert("#relationname".to_string(), RELATION_NAMES.to_string());
                exp_attr.insert("#type".to_string(), TYPE.to_string());
            }

            let mut filter_expression = if edges_len > 0 {
                let edges = query_key
                    .edges
                    .iter()
                    .enumerate()
                    .map(|(index, q)| {
                        exp.insert(format!(":relation{index}"), q.clone().into_attr());
                        format!(" contains(#relationname, :relation{index})")
                    })
                    .join(" OR ");

                let ty_attr = pk.ty().into_attr();

                exp.insert(":type".to_string(), ty_attr);
                vec![format!("(begins_with(#type, :type) OR {edges})")]
            } else {
                vec![]
            };

            if let Some(owned_by) = owned_by {
                exp_attr.insert("#owned_by_name".to_string(), OWNED_BY.to_string());
                exp.insert(":owned_by_value".to_string(), owned_by.to_string().into_attr());
                filter_expression.push("contains(#owned_by_name, :owned_by_value)".to_string());
            }

            let filter_expression = if filter_expression.is_empty() {
                None
            } else {
                Some(filter_expression.join(" AND "))
            };
            let input: QueryInput = QueryInput {
                table_name: self.ctx.dynamodb_table_name.clone(),
                key_condition_expression: Some("#pk = :pk".to_string()),
                filter_expression,
                index_name: self.index.to_index_name(),
                expression_attribute_values: Some(exp),
                expression_attribute_names: Some(exp_attr),

                ..Default::default()
            };
            let future_get = || async move {
                let req = self
                    .ctx
                    .dynamodb_client
                    .clone()
                    .query_pages(input)
                    .inspect_err(|err| {
                        log::error!(self.ctx.trace_id, "QueryError {:?}", err);
                    })
                    .try_fold(
                        (
                            query_key.clone(),
                            QueryResult {
                                values: IndexMap::with_capacity(100),
                                last_evaluated_key: None,
                            },
                        ),
                        |(query_key, mut acc), curr| async move {
                            let pk = ID::try_from(curr.get(PK).and_then(|x| x.s.as_ref()).expect("can't fail").clone())
                                .expect("Can't fail");
                            let sk = ID::try_from(curr.get(SK).and_then(|x| x.s.as_ref()).expect("can't fail").clone())
                                .expect("Can't fail");
                            let relation_names = curr.get(RELATION_NAMES).and_then(|y| y.ss.clone());

                            let value = acc.values.entry(pk.to_string()).or_default();

                            match (pk, sk) {
                                (ID::NodeID(pk), ID::NodeID(sk)) => {
                                    if sk.eq(&pk) {
                                        value.node = Some(curr.clone());
                                    } else if let Some(edges) = relation_names {
                                        for edge in edges {
                                            value.edges.entry(edge).or_default().push(curr.clone());
                                        }
                                    }
                                }
                                (ID::ConstraintID(constraint_id), ID::ConstraintID(_)) => {
                                    value.constraints.push((constraint_id, curr));
                                }
                                _ => {}
                            }

                            Ok((query_key, acc))
                        },
                    );
                #[cfg(feature = "tracing")]
                let req = req.instrument(info_span!("fetch query"));
                req.await
            };
            concurrent_f.push(future_get());
        }

        let b = futures_util::future::try_join_all(concurrent_f);
        #[cfg(feature = "tracing")]
        let b = b.instrument(info_span!("fetch query concurrent"));
        let b = b.await.map_err(|err| {
            log::error!(self.ctx.trace_id, "Error while querying: {:?}", err);
            QueryLoaderError::QueryError
        })?;

        for (q, r) in b {
            h.insert(q, r);
        }

        log::debug!(self.ctx.trace_id, "Query Dataloader executed {:?}", keys);
        Ok(h)
    }
}

pub fn get_loader_query(ctx: Arc<DynamoDBContext>, index: DynamoDBRequestedIndex) -> DataLoader<QueryLoader, LruCache> {
    DataLoader::with_cache(QueryLoader { ctx, index }, async_runtime::spawn, LruCache::new(256))
        .max_batch_size(10)
        .delay(Duration::from_millis(2))
}
