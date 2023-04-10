use super::bridge_api;
use super::types::{Operation, Sql, SqlValue};
use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::paginated::QueryResult;
use crate::runtime::Runtime;
use crate::{DynamoDBContext, DynamoDBRequestedIndex, LocalContext};
use grafbase::auth::Operations;
use graph_entities::{NodeID, ID};
use indexmap::map::IndexMap;
use maplit::hashmap;
use quick_error::quick_error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

quick_error! {
    #[derive(Debug, Clone)]
    pub enum QueryLoaderError {
        UnknownError {
            display("An internal error happened")
        }
        QueryError {
            display("An internal error happened while fetching a list of entities")
        }
    }
}

pub struct QueryLoader {
    local_context: Arc<LocalContext>,
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
        let mut query_result = HashMap::new();
        let mut concurrent_futures = vec![];
        for query_key in keys {
            let has_edges = !query_key.edges.is_empty();
            let number_of_edges = query_key.edges.len();
            let Ok(pk) = NodeID::from_borrowed(&query_key.pk) else {
                query_result.insert(query_key.clone(), QueryResult::default());
                continue;
            };

            let mut value_map = hashmap! {
                "pk" => SqlValue::String(pk.to_string()),
                "entity_type" => SqlValue::String(pk.ty().to_string()),
                "edges" => SqlValue::VecDeque(query_key.edges.clone().into()),
            };
            let filter_by_owner = if let Some(user_id) = self.ctx.restrict_by_owner(Operations::LIST) {
                value_map.insert(crate::local::types::OWNED_BY_KEY, SqlValue::String(user_id.to_string()));
                true
            } else {
                false
            };

            let (query, values) = if has_edges {
                Sql::SelectIdWithEdges {
                    pk: self.index.pk(),
                    number_of_edges,
                    filter_by_owner,
                }
                .compile(value_map)
            } else {
                Sql::SelectId {
                    pk: self.index.pk(),
                    filter_by_owner,
                }
                .compile(value_map)
            };

            let future = || async move {
                let query_results = bridge_api::query(
                    Operation {
                        sql: query,
                        values,
                        kind: None,
                    },
                    &self.local_context.bridge_port,
                )
                .await
                .map_err(|_| QueryLoaderError::QueryError)?;

                query_results.iter().try_fold(
                    (
                        query_key.clone(),
                        QueryResult {
                            values: IndexMap::with_capacity(100),
                            last_evaluated_key: None,
                        },
                    ),
                    |(query_key, mut accumulator), current| {
                        let pk = ID::try_from(current.pk.clone()).expect("Can't fail");
                        let sk = ID::try_from(current.sk.clone()).expect("Can't fail");
                        let relation_names = current.relation_names.clone();

                        let value = accumulator.values.entry(pk.to_string()).or_default();
                        match (pk, sk) {
                            (ID::NodeID(pk), ID::NodeID(sk)) => {
                                if sk.eq(&pk) {
                                    value.node = Some(current.document.clone());
                                } else if !relation_names.is_empty() {
                                    for edge in relation_names {
                                        value.edges.entry(edge).or_default().push(current.document.clone());
                                    }
                                }
                            }
                            (ID::ConstraintID(constraint_id), ID::ConstraintID(_)) => {
                                value.constraints.push((constraint_id, current.document.clone()));
                            }
                            _ => {}
                        };
                        Ok::<_, QueryLoaderError>((query_key, accumulator))
                    },
                )
            };
            concurrent_futures.push(future());
        }

        let joined_futures = futures_util::future::try_join_all(concurrent_futures)
            .await
            .map_err(|_| QueryLoaderError::QueryError)?;

        query_result.extend(joined_futures.into_iter().collect::<HashMap<_, _>>());

        Ok(query_result)
    }
}

pub fn get_loader_query(
    local_context: Arc<LocalContext>,
    ctx: Arc<DynamoDBContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QueryLoader, LruCache> {
    DataLoader::with_cache(
        QueryLoader {
            local_context,
            ctx,
            index,
        },
        |f| Runtime::locate().spawn(f),
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
