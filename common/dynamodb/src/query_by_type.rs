use dynomite::{Attribute, DynamoDbExt};
use futures_util::TryStreamExt;
use indexmap::map::Entry;
use indexmap::IndexMap;
use itertools::Itertools;
use quick_error::quick_error;
use rusoto_dynamodb::QueryInput;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info_span, Instrument};

use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::paginated::{QueryResult, QueryValue};
use crate::{DynamoDBContext, DynamoDBRequestedIndex};

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

pub struct QueryTypeLoader {
    ctx: Arc<DynamoDBContext>,
    index: DynamoDBRequestedIndex,
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
        log::debug!(self.ctx.trace_id, "Query Type Dataloader invoked {:?}", keys);
        let mut h = HashMap::new();
        let mut concurrent_f = vec![];
        for query_key in keys {
            let mut exp = dynomite::attr_map! {
                ":pk" => query_key.r#type.clone(),
            };
            let edges_len = query_key.edges.len();
            let mut exp_att_name = HashMap::from([
                ("#pk".to_string(), self.index.pk()),
                ("#type".to_string(), "__type".to_string()),
            ]);
            let sk_string = if edges_len > 0 {
                exp_att_name.insert("#relationname".to_string(), "__relation_names".to_string());
                let edges = query_key
                    .edges
                    .iter()
                    .enumerate()
                    .map(|(index, q)| {
                        exp.insert(format!(":relation{}", index), q.clone().into_attr());
                        format!(" contains(#relationname, :relation{})", index)
                    })
                    .join(" OR ");
                exp.insert(":type".to_string(), query_key.r#type.clone().into_attr());
                Some(format!("begins_with(#type, :type) OR {edges}"))
            } else {
                None
            };

            let input: QueryInput = QueryInput {
                table_name: self.ctx.dynamodb_table_name.clone(),
                key_condition_expression: Some("#pk = :pk".to_string()),
                filter_expression: sk_string,
                index_name: self.index.to_index_name(),
                expression_attribute_values: Some(exp),
                expression_attribute_names: Some(exp_att_name),

                ..Default::default()
            };
            let future_get = || async move {
                self.ctx
                    .dynamodb_client
                    .clone()
                    .query_pages(input)
                    .inspect_err(|err| {
                        log::error!(self.ctx.trace_id, "Query By Type Error {:?}", err);
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
                            let pk = curr.get("__pk").and_then(|x| x.s.as_ref()).expect("can't fail");
                            let sk = curr.get("__sk").and_then(|y| y.s.clone()).expect("Can't fail");
                            let relation_names = curr.get("__relation_names").and_then(|y| y.ss.clone());

                            match acc.values.entry(pk.clone()) {
                                Entry::Vacant(vac) => {
                                    let mut value = QueryValue {
                                        node: None,
                                        edges: IndexMap::with_capacity(5),
                                    };

                                    // If it's the entity
                                    if sk.starts_with(format!("{}#", &query_key.r#type).as_str()) {
                                        value.node = Some(curr.clone());
                                    // If it's a relation
                                    } else if let Some(edge) = query_key.edges.iter().find(|edge| {
                                        relation_names
                                            .as_ref()
                                            .map(|x| x.contains(edge))
                                            .unwrap_or_else(|| false)
                                    }) {
                                        value.edges.insert(edge.clone(), vec![curr.clone()]);
                                    }

                                    vac.insert(value);
                                }
                                Entry::Occupied(mut oqp) => {
                                    if sk.starts_with(format!("{}#", &query_key.r#type).as_str()) {
                                        oqp.get_mut().node = Some(curr);
                                    } else if let Some(edge) = query_key.edges.iter().find(|edge| {
                                        relation_names
                                            .as_ref()
                                            .map(|x| x.contains(edge))
                                            .unwrap_or_else(|| false)
                                    }) {
                                        oqp.get_mut().edges.entry(edge.clone()).or_default().push(curr);
                                    }
                                }
                            };
                            Ok((query_key, acc))
                        },
                    )
                    .instrument(info_span!("fetch query by type"))
                    .await
            };
            concurrent_f.push(future_get());
        }

        let b = futures_util::future::try_join_all(concurrent_f)
            .instrument(info_span!("fetch query by type concurrent"))
            .await
            .map_err(|err| {
                log::error!(self.ctx.trace_id, "Error while querying: {:?}", err);
                QueryTypeLoaderError::QueryError
            })?;

        for (q, r) in b {
            h.insert(q, r);
        }

        Ok(h)
    }
}

pub fn get_loader_query_type(
    ctx: Arc<DynamoDBContext>,
    index: DynamoDBRequestedIndex,
) -> DataLoader<QueryTypeLoader, LruCache> {
    DataLoader::with_cache(
        QueryTypeLoader { ctx, index },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
