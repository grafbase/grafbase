use dynomite::AttributeValue;
use quick_error::quick_error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "tracing")]
use tracing::{info_span, Instrument};

use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::paginated::{DynamoDbExtPaginated, PaginatedCursor, PaginationOrdering, QueryResult};
use crate::runtime::Runtime;
use crate::{
    DynamoDBContext, DynamoDBRequestedIndex, OperationAuthorization, OperationAuthorizationError, RequestedOperation,
};

// TODO: Should ensure Rosoto Errors impl clone
quick_error! {
    #[derive(Debug, Clone)]
    pub enum QueryTypePaginatedLoaderError {
        QueryError {
            display("An internal error happened while fetching a list of entities")
        }
        AuthorizationError(err: OperationAuthorizationError) {
            from()
            source(err)
            display("Unauthorized")
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
    pub ordering: PaginationOrdering,
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
}

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
    type Value = Arc<QueryResult>;
    type Error = QueryTypePaginatedLoaderError;

    async fn load(
        &self,
        keys: &[QueryTypePaginatedKey],
    ) -> Result<HashMap<QueryTypePaginatedKey, Self::Value>, Self::Error> {
        log::debug!(self.ctx.trace_id, "Query Paginated Dataloader invoked {:?}", keys);
        let mut h = HashMap::new();
        let mut concurrent_f = vec![];
        let owned_by = match self.ctx.authorize_operation(RequestedOperation::List)? {
            OperationAuthorization::OwnerBased(owned_by) => Some(owned_by),
            OperationAuthorization::PrivateOrGroupBased => None,
        };
        for query_key in keys {
            let future_get = || async move {
                let req = self.ctx.dynamodb_client.clone().query_node_edges(
                    &self.ctx.trace_id,
                    query_key.clone(),
                    self.ctx.dynamodb_table_name.clone(),
                    self.index.clone(),
                    owned_by,
                );
                #[cfg(feature = "tracing")]
                let req = req.instrument(info_span!("fetch query by type paginated"));
                req.await.map(|r| (query_key.clone(), Arc::new(r)))
            };
            concurrent_f.push(future_get());
        }

        let b = futures_util::future::try_join_all(concurrent_f);
        #[cfg(feature = "tracing")]
        let b = b.instrument(info_span!("fetch query by type paginated concurrent"));
        let b = b.await.map_err(|err| {
            log::error!(self.ctx.trace_id, "Error while querying: {:?}", err);
            QueryTypePaginatedLoaderError::QueryError
        })?;

        for (q, r) in b {
            h.insert(q, r);
        }

        log::debug!(self.ctx.trace_id, "Query Paginated Dataloader executed");
        Ok(h)
    }
}

pub fn get_loader_paginated_query_type(
    ctx: Arc<DynamoDBContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QueryTypePaginatedLoader, LruCache> {
    DataLoader::with_cache(
        QueryTypePaginatedLoader { ctx, index },
        |f| Runtime::locate().spawn(f),
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
