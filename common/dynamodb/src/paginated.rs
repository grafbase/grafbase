//! Extention interfaces for rusoto `DynamoDb`

use crate::DynamoDBRequestedIndex;
use dynomite::Attribute;
use futures::TryFutureExt;
use indexmap::map::Entry;
use indexmap::IndexMap;
use quick_error::quick_error;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{AttributeValue, DynamoDb, QueryError, QueryInput};
use std::collections::HashMap;

/// A Cursor.
/// The first elements are the most recents ones.
/// The last elements are the most anciens.
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum PaginatedCursor {
    Forward {
        exclusive_last_key: Option<String>,
        first: usize,
    },
    Backward {
        exclusive_first_key: Option<String>,
        last: usize,
    },
}

quick_error! {
    #[derive(Debug, Clone)]
    pub enum CursorCreation {
        SameParameterSameTime {
            display("The \"first\" and \"last\" parameters cannot exist at the same time.")
        }
        FirstNonNegative {
            display("The \"first\" parameter must be a non-negative number")
        }
        LastNonNegative {
            display("The \"last\" parameter must be a non-negative number")
        }
        FirstAndBeforeSameTime {
            display("The \"first\" and \"before\" parameter cannot exist at the same time.")
        }
        LastAndAfterSameTime {
            display("The \"last\" and \"after\" parameter cannot exist at the same time.")
        }
        Direction {
            display("You must choose a Pagination direction by having the \"first\" or \"last\" parameter.")
        }
    }
}

impl PaginatedCursor {
    /// To create the Cursor from GraphQL Input
    #[allow(
        clippy::missing_const_for_fn,
        /* reason = "False positive, destructors cannot be evaluated at compile-time" */
    )]
    pub fn from_graphql(
        first: Option<usize>,
        last: Option<usize>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<Self, CursorCreation> {
        match (first, after, last, before) {
            (Some(_), _, Some(_), _) => Err(CursorCreation::SameParameterSameTime),
            (Some(_), _, _, Some(_)) => Err(CursorCreation::FirstAndBeforeSameTime),
            (_, Some(_), Some(_), _) => Err(CursorCreation::LastAndAfterSameTime),
            (Some(first), after, None, None) => Ok(Self::Forward {
                exclusive_last_key: after,
                first,
            }),
            (None, None, Some(last), before) => Ok(Self::Backward {
                exclusive_first_key: before,
                last,
            }),
            (None, _, None, _) => Err(CursorCreation::Direction),
        }
    }

    const fn scan_index_forward(&self) -> bool {
        match self {
            PaginatedCursor::Forward { .. } => false,
            PaginatedCursor::Backward { .. } => true,
        }
    }

    fn pagination_string(&self) -> Option<String> {
        match self {
            PaginatedCursor::Forward { exclusive_last_key, .. } => exclusive_last_key.clone(),
            PaginatedCursor::Backward {
                exclusive_first_key, ..
            } => exclusive_first_key.clone(),
        }
    }

    const fn limit(&self) -> usize {
        match self {
            PaginatedCursor::Forward { first, .. } => *first,
            PaginatedCursor::Backward { last, .. } => *last,
        }
    }
}

/// Extension methods for DynamoDb client types
///
/// A default impl is provided for `DynamoDb  Clone + Send + Sync + 'static` which adds autopaginating `Stream` interfaces that require
/// taking ownership.
#[async_trait::async_trait]
pub trait DynamoDbExtPaginated {
    /// Specialized Query to fetch
    /// Will return as soon as we have `limit` items with pagination or if we do have
    /// less than limit without pagination.
    /// Return values are like
    async fn query_node_edges(
        self,
        trace_id: &str,
        cursor: PaginatedCursor,
        edges: Vec<String>,
        node: String,
        table: String,
        index: DynamoDBRequestedIndex,
    ) -> Result<QueryResult, RusotoError<QueryError>>;
}

#[derive(Debug, Clone)]
pub struct QueryValue {
    pub node: Option<HashMap<String, AttributeValue>>,
    pub edges: IndexMap<String, Vec<HashMap<String, AttributeValue>>>,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Returned values by PK
    pub values: IndexMap<String, QueryValue>,
    pub last_evaluated_key: Option<String>,
}

