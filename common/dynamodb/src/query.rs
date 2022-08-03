use dynomite::{Attribute, DynamoDbExt};
use futures_util::TryStreamExt;
use indexmap::{map::Entry, IndexMap};
use itertools::Itertools;
use quick_error::quick_error;
use rusoto_dynamodb::QueryInput;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info_span, Instrument};

use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::model::constraint::db::ConstraintID;
use crate::paginated::{QueryResult, QueryValue};
use crate::{DynamoDBContext, DynamoDBRequestedIndex};

// TODO: Should ensure Rosoto Errors impl clone
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
    ctx: Arc<DynamoDBContext>,
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
        log::debug!(self.ctx.trace_id, "Query Dataloader invoked {:?}", keys);
        let mut h = HashMap::new();
        let mut concurrent_f = vec![];
        for query_key in keys {
            let mut exp = dynomite::attr_map! {
                ":pk" => query_key.pk.clone(),
            };
            let edges_len = query_key.edges.len();

            let mut exp_attr = HashMap::with_capacity(3);
            exp_attr.insert("#pk".to_string(), self.index.pk());

            if edges_len > 0 {
                exp_attr.insert("#relationname".to_string(), "__relation_names".to_string());
                exp_attr.insert("#type".to_string(), "__type".to_string());
            }

            let sk_string = if edges_len > 0 {
                let edges = query_key
                    .edges
                    .iter()
                    .enumerate()
                    .map(|(index, q)| {
                        exp.insert(format!(":relation{}", index), q.clone().into_attr());
                        format!(" contains(#relationname, :relation{})", index)
                    })
                    .join(" OR ");

                let ty_attr = query_key
                    .pk
                    .rsplit_once('#')
                    .map(|x| x.0)
                    .unwrap_or_else(|| "")
                    .to_string()
                    .into_attr();

                exp.insert(":type".to_string(), ty_attr);
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
                expression_attribute_names: Some(exp_attr),

                ..Default::default()
            };
            let future_get = || async move {
                self.ctx
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
                            let pk = curr.get("__pk").and_then(|x| x.s.as_ref()).expect("can't fail");
                            let sk = curr.get("__sk").and_then(|y| y.s.clone()).expect("Can't fail");
                            let relation_names = curr.get("__relation_names").and_then(|y| y.ss.clone());

                            match acc.values.entry(pk.clone()) {
                                Entry::Vacant(vac) => {
                                    let mut value = QueryValue {
                                        node: None,
                                        constraints: Vec::new(),
                                        edges: IndexMap::with_capacity(5),
                                    };

                                    // If it's the entity
                                    if sk.eq(pk) {
                                        value.node = Some(curr.clone());
                                    } else if ConstraintID::try_from(sk).is_ok() {
                                        value.constraints.push(curr);
                                    // If it's a relation
                                    } else if let Some(edges) = relation_names {
                                        for edge in edges {
                                            value.edges.insert(edge, vec![curr.clone()]);
                                        }
                                    }

                                    vac.insert(value);
                                }
                                Entry::Occupied(mut oqp) => {
                                    if sk.eq(pk) {
                                        oqp.get_mut().node = Some(curr);
                                    } else if ConstraintID::try_from(sk).is_ok() {
                                        oqp.get_mut().constraints.push(curr);
                                    } else if let Some(edges) = relation_names {
                                        for edge in edges {
                                            oqp.get_mut().edges.entry(edge).or_default().push(curr.clone());
                                        }
                                    }
                                }
                            };
                            Ok((query_key, acc))
                        },
                    )
                    .instrument(info_span!("fetch query"))
                    .await
            };
            concurrent_f.push(future_get());
        }

        let b = futures_util::future::try_join_all(concurrent_f)
            .instrument(info_span!("fetch query concurrent"))
            .await
            .map_err(|err| {
                log::error!(self.ctx.trace_id, "Error while querying: {:?}", err);
                QueryLoaderError::QueryError
            })?;

        for (q, r) in b {
            h.insert(q, r);
        }

        log::debug!(self.ctx.trace_id, "Query Dataloader executed {:?}", keys);
        Ok(h)
    }
}

pub fn get_loader_query(ctx: Arc<DynamoDBContext>, index: DynamoDBRequestedIndex) -> DataLoader<QueryLoader, LruCache> {
    DataLoader::with_cache(
        QueryLoader { ctx, index },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(256),
    )
    .max_batch_size(10)
    .delay(Duration::from_millis(2))
}
