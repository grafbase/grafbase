use super::bridge_api;
use super::types::Sql;
use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::{DynamoDBRequestedIndex, LocalContext};
use dynomite::AttributeValue;
use indexmap::map::Entry;
use indexmap::IndexMap;
use quick_error::quick_error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// TODO: Should ensure Rosoto Errors impl clone
quick_error! {
    #[derive(Debug, Clone)]
    pub enum QueryTypeLoaderError {
        UnknownError {
            display("An internal error happened")
        }
        QueryError {
            display("An internal error happened while fetching a list of entities")
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryValue {
    pub node: Option<HashMap<String, AttributeValue>>,
    pub edges: IndexMap<String, Vec<HashMap<String, AttributeValue>>>,
}

pub struct QueryTypeLoader {
    local_context: Arc<LocalContext>,
    #[allow(dead_code)]
    index: DynamoDBRequestedIndex,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Returned values by PK
    pub values: IndexMap<String, QueryValue>,
    pub last_evaluated_key: Option<String>,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct QueryTypeKey {
    r#type: String,
    edges: Vec<String>,
}

impl QueryTypeKey {
    pub fn new(r#type: String, mut edges: Vec<String>) -> Self {
        Self {
            r#type,
            edges: {
                edges.sort();
                edges
            },
        }
    }
}

#[async_trait::async_trait]
impl Loader<QueryTypeKey> for QueryTypeLoader {
    /// We define the Value of this QueryLoader like this:
    ///
    /// ```json
    /// {
    ///   "Blog#PK": {
    ///     "Blog": Vec<HashMap<String, AttributeValue>>,
    ///     "published": Vec<HashMap<String, AttributeValue>>,
    ///     "relation_name": Vec<HashMap<String, AttributeValue>>,
    ///   }
    /// }
    /// ```
    type Value = QueryResult;
    type Error = QueryTypeLoaderError;

    async fn load(&self, keys: &[QueryTypeKey]) -> Result<HashMap<QueryTypeKey, Self::Value>, Self::Error> {
        let mut query_result = HashMap::new();
        let mut concurrent_futures = vec![];
        for query_key in keys {
            let has_edges = !query_key.edges.is_empty();
            let number_of_edges = query_key.edges.len();

            let query = if has_edges {
                Sql::SelectTypeWithEdges(number_of_edges).to_string()
            } else {
                Sql::SelectType.to_string()
            };

            let entity_type = query_key.r#type.clone();

            let values = if has_edges {
                vec![vec![entity_type], query_key.edges.clone()].concat()
            } else {
                vec![entity_type]
            };

            let future = || async move {
                let query_results = bridge_api::query(&query, &values, &self.local_context.bridge_port)
                    .await
                    .map_err(|_| QueryTypeLoaderError::QueryError)?;

                query_results.iter().try_fold(
                    (
                        query_key.clone(),
                        QueryResult {
                            values: IndexMap::with_capacity(100),
                            last_evaluated_key: None,
                        },
                    ),
                    |(query_key, mut accumulator), current| {
                        let pk = current.pk.clone();
                        let sk = current.sk.clone();
                        let relation_names = current.relation_names.clone();

                        match accumulator.values.entry(pk.clone()) {
                            Entry::Vacant(vacant) => {
                                let mut value = QueryValue {
                                    node: None,
                                    edges: IndexMap::with_capacity(5),
                                };

                                // If it's the entity
                                if sk == pk {
                                    value.node = Some(current.document.clone());
                                // If it's a relation
                                } else if !relation_names.is_empty() {
                                    for edge in relation_names {
                                        value.edges.insert(edge, vec![current.document.clone()]);
                                    }
                                }

                                vacant.insert(value);
                            }
                            Entry::Occupied(mut occupied) => {
                                if sk == pk {
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
                        };
                        Ok::<_, QueryTypeLoaderError>((query_key, accumulator))
                    },
                )
            };
            concurrent_futures.push(future());
        }

        let joined_futures = futures_util::future::try_join_all(concurrent_futures)
            .await
            .map_err(|_| QueryTypeLoaderError::QueryError)?;

        for (query_key, result) in joined_futures {
            query_result.insert(query_key, result);
        }

        Ok(query_result)
    }
}

pub fn get_loader_query_type(
    local_context: Arc<LocalContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QueryTypeLoader, LruCache> {
    DataLoader::with_cache(
        QueryTypeLoader { local_context, index },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
