use dynomite::AttributeValue;
use indexmap::map::Entry;
use indexmap::IndexMap;
use quick_error::quick_error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::model::id::ID;
use crate::{DynamoDBRequestedIndex, LocalContext, PaginatedCursor};

use super::bridge_api;
use super::types::Sql;

// TODO: Should ensure Rosoto Errors impl clone
quick_error! {
    #[derive(Debug, Clone)]
    pub enum QueryTypePaginatedLoaderError {
        QueryError {
            display("An internal error happened while fetching a list of entities")
        }
    }
}
#[derive(Debug, Clone)]
pub struct QueryValue {
    pub node: Option<HashMap<String, AttributeValue>>,
    pub edges: IndexMap<String, Vec<HashMap<String, AttributeValue>>>,
    /// Constraints are other kind of row we can store, it'll add data over a node
    pub constraints: Vec<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Returned values by PK
    pub values: IndexMap<String, QueryValue>,
    pub last_evaluated_key: Option<String>,
}

pub struct QueryTypePaginatedLoader {
    local_context: Arc<LocalContext>,
    #[allow(dead_code)]
    index: DynamoDBRequestedIndex,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct QueryTypePaginatedKey {
    pub r#type: String,
    pub edges: Vec<String>,
    pub cursor: PaginatedCursor,
}

impl QueryTypePaginatedKey {
    pub fn new(r#type: String, mut edges: Vec<String>, cursor: PaginatedCursor) -> Self {
        Self {
            r#type: r#type.to_lowercase(),
            edges: {
                edges.sort();
                edges
            },
            cursor,
        }
    }
    fn ty(&self) -> &String {
        &self.r#type
    }
}

pub enum QueryTypePaginatedInfo {
    Forward {
        /// The last cursor of the nodes if it exist.
        last_key: Option<String>,
        has_next_page: bool,
    },
    Backward {
        /// The first cursor of the nodes if it exist.
        exclusive_last_key: Option<String>,
        has_previous_page: bool,
    },
}

// TODO: remove or change this documentation to fit sqlite (suggested by @Miaxos)
/// The Result of the Paginated query.
///
/// # Modelization
///
/// When we query the GSI1 we do have the entities stored together, it means that if we
/// ask Node of a type A we would have this kind of answer:
///
/// ```ignore
/// ┌────────┐
/// │Node A  │
/// ├────────┤
/// │Edge A.1│
/// ├────────┤
/// │Edge A.2│
/// ├────────┤
/// │Edge A.3│
/// ├────────┤
/// │Node B  │
/// ├────────┤
/// │Edge B.1│
/// ├────────┤
/// │Node C  │
/// ├────────┤
/// │Node D  │
/// └────────┘
/// ```
///
/// **Note about cursors: a cursor is not a start key, it's an indice that could
/// be removed later but still be used.**
///
/// **Sorted by**: Creation date.
/// The sort must be: **Stable**
///
/// **Even if we disconnect/connect edges later, it would still be grouped that way**.
pub struct QueryTypePaginatedValue {
    /// We define the Value of this QueryLoader like this:
    ///
    /// ```json
    /// {
    ///   "Blog#PK": {
    ///     "Blog": HashMap<String, AttributeValue>,
    ///     "published": Vec<HashMap<String, AttributeValue>>,
    ///     "relation_name": Vec<HashMap<String, AttributeValue>>,
    ///   }
    /// }
    /// ```
    pub fetched_values: HashMap<String, HashMap<String, Vec<HashMap<String, AttributeValue>>>>,
    pub pagination_info: QueryTypePaginatedInfo,
}

#[async_trait::async_trait]
impl Loader<QueryTypePaginatedKey> for QueryTypePaginatedLoader {
    /// We define the Value of this QueryLoader like this:
    ///
    /// ```json
    /// {
    ///   "Blog#PK": {
    ///     "Blog": HashMap<String, AttributeValue>,
    ///     "published": Vec<HashMap<String, AttributeValue>>,
    ///     "relation_name": Vec<HashMap<String, AttributeValue>>,
    ///   }
    /// }
    /// ```
    type Value = QueryResult;
    type Error = QueryTypePaginatedLoaderError;

