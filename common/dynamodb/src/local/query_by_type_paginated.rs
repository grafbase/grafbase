use dynomite::{attr_map, AttributeValue};
use grafbase::auth::Operations;
use graph_entities::ID;
use indexmap::IndexMap;
use maplit::hashmap;
use quick_error::quick_error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::paginated::ParentEdge;
use crate::runtime::Runtime;
use crate::{DynamoDBContext, DynamoDBRequestedIndex, LocalContext, PaginatedCursor, PaginationOrdering};

use super::bridge_api;
use super::types::{Operation, Sql, SqlValue};

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

impl Default for QueryValue {
    fn default() -> Self {
        QueryValue {
            node: None,
            constraints: Vec::new(),
            edges: IndexMap::with_capacity(5),
        }
    }
}

pub struct QueryValueIter<'a> {
    pub node: Option<&'a HashMap<String, AttributeValue>>,
    pub edges: Box<dyn Iterator<Item = &'a HashMap<String, AttributeValue>> + 'a + Send + Sync>,
}

impl<'a> QueryValue
where
    Self: 'a,
{
    pub fn iter(&'a self) -> QueryValueIter<'a> {
        let node = self.node.as_ref();
        let edges = Box::new(self.edges.iter().flat_map(|(_, y)| y.iter()));
        QueryValueIter { node, edges }
    }
}

impl<'a> Iterator for QueryValueIter<'a> {
    type Item = &'a HashMap<String, AttributeValue>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.node {
            self.node = None;
            return Some(node);
        }

        self.edges.next()
    }
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Returned values by PK
    pub values: IndexMap<String, QueryValue>,
    pub last_evaluated_key: Option<String>,
}

