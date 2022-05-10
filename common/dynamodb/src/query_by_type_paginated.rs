use dynomite::AttributeValue;
use quick_error::quick_error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::paginated::{DynamoDbExtPaginated, PaginatedCursor, QueryResult};
use crate::{DynamoDBContext, DynamoDBRequestedIndex};

// TODO: Should ensure Rosoto Errors impl clone
quick_error! {
    #[derive(Debug, Clone)]
    pub enum QueryTypePaginatedLoaderError {
        UnknowError {
            display("An internal error happened")
        }
        QueryError {
            display("An internal error happened while fetching a list of entities")
        }
    }
}

pub struct QueryTypePaginatedLoader {
    ctx: Arc<DynamoDBContext>,
    index: DynamoDBRequestedIndex,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct QueryTypePaginatedKey {
    pub r#type: String,
    pub edges: Vec<String>,
    pub cursor: PaginatedCursor,
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

impl QueryTypePaginatedKey {
    pub fn new(r#type: String, mut edges: Vec<String>, cursor: PaginatedCursor) -> Self {
        Self {
            r#type,
            edges: {
                edges.sort();
                edges
            },
            cursor,
        }
    }
}

/// The Result of the Paginated query.
///
/// # Modelization
///
/// When we query the GSI1 we do have the entities stored together, it means that if we
/// ask Node of a type A we would have this kind of answer:
///
/// ```
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
    ///     "Author": Vec<HashMap<String, AttributeValue>>,
    ///     "Edge": Vec<HashMap<String, AttributeValue>>,
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
    ///     "Author": Vec<HashMap<String, AttributeValue>>,
    ///     "Edge": Vec<HashMap<String, AttributeValue>>,
    ///   }
    /// }
    /// ```
    type Value = QueryResult;
    type Error = QueryTypePaginatedLoaderError;

    async fn load(
        &self,
        keys: &[QueryTypePaginatedKey],
    ) -> Result<HashMap<QueryTypePaginatedKey, Self::Value>, Self::Error> {
        log::info!(self.ctx.trace_id, "Query Paginated Dataloader invoked {:?}", keys);
        let mut h = HashMap::new();
        let mut concurrent_f = vec![];
        for query_key in keys {
            let future_get = || async move {
                self.ctx
                    .dynamodb_client
                    .clone()
                    .query_node_edges(
                        query_key.cursor.clone(),
                        query_key.edges.clone(),
                        query_key.r#type.clone(),
                        self.ctx.dynamodb_table_name.clone(),
                        self.index.clone(),
                    )
                    .await
                    .map(|r| (query_key.clone(), r))
            };
            concurrent_f.push(future_get());
        }

        let b = futures_util::future::try_join_all(concurrent_f).await.map_err(|err| {
            log::error!(self.ctx.trace_id, "Error while querying: {:?}", err);
            QueryTypePaginatedLoaderError::QueryError
        })?;

        for (q, r) in b {
            h.insert(q, r);
        }

        log::info!(self.ctx.trace_id, "Query Paginated Dataloader executed");
        Ok(h)
    }
}

pub fn get_loader_paginated_query_type(
    ctx: Arc<DynamoDBContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QueryTypePaginatedLoader, LruCache> {
    DataLoader::with_cache(
        QueryTypePaginatedLoader { ctx, index },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
