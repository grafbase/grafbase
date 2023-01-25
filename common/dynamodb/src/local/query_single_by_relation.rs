use super::bridge_api;
use super::types::{Operation, Sql, SqlValue};
use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::paginated::QueryResult;
use crate::runtime::Runtime;
use crate::{DynamoDBRequestedIndex, LocalContext};
use graph_entities::{NodeID, ID};
use indexmap::IndexMap;
use maplit::hashmap;
use quick_error::quick_error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

quick_error! {
    #[derive(Debug, Clone)]
    pub enum QuerySingleRelationLoaderError {
        UnknownError {
            display("An internal error happened")
        }
        QueryError {
            display("An internal error happened while fetching a list of entities")
        }
    }
}

pub struct QuerySingleRelationLoader {
    local_context: Arc<LocalContext>,
    index: DynamoDBRequestedIndex,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct QuerySingleRelationKey {
    parent_pk: String,
    relation_name: String,
}

impl QuerySingleRelationKey {
    pub fn new(parent_pk: String, relation_name: String) -> Self {
        Self {
            parent_pk,
            relation_name,
        }
    }
}

#[async_trait::async_trait]
impl Loader<QuerySingleRelationKey> for QuerySingleRelationLoader {
    type Value = QueryResult;
    type Error = QuerySingleRelationLoaderError;

    async fn load(
        &self,
        keys: &[QuerySingleRelationKey],
    ) -> Result<HashMap<QuerySingleRelationKey, Self::Value>, Self::Error> {
        let mut query_result = HashMap::new();
        let mut concurrent_futures = vec![];
        for query_key in keys {
            let parent_pk = match NodeID::from_borrowed(&query_key.parent_pk) {
                Ok(id) => id,
                Err(_) => {
                    query_result.insert(query_key.clone(), QueryResult::default());
                    continue;
                }
            };

            let value_map = hashmap! {
                "parent_pk" => SqlValue::String(parent_pk.to_string()),
                "relation_name" => SqlValue::String(query_key.relation_name.clone()),
            };

            let (query, values) = Sql::SelectSingleRelation(self.index.pk()).compile(value_map);

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
                .map_err(|_| QuerySingleRelationLoaderError::QueryError)?;

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

                        let value = accumulator.values.entry(sk.to_string()).or_default();

                        match (pk, sk) {
                            (ID::NodeID(_), ID::NodeID(_)) => {
                                value.node = Some(current.document.clone());
                            }
                            (ID::ConstraintID(constraint_id), ID::ConstraintID(_)) => {
                                value.constraints.push((constraint_id, current.document.clone()));
                            }
                            _ => {}
                        }

                        Ok::<_, QuerySingleRelationLoaderError>((query_key, accumulator))
                    },
                )
            };
            concurrent_futures.push(future());
        }

        let joined_futures = futures_util::future::try_join_all(concurrent_futures)
            .await
            .map_err(|_| QuerySingleRelationLoaderError::QueryError)?;

        query_result.extend(joined_futures.into_iter().collect::<HashMap<_, _>>());

        Ok(query_result)
    }
}

pub fn get_loader_single_relation_query(
    local_context: Arc<LocalContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QuerySingleRelationLoader, LruCache> {
    DataLoader::with_cache(
        QuerySingleRelationLoader { local_context, index },
        |f| Runtime::locate().spawn(f),
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
