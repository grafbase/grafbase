use crate::constant::{PK, RELATION_NAMES, SK};
use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::paginated::{QueryResult, QueryValue};
use crate::runtime::Runtime;
use crate::{DynamoDBContext, DynamoDBRequestedIndex};
use dynomite::{Attribute, DynamoDbExt};
use futures_util::TryStreamExt;
use graph_entities::{NodeID, ID};
use indexmap::{map::Entry, IndexMap};
use rusoto_dynamodb::QueryInput;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "tracing")]
use tracing::{info_span, Instrument};
#[cfg(not(feature = "wasm"))]
#[derive(Debug, Clone, thiserror::Error)]
pub enum QuerySingleRelationLoaderError {
    #[error("An internal error happened")]
    UnknownError,
    #[error("An internal error happened while fetching a list of entities")]
    QueryError,
}

pub struct QuerySingleRelationLoader {
    ctx: Arc<DynamoDBContext>,
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
        log::debug!(self.ctx.trace_id, "QuerySingleRelation Dataloader invoked {:?}", keys);
        let mut h = HashMap::new();
        let mut concurrent_f = vec![];
        for query_key in keys {
            // TODO: Handle this when dealing with Custom ID
            let Ok(pk) = NodeID::from_borrowed(&query_key.parent_pk) else {
                h.insert(query_key.clone(), QueryResult::default());
                continue;
            };
            let mut exp = dynomite::attr_map! {
                ":pk" => pk.to_string(),
            };

            let mut exp_attr = HashMap::with_capacity(3);
            exp_attr.insert("#relationname".to_string(), RELATION_NAMES.to_string());
            exp_attr.insert("#pk".to_string(), self.index.pk());

            exp.insert(":relationname".to_string(), query_key.relation_name.clone().into_attr());

            let sk_string = Some("contains(#relationname, :relationname)".to_string());

            let input: QueryInput = QueryInput {
                table_name: self.ctx.dynamodb_table_name.clone(),
                key_condition_expression: Some("#pk = :pk".to_string()),
                filter_expression: sk_string,
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

                            match acc.values.entry(pk.to_string()) {
                                Entry::Vacant(vac) => {
                                    let mut value = QueryValue {
                                        node: None,
                                        constraints: Vec::new(),
                                        edges: IndexMap::with_capacity(5),
                                    };

                                    match (pk, sk) {
                                        (ID::NodeID(_), ID::NodeID(_)) => {
                                            value.node = Some(curr.clone());
                                        }
                                        (ID::ConstraintID(_), ID::ConstraintID(_)) => {
                                            value.constraints.push(curr);
                                        }
                                        _ => {}
                                    }

                                    vac.insert(value);
                                }
                                Entry::Occupied(mut oqp) => match (pk, sk) {
                                    (ID::NodeID(_), ID::NodeID(_)) => {
                                        oqp.get_mut().node = Some(curr);
                                    }
                                    (ID::ConstraintID(_), ID::ConstraintID(_)) => {
                                        oqp.get_mut().constraints.push(curr);
                                    }
                                    _ => {}
                                },
                            };
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
            QuerySingleRelationLoaderError::QueryError
        })?;

        for (q, r) in b {
            h.insert(q, r);
        }

        log::debug!(self.ctx.trace_id, "Query Dataloader executed {:?}", keys);
        Ok(h)
    }
}

pub fn get_loader_single_relation_query(
    ctx: Arc<DynamoDBContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QuerySingleRelationLoader, LruCache> {
    DataLoader::with_cache(
        QuerySingleRelationLoader { ctx, index },
        |f| Runtime::locate().spawn(f),
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