pub struct QueryTypePaginatedLoader {
    local_context: Arc<LocalContext>,
    ctx: Arc<DynamoDBContext>,
    #[allow(dead_code)]
    index: DynamoDBRequestedIndex,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct QueryTypePaginatedKey {
    pub r#type: String,
    pub edges: Vec<String>,
    pub cursor: PaginatedCursor,
    pub ordering: PaginationOrdering,
}

impl QueryTypePaginatedKey {
    pub fn new(r#type: String, mut edges: Vec<String>, cursor: PaginatedCursor, ordering: PaginationOrdering) -> Self {
        Self {
            r#type: r#type.to_lowercase(),
            edges: {
                edges.sort();
                edges
            },
            cursor,
            ordering,
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

/// The Result of the Paginated query.
///
/// # Modelization
///
/// When we query we have the entities stored together, that means that if we
/// ask Node of type A we would get this kind of answer:
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
        let mut concurrent_futures = vec![];
        for query_key in keys {
            let entity_type = query_key.r#type.clone();

            let user_limit = match query_key.cursor {
                PaginatedCursor::Forward { first, .. } => first,
                PaginatedCursor::Backward { last, .. } => last,
            };

            // as we currently limit the result count to 100, will not overflow
            let query_limit = user_limit + 1;

            let mut value_map = hashmap! {
                "entity_type" => SqlValue::String(entity_type),
                "user_limit" => SqlValue::String(user_limit.to_string()),
                "query_limit" => SqlValue::String(query_limit.to_string()),
                "edges" => SqlValue::VecDeque(query_key.edges.clone().into()),
            };

            if let Some(origin) = query_key.cursor.maybe_origin() {
                value_map.insert("sk", SqlValue::String(origin.to_string()));
            }

            if let Some(ParentEdge {
                relation_name,
                parent_id,
            }) = query_key.cursor.maybe_parent_edge()
            {
                value_map.insert("pk", SqlValue::String(parent_id.to_string()));
                value_map.insert("relation_name", SqlValue::String(relation_name.to_string()));
            }
            let filter_by_owner = if let Some(user_id) = self.ctx.restrict_by_owner(Operations::LIST) {
                value_map.insert(crate::local::types::OWNED_BY_KEY, SqlValue::String(user_id.to_string()));
                true
            } else {
                false
            };

            let (query, values) = Sql::SelectTypePaginated {
                has_origin: query_key.cursor.maybe_origin().is_some(),
                is_nested: query_key.cursor.maybe_parent_edge().is_some(),
                // compact version: query_key.cursor.is_backward() ^ query_key.ordering.is_asc()
                ascending: if query_key.cursor.is_forward() {
                    query_key.ordering.is_asc()
                } else {
                    // As we're going backwards, we need to reverse the database scan and reverse
                    // the results at the end to return the expected ordering.
                    //                         after
                    //                           ┌───────► first (forward)
                    //                           │
                    //              ─────────────┼───────────────► Record order
                    //                           │
                    // last (backward) ◄─────────┘
                    //                         before
                    !query_key.ordering.is_asc()
                },
                edges_count: query_key.edges.len(),
                filter_by_owner,
            }
            .compile(value_map);

            let future_get = || async move {
                let query_results = bridge_api::query(
                    Operation {
                        sql: query.to_string(),
                        values,
                        kind: None,
                    },
                    &self.local_context.bridge_port,
                )
                .await
                .map_err(|_| QueryTypePaginatedLoaderError::QueryError)?;

                let more_available = user_limit > 0 && query_results.len() > user_limit;
                // FIXME: last_evaluated_key is part of the API exposed by the DynamoDB functions,
                // but it's an interface problem, pagination only needs to know whether there's
                // more data available or not AFAIK.
                let (last_evaluated_key, mut page_records) = if more_available {
                    // we currently use only pk/sk for pagination
                    // the last evaluated key is always the last item, regardless of direction

                    let last_item = query_results.get(user_limit - 1).expect("must exist");
                    let last_evaluated_key = serde_json::to_string(&attr_map! {
                        "pk" => last_item.pk.clone(),
                        "sk" => last_item.sk.clone(),
                    })
                    .expect("must parse");

                    // Skipping last item which corresponds to the last item retrieved in cursor
                    // direction.
                    let page_records = query_results[..(query_results.len() - 1)].iter();
                    (Some(last_evaluated_key), page_records)
                } else {
                    (None, query_results.iter())
                };

                let result = page_records.try_fold(
                    (
                        query_key.clone(),
                        QueryResult {
                            values: IndexMap::with_capacity(100),
                            last_evaluated_key,
                        },
                    ),
                    |(query_key, mut accumulator), current| {
                        let pk = ID::try_from(current.pk.clone()).expect("can't fail");
                        let sk = ID::try_from(current.sk.clone()).expect("can't fail");
                        let relation_names = current.relation_names.clone();

                        let is_top_level_nested = query_key
                            .cursor
                            .nested_parent_pk()
                            .filter(|query_pk| query_pk == &pk.to_string())
                            .is_some();

                        let key = if is_top_level_nested {
                            sk.to_string()
                        } else {
                            pk.to_string()
                        };

                        let value = accumulator.values.entry(key).or_default();

                        match (pk, sk) {
                            (ID::NodeID(_), ID::NodeID(sk)) => {
                                if sk.ty() == *query_key.ty() {
                                    value.node = Some(current.document.clone());
                                } else if let Some(edge) =
                                    query_key.edges.iter().find(|edge| relation_names.contains(edge))
                                {
                                    value
                                        .edges
                                        .entry(edge.clone())
                                        .or_default()
                                        .push(current.document.clone());
                                }
                            }
                            (ID::ConstraintID(_), ID::ConstraintID(_)) => {
                                value.constraints.push(current.document.clone());
                            }
                            _ => {}
                        }

                        Ok::<_, QueryTypePaginatedLoaderError>((query_key, accumulator))
                    },
                );

                result.map(|(key, mut query_result)| {
                    // Ordering of the items is independent of cursor direction. So if cursor dicrection
                    // doesn't matches the record one, we must reverse the results.
                    //                         after
                    //                           ┌───────► first (forward)
                    //                           │
                    //              ─────────────┼───────────────► Record order
                    //                           │
                    // last (backward) ◄─────────┘
                    //                         before
                    if query_key.cursor.is_backward() {
                        query_result.values.reverse();
                    }
                    (key, query_result)
                })
            };

            concurrent_futures.push(future_get());
        }

        let joined_futures = futures_util::future::try_join_all(concurrent_futures)
            .await
            .map_err(|_| QueryTypePaginatedLoaderError::QueryError)?;

        Ok(joined_futures.into_iter().collect())
    }
}

pub fn get_loader_paginated_query_type(
    local_context: Arc<LocalContext>,
    ctx: Arc<DynamoDBContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QueryTypePaginatedLoader, LruCache> {
    DataLoader::with_cache(
        QueryTypePaginatedLoader {
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
