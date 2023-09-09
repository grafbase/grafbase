use std::{collections::HashMap, sync::Arc, time::Duration};

use dataloader::{DataLoader, Loader, LruCache};
use dynomite::AttributeValue;
use graph_entities::ID;
use indexmap::IndexMap;
use maplit::hashmap;
use quick_error::quick_error;

use super::{
    bridge_api,
    types::{Operation, Sql, SqlValue},
};
use crate::{DynamoDBRequestedIndex, LocalContext};

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
            r#type: r#type.to_lowercase(),
            edges: {
                edges.sort();
                edges
            },
        }
    }
    fn ty(&self) -> &String {
        &self.r#type
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
        let mut concurrent_futures = vec![];
        for query_key in keys {
            let entity_type = query_key.ty().clone();

            let value_map = hashmap! {
                "entity_type" => SqlValue::String(entity_type),
                "edges" => SqlValue::VecDeque(query_key.edges.clone().into()),
            };

            // TODO: unify SelectType and SelectTypeWithEdges (suggested by @jakubadamw)
            let (query, values) = Sql::SelectType.compile(value_map);

            let future = || async move {
                let query_results = bridge_api::query(
                    Operation {
                        sql: query.to_string(),
                        values,
                        kind: None,
                    },
                    &self.local_context.bridge_port,
                )
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
                        let pk = ID::try_from(current.pk.clone()).expect("can't fail");
                        let sk = ID::try_from(current.sk.clone()).expect("can't fail");
                        let relation_names = current.relation_names.clone();

                        let value = accumulator.values.entry(pk.to_string()).or_default();

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

                        Ok::<_, QueryTypeLoaderError>((query_key, accumulator))
                    },
                )
            };
            concurrent_futures.push(future());
        }

        let joined_futures = futures_util::future::try_join_all(concurrent_futures)
            .await
            .map_err(|_| QueryTypeLoaderError::QueryError)?;

        Ok(joined_futures.into_iter().collect())
    }
}

pub fn get_loader_query_type(
    local_context: Arc<LocalContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QueryTypeLoader, LruCache> {
    DataLoader::with_cache(
        QueryTypeLoader { local_context, index },
        async_runtime::spawn,
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
