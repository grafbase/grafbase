use super::bridge_api;
use super::types::{Operation, Sql};
use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::model::id::ID;
use crate::model::node::NodeID;
use crate::paginated::QueryResult;
use crate::paginated::QueryValue;
use crate::{DynamoDBRequestedIndex, LocalContext};
use indexmap::{map::Entry, IndexMap};
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
            let pk = match NodeID::from_borrowed(&query_key.pk) {
                Ok(id) => id,
                Err(_) => {
                    query_result.insert(query_key.clone(), QueryResult::default());
                    continue;
                }
            };

            let query = if has_edges {
                Sql::SelectIdWithEdges(self.index.pk(), number_of_edges).to_string()
            } else {
                Sql::SelectId(self.index.pk()).to_string()
            };

            let entity_type = pk.ty().to_string();

            let values = if has_edges {
                vec![
                    vec![pk.to_string(), entity_type, pk.to_string()],
                    query_key.edges.clone(),
                ]
                .concat()
            } else {
                vec![pk.to_string()]
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

                        match accumulator.values.entry(pk.to_string()) {
                            Entry::Vacant(vacant) => {
                                let mut value = QueryValue {
                                    node: None,
                                    constraints: Vec::new(),
                                    edges: IndexMap::with_capacity(5),
                                };

                                match (pk, sk) {
                                    (ID::NodeID(pk), ID::NodeID(sk)) => {
                                        if sk.eq(&pk) {
                                            value.node = Some(current.document.clone());
                                        } else if !relation_names.is_empty() {
                                            for edge in relation_names {
                                                value.edges.insert(edge, vec![current.document.clone()]);
                                            }
                                        }
                                    }
                                    (ID::ConstraintID(_), ID::ConstraintID(_)) => {
                                        value.constraints.push(current.document.clone());
                                    }
                                    _ => {}
                                }

                                vacant.insert(value);
                            }
                            Entry::Occupied(mut occupied) => match (pk, sk) {
                                (ID::NodeID(pk), ID::NodeID(sk)) => {
                                    if sk.eq(&pk) {
                                        occupied.get_mut().node = Some(current.document.clone());
                                    } else if !relation_names.is_empty() {
                                        for edge in relation_names {
                                            occupied
                                                .get_mut()
                                                .edges
                                                .entry(edge)
                                                .or_default()
                                                .push(current.document.clone());
                                        }
                                    }
                                }
                                (ID::ConstraintID(_), ID::ConstraintID(_)) => {
                                    occupied.get_mut().constraints.push(current.document.clone());
                                }
                                _ => {}
                            },
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

        // TODO: joined_futures.into_iter().collect() (suggested by @jakubadamw)
        for (query_key, result) in joined_futures {
            query_result.insert(query_key, result);
        }

        Ok(query_result)
    }
}

pub fn get_loader_query(
    local_context: Arc<LocalContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QueryLoader, LruCache> {
    DataLoader::with_cache(
        QueryLoader { local_context, index },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
