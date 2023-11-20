//! Extention interfaces for rusoto `DynamoDb`

use std::collections::HashMap;

use dynomite::Attribute;
use futures::TryFutureExt;
use graph_entities::{ConstraintID, ID};
use indexmap::{map::Entry, IndexMap};
use itertools::Itertools;
use quick_error::quick_error;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{AttributeValue, DynamoDb, QueryError, QueryInput};
use tracing::{info_span, Instrument};

use crate::{
    constant::{OWNED_BY, PK, RELATION_NAMES, SK, TYPE},
    DynamoDBRequestedIndex, QueryTypePaginatedKey,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ParentEdge {
    pub relation_name: String,
    pub parent_id: String,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum PaginationOrdering {
    ASC,
    DESC,
}

impl PaginationOrdering {
    pub fn is_asc(&self) -> bool {
        matches!(self, PaginationOrdering::ASC)
    }

    pub fn is_desc(&self) -> bool {
        matches!(self, PaginationOrdering::DESC)
    }
}

/// A Cursor.
/// The first elements are the most recents ones.
/// The last elements are the most anciens.
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum PaginatedCursor {
    // after
    Forward {
        exclusive_last_key: Option<String>,
        first: usize,
        maybe_parent_edge: Option<ParentEdge>,
    },
    // before
    Backward {
        exclusive_first_key: Option<String>,
        last: usize,
        maybe_parent_edge: Option<ParentEdge>,
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
        nested: Option<ParentEdge>,
    ) -> Result<Self, CursorCreation> {
        match (first, after, last, before) {
            (Some(_), _, Some(_), _) => Err(CursorCreation::SameParameterSameTime),
            (Some(_), _, _, Some(_)) => Err(CursorCreation::FirstAndBeforeSameTime),
            (_, Some(_), Some(_), _) => Err(CursorCreation::LastAndAfterSameTime),
            (Some(first), after, None, None) => Ok(Self::Forward {
                exclusive_last_key: after,
                first,
                maybe_parent_edge: nested,
            }),
            (None, None, Some(last), before) => Ok(Self::Backward {
                exclusive_first_key: before,
                last,
                maybe_parent_edge: nested,
            }),
            (None, _, None, _) => Err(CursorCreation::Direction),
        }
    }

    pub fn is_forward(&self) -> bool {
        matches!(self, Self::Forward { .. })
    }

    pub fn is_backward(&self) -> bool {
        matches!(self, Self::Backward { .. })
    }

    pub fn maybe_parent_edge(&self) -> Option<&ParentEdge> {
        match self {
            PaginatedCursor::Forward { maybe_parent_edge, .. }
            | PaginatedCursor::Backward { maybe_parent_edge, .. } => maybe_parent_edge.as_ref(),
        }
    }

    pub fn maybe_origin(&self) -> Option<String> {
        match self {
            PaginatedCursor::Forward { exclusive_last_key, .. } => exclusive_last_key.clone(),
            PaginatedCursor::Backward {
                exclusive_first_key, ..
            } => exclusive_first_key.clone(),
        }
    }

    fn relation_name(&self) -> Option<String> {
        self.maybe_parent_edge()
            .map(|parent_edge| parent_edge.relation_name.clone())
    }

    pub fn nested_parent_pk(&self) -> Option<String> {
        self.maybe_parent_edge()
            .map(|parent_edge| parent_edge.parent_id.clone())
    }

    fn is_nested_relation(&self) -> bool {
        self.maybe_parent_edge().is_some()
    }

    pub fn limit(&self) -> usize {
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
        query_key: QueryTypePaginatedKey,
        table: String,
        index: DynamoDBRequestedIndex,
        owned_by: Option<&str>,
    ) -> Result<QueryResult, RusotoError<QueryError>>;
}

#[derive(Debug, Clone)]
pub struct QueryValue {
    pub node: Option<HashMap<String, AttributeValue>>,
    pub edges: IndexMap<String, Vec<HashMap<String, AttributeValue>>>,
    /// Constraints are other kind of row we can store, it'll add data over a node
    pub constraints: Vec<(ConstraintID<'static>, HashMap<String, AttributeValue>)>,
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

#[allow(clippy::iter_without_into_iter)]
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

#[derive(Debug, Clone, Default)]
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
        query_key: QueryTypePaginatedKey,
        table: String,
        index: DynamoDBRequestedIndex,
        owned_by: Option<&str>,
    ) -> Result<QueryResult, RusotoError<QueryError>> {
        let QueryTypePaginatedKey {
            r#type: node_type,
            edges,
            cursor,
            ordering,
        } = query_key;
        let node_type = node_type.to_lowercase();
        let mut exp = dynomite::attr_map! {
            ":pk" => cursor.nested_parent_pk().unwrap_or_else(|| node_type.clone())
        };

        let edges_len = edges.len();

        // TODO: consolidate these branches
        let primary_index = if cursor.is_nested_relation() {
            PK.to_string()
        } else {
            index.pk()
        };

        let sort_index = if cursor.is_nested_relation() {
            SK.to_string()
        } else {
            index.sk()
        };

        let mut exp_att_name = HashMap::from([
            ("#pk".to_string(), primary_index.clone()),
            ("#type".to_string(), TYPE.to_string()),
            ("#sk".to_string(), sort_index.clone()),
        ]);

        let edge_query = if edges_len > 0 {
            exp_att_name.insert("#relationname".to_string(), RELATION_NAMES.to_string());
            let edges = edges
                .clone()
                .into_iter()
                .enumerate()
                .map(|(index, q)| {
                    exp.insert(format!(":relation{index}"), q.into_attr());
                    format!(" contains(#relationname, :relation{index})")
                })
                .join(" OR ");

            format!("OR ({edges})")
        } else {
            String::new()
        };

        let mut filter_expression = if cursor.is_nested_relation() {
            exp_att_name.insert("#relationname".to_string(), RELATION_NAMES.to_string());

            exp.insert(":relation".to_string(), cursor.relation_name().into_attr());
            exp.insert(":type".to_string(), node_type.clone().into_attr());
            format!("contains(#relationname, :relation) AND #type = :type {edge_query}")
        } else {
            exp.insert(":type".to_string(), node_type.clone().into_attr());
            format!("#type = :type {edge_query}")
        };

        if let Some(owned_by) = owned_by {
            exp_att_name.insert("#owned_by_name".to_string(), OWNED_BY.to_string());
            exp.insert(":owned_by_value".to_string(), owned_by.to_string().into_attr());
            filter_expression += " AND contains(#owned_by_name, :owned_by_value)";
        }

        log::debug!(trace_id, "FilterExpression: {filter_expression}");

        let scan_index_forward = if cursor.is_forward() {
            ordering.is_asc()
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
            !ordering.is_asc()
        };

        let key_condition_expression = Some("#pk = :pk AND begins_with(#sk, :type) ".to_string());
        let exclusive_start_key = cursor.maybe_origin().map(|origin| {
            let attr = origin.into_attr();
            let mut key = HashMap::from([
                (
                    primary_index,
                    exp.get(":pk").expect(":pk is used in the query").clone().into_attr(),
                ),
                (sort_index, attr.clone()),
            ]);
            if !key.contains_key(PK) {
                key.insert(PK.to_string(), attr.clone());
                key.insert(SK.to_string(), attr.clone());
            }
            key
        });
        let input: QueryInput = QueryInput {
            table_name: table,
            key_condition_expression,
            filter_expression: Some(filter_expression),
            index_name: if cursor.is_nested_relation() {
                None
            } else {
                index.to_index_name()
            },
            expression_attribute_values: Some(exp),
            expression_attribute_names: Some(exp_att_name),
            scan_index_forward: Some(scan_index_forward),
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
            End,
        }

        let mut actual_state = PageState::Next(exclusive_start_key, input);

        // While we do not have enough value, we try to get more.
        while result.values.len() <= limit {
            let (exclusive_start_key, input) = match actual_state {
                PageState::Next(start, input) => (start, input),
                PageState::End => {
                    break;
                }
            };
            let input = QueryInput {
                exclusive_start_key,
                ..input
            };
            log::debug!(trace_id, "QueryPaginated Input {:?}", input);
            let resp = crate::retry::rusoto_retry(|| {
                self.query(input.clone()).inspect_err(|err| {
                    log::error!(trace_id, "Query Paginated Error {:?}", err);
                })
            })
            .instrument(info_span!("fetch paginated"))
            .await?;

            // For each items in the result, we'll group them by pk.
            // As soon as we have more than limit items, we return.
            for x in resp.items.unwrap_or_default() {
                let len = result.values.len();
                if len <= limit {
                    let pk = ID::try_from(x.get(PK).and_then(|x| x.s.as_ref()).expect("can't fail").clone())
                        .expect("Can't fail");
                    let sk = ID::try_from(x.get(SK).and_then(|x| x.s.as_ref()).expect("can't fail").clone())
                        .expect("Can't fail");
                    let relation_names = x.get(RELATION_NAMES).and_then(|y| y.ss.clone());

                    let is_top_level_nested = cursor
                        .nested_parent_pk()
                        .filter(|query_pk| query_pk == &pk.to_string())
                        .is_some();

                    let key = if is_top_level_nested {
                        sk.to_string()
                    } else {
                        pk.to_string()
                    };

                    match result.values.entry(key) {
                        Entry::Vacant(vac) => {
                            // We do insert the PK just before inserting it the n+1 element.
                            if len == limit {
                                result.last_evaluated_key = Some(pk.to_string());
                            }

                            let mut value = QueryValue {
                                node: None,
                                constraints: Vec::new(),
                                edges: IndexMap::with_capacity(5),
                            };

                            match (pk, sk) {
                                (ID::NodeID(_), ID::NodeID(sk)) => {
                                    if sk.ty() == node_type {
                                        value.node = Some(x.clone());
                                    } else if let Some(edge) = edges.iter().find(|edge| {
                                        relation_names.as_ref().map(|x| x.contains(edge)).unwrap_or_default()
                                    }) {
                                        value.edges.insert(edge.clone(), vec![x.clone()]);
                                    }
                                }
                                (ID::ConstraintID(constraint_id), ID::ConstraintID(_)) => {
                                    value.constraints.push((constraint_id, x));
                                }
                                _ => {}
                            }

                            vac.insert(value);
                        }
                        Entry::Occupied(mut oqp) => match (pk, sk) {
                            (ID::NodeID(_), ID::NodeID(sk)) => {
                                if sk.ty() == node_type {
                                    oqp.get_mut().node = Some(x);
                                    continue;
                                }

                                if let Some(edge) = edges
                                    .iter()
                                    .find(|edge| relation_names.as_ref().map(|x| x.contains(edge)).unwrap_or_default())
                                {
                                    oqp.get_mut().edges.entry(edge.clone()).or_default().push(x);
                                    continue;
                                }
                            }
                            (ID::ConstraintID(constraint_id), ID::ConstraintID(_)) => {
                                oqp.get_mut().constraints.push((constraint_id, x));
                                continue;
                            }
                            _ => {}
                        },
                    };
                }
            }

            // Multiple cases:
            // - Enough elements (n+1) so we won't go into another cycle.
            // - No enough elements, but we can't fetch another elements.
            // - Not enough elements, but we can still fetch more, so we'll go into another cycle
            actual_state = if result.values.len() > limit {
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
                PageState::End
            } else {
                match resp.last_evaluated_key {
                    Some(elm) => PageState::Next(Some(elm), input),
                    None => PageState::End,
                }
            };
        }

        // Ordering of the items is independent of cursor direction. So if cursor dicrection
        // doesn't matches the record one, we must reverse the results.
        //                         after
        //                           ┌───────► first (forward)
        //                           │
        //              ─────────────┼───────────────► Record order
        //                           │
        // last (backward) ◄─────────┘
        //                         before
        if cursor.is_backward() {
            result.values.reverse();
        }
        Ok(result)
    }
}