    async fn load(
        &self,
        keys: &[QueryTypePaginatedKey],
    ) -> Result<HashMap<QueryTypePaginatedKey, Self::Value>, Self::Error> {
        let mut query_result = HashMap::new();
        let mut concurrent_futures = vec![];
        for query_key in keys {
            let has_edges = !query_key.edges.is_empty();
            let entity_type = query_key.r#type.clone();
            // TODO: consider matching over entire query key (suggested by @jakubadamw)
            let (query, values) = match query_key.cursor.clone() {
                PaginatedCursor::Forward {
                    exclusive_last_key,
                    first,
                } => {
                    let query = if has_edges {
                        Sql::SelectTypePaginatedForwardWithEdges(exclusive_last_key.is_some(), query_key.edges.len())
                    } else {
                        Sql::SelectTypePaginatedForward(exclusive_last_key.is_some())
                    };

                    let values = vec![Some(entity_type), exclusive_last_key, Some(first.to_string())]
                        .iter()
                        .filter_map(|value| value.clone())
                        .chain(query_key.edges.iter().cloned())
                        .collect::<Vec<String>>();

                    (query, values)
                }
                PaginatedCursor::Backward {
                    exclusive_first_key,
                    last,
                } => {
                    let query = if has_edges {
                        Sql::SelectTypePaginatedBackwardWithEdges(exclusive_first_key.is_some(), query_key.edges.len())
                    } else {
                        Sql::SelectTypePaginatedBackward(exclusive_first_key.is_some())
                    };

                    let values = vec![Some(entity_type), exclusive_first_key, Some(last.to_string())]
                        .iter()
                        .filter_map(|value| value.clone())
                        .chain(query_key.edges.iter().cloned())
                        .collect::<Vec<String>>();

                    (query, values)
                }
            };

            let future_get = || async move {
                let query_results = bridge_api::query(&query.to_string(), &values, &self.local_context.bridge_port)
                    .await
                    .map_err(|_| QueryTypePaginatedLoaderError::QueryError)?;

                query_results.iter().try_fold(
                    (
                        query_key.clone(),
                        QueryResult {
                            values: IndexMap::with_capacity(100),
                            last_evaluated_key: None,
                        },
                    ),
                    |(query_key, mut accumulator), current| {
                        let pk = ID::try_from(current.pk.clone()).expect("can't fail");
                        let sk = ID::try_from(current.sk.clone()).expect("can't fail");
                        let relation_names = current.relation_names.clone();

                        match accumulator.values.entry(pk.to_string()) {
                            Entry::Vacant(vacant) => {
                                let mut value = QueryValue {
                                    node: None,
                                    edges: IndexMap::with_capacity(5),
                                    constraints: Vec::new(),
                                };
                                match (pk, sk) {
                                    (ID::NodeID(_), ID::NodeID(sk)) => {
                                        if sk.ty() == *query_key.ty() {
                                            value.node = Some(current.document.clone());
                                        } else if let Some(edge) =
                                            query_key.edges.iter().find(|edge| relation_names.contains(edge))
                                        {
                                            value.edges.insert(edge.clone(), vec![current.document.clone()]);
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
                                (ID::NodeID(_), ID::NodeID(sk)) => {
                                    if sk.ty() == *query_key.ty() {
                                        occupied.get_mut().node = Some(current.document.clone());
                                    } else if let Some(edge) =
                                        query_key.edges.iter().find(|edge| relation_names.contains(edge))
                                    {
                                        occupied
                                            .get_mut()
                                            .edges
                                            .entry(edge.clone())
                                            .or_default()
                                            .push(current.document.clone());
                                    }
                                }
                                (ID::ConstraintID(_), ID::ConstraintID(_)) => {
                                    occupied.get_mut().constraints.push(current.document.clone());
                                }
                                _ => {}
                            },
                        }
                        Ok::<_, QueryTypePaginatedLoaderError>((query_key, accumulator))
                    },
                )
            };

            concurrent_futures.push(future_get());
        }

        let joined_futures = futures_util::future::try_join_all(concurrent_futures)
            .await
            .map_err(|_| QueryTypePaginatedLoaderError::QueryError)?;

        // TODO: joined_futures.into_iter().collect() (suggested by @jakubadamw)
        for (query_key, result) in joined_futures {
            query_result.insert(query_key, result);
        }

        Ok(query_result)
    }
}

pub fn get_loader_paginated_query_type(
    local_context: Arc<LocalContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QueryTypePaginatedLoader, LruCache> {
    DataLoader::with_cache(
        QueryTypePaginatedLoader { local_context, index },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