#[async_trait::async_trait]
impl<D> DynamoDbExtPaginated for D
where
    D: DynamoDb + Clone + Send + Sync + 'static,
{
    async fn query_node_edges(
        self,
        trace_id: &str,
        cursor: PaginatedCursor,
        edges: Vec<String>,
        node: String,
        table: String,
        index: DynamoDBRequestedIndex,
    ) -> Result<QueryResult, RusotoError<QueryError>> {
        let mut exp = dynomite::attr_map! {
            ":pk" => node.clone(),
        };
        let mut edges_and_node = edges.clone();
        edges_and_node.push(node.clone());

        let edges_len = edges_and_node.len();
        let sk_string = if edges_len > 0 {
            Some(
                edges_and_node
                    .into_iter()
                    .enumerate()
                    .map(|(index, q)| {
                        exp.insert(format!(":type{}", index), q.into_attr());
                        format!(" begins_with(#type, :type{})", index)
                    })
                    .fold(String::new(), |acc, cur| {
                        if !acc.is_empty() {
                            format!("{} OR {}", cur, acc)
                        } else {
                            cur
                        }
                    }),
            )
        } else {
            None
        };

        let mut exp_att_name = HashMap::from([
            ("#pk".to_string(), index.pk()),
            ("#type".to_string(), "__type".to_string()),
        ]);
        let pagination_string = cursor.pagination_string();
        let key_condition_expression = match (&pagination_string, &cursor) {
            (Some(_), PaginatedCursor::Forward { .. }) => Some("#pk = :pk AND #sk < :pkorder".to_string()),
            (Some(_), PaginatedCursor::Backward { .. }) => Some("#pk = :pk AND #sk > :pkorder".to_string()),
            _ => Some("#pk = :pk".to_string()),
        };

        pagination_string.map(|x| {
            exp_att_name.insert("#sk".to_string(), index.sk());
            exp.insert(":pkorder".to_string(), x.into_attr())
        });

        let input: QueryInput = QueryInput {
            table_name: table,
            key_condition_expression,
            filter_expression: sk_string,
            index_name: index.to_index_name(),
            expression_attribute_values: Some(exp),
            expression_attribute_names: Some(exp_att_name),
            scan_index_forward: Some(cursor.scan_index_forward()),
            ..Default::default()
        };

        let limit = cursor.limit();

        let mut result = QueryResult {
            values: IndexMap::with_capacity(limit),
            last_evaluated_key: None,
        };

        #[allow(clippy::large_enum_variant)]
        enum PageState {
            Next(Option<HashMap<String, AttributeValue>>, QueryInput),
            // Optional Cursor
            End(Option<String>),
        }

        let mut actual_state = PageState::Next(None, input);

        // While we do not have enough value, we try to get more.
        while result.values.len() <= limit {
            let (exclusive_start_key, input) = match actual_state {
                PageState::Next(start, input) => (start, input),
                PageState::End(_) => {
                    break;
                }
            };
            log::debug!(trace_id, "QueryPaginated Input {:?}", input);
            let resp = self
                .query(QueryInput {
                    exclusive_start_key: exclusive_start_key.clone(),
                    ..input.clone()
                })
                .inspect_err(|err| {
                    log::error!(trace_id, "Query Paginated Error {:?}", err);
                })
                .await?;

            // For each items in the result, we'll group them by pk.
            // As soon as we have more than limit items, we return.
            for x in resp.items.unwrap_or_default().into_iter() {
                let len = result.values.len();
                if len <= limit {
                    let pk = x.get("__pk").and_then(|y| y.s.clone()).expect("Can't fail");
                    let sk = x.get("__sk").and_then(|y| y.s.clone()).expect("Can't fail");
                    match result.values.entry(pk.clone()) {
                        Entry::Vacant(vac) => {
                            // We do insert the PK just before inserting it the n+1 element.
                            if len == limit {
                                result.last_evaluated_key = Some(pk.clone());
                            }

                            let mut value = QueryValue {
                                node: None,
                                edges: IndexMap::with_capacity(5),
                            };

                            if sk.starts_with(&node) {
                                value.node = Some(x.clone());
                            } else if let Some(edge) = edges.iter().find(|edge| sk.starts_with(edge.as_str())) {
                                value.edges.insert(edge.clone(), vec![x.clone()]);
                            }

                            vac.insert(value);
                        }
                        Entry::Occupied(mut oqp) => {
                            if sk.starts_with(&node) {
                                oqp.get_mut().node = Some(x);
                                continue;
                            }

                            if let Some(edge) = edges.iter().find(|edge| sk.starts_with(edge.as_str())) {
                                match oqp.get_mut().edges.entry(edge.clone()) {
                                    Entry::Vacant(vac) => {
                                        vac.insert(vec![x]);
                                    }
                                    Entry::Occupied(mut oqp) => {
                                        oqp.get_mut().push(x);
                                    }
                                };
                                continue;
                            }
                        }
                    };
                }
            }

            // Multiple cases:
            // - Enough elements (n+1) so we won't go into another cycle.
            // - No enough elements, but we can't fetch another elements.
            // - Not enough elements, but we can still fetch more, so we'll go into another cycle
            if result.values.len() > limit || result.last_evaluated_key.is_some() {
                result.values = result
                    .values
                    .into_iter()
                    // If we have enough elements we need to filter the n+1 one.
                    // If we do not have enough elements, we'll need to fetch more, but we do not need
                    // to filter the n+1 element.
                    .filter(|(k, _)| match &result.last_evaluated_key {
                        // If we have an element which is the last_evaluated_key it means it's the n+1
                        // element we fetched, so we must discard it.
                        Some(last) => !k.eq(last),
                        _ => true,
                    })
                    .take(limit)
                    .collect();
            }

            actual_state = match resp.last_evaluated_key {
                Some(elm) => PageState::Next(Some(elm), input),
                None => PageState::End(
                    resp.last_evaluated_key
                        .and_then(|x| x.get("__pk").and_then(|s| s.s.clone())),
                ),
            };
        }

        Ok(result)
    }
}
