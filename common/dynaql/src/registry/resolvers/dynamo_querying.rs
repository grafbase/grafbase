use super::{ResolvedPaginationDirection, ResolvedPaginationInfo, ResolvedValue, ResolverTrait};

use crate::registry::relations::{MetaRelation, MetaRelationKind};
use crate::registry::{resolvers::ResolverContext, variables::VariableResolveDefinition};
use crate::{Context, Error, Value};
use dynamodb::constant::{INVERTED_INDEX_PK, SK};
use dynamodb::{
    DynamoDBBatchersData, PaginatedCursor, QueryKey, QuerySingleRelationKey, QueryTypePaginatedKey,
};
use graph_entities::cursor::PaginationCursor;
use graph_entities::ConstraintID;
use indexmap::IndexMap;
use itertools::Itertools;
use serde_json::Map;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::Hash;

use std::sync::Arc;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum DynamoResolver {
    /// A Query based on the PK and the SK
    ///
    /// We query dynamodb with PK and SK, we also query the edges based on the type infered.
    ///
    /// # Returns
    ///
    /// We expect this resolver to return a Value with this type, if for example.
    /// This resolver should ALWAYS be used for Unique Results.
    ///
    /// With a Blog example where Author would be an Edge:
    ///
    /// ```json
    /// {
    ///   data: {
    ///     "Blog": HashMap<String, AttributeValue>,
    ///     "Author": Vec<HashMap<String, AttributeValue>>,
    ///     "Edge2": Vec<HashMap<String, AttributeValue>>,
    ///   }
    /// }
    /// ```
    ///
    /// Event if the relation is unique, we'll return a Vec, it's the purpose of the EdgeResolver
    /// to determine if the schema is coherent and to fallback an error if it's not.
    ///
    /// The purpose of this resolver is only to resolve the n-1 level in an optimized way and to
    /// pass the results.
    QueryPKSK {
        pk: VariableResolveDefinition,
        sk: VariableResolveDefinition,
    },
    /// A Paginated Query based on the type of the entity.
    ///
    /// We query the reverted index by type to get a node and his edges.
    /// This Resolver is paginated.
    ///
    /// # Returns
    ///
    /// We expect this resolver to return a Value with this type, if for example.
    ///
    /// With a Blog example where Author would be an Edge:
    ///
    /// ```json
    /// {
    ///   paginationInfo: {
    ///     has_next: bool,
    ///     last_cursor: Option<String>
    ///
    ///     has_previous: bool,
    ///     first_cursor: Option<String>,
    ///
    ///     count: i32,
    ///   },
    ///   data: [{
    ///     "Blog": HashMap<String, AttributeValue>,
    ///     "Author": Vec<HashMap<String, AttributeValue>>,
    ///     "Edge2": Vec<HashMap<String, AttributeValue>>,
    ///   }]
    /// }
    /// ```
    ListResultByTypePaginated {
        r#type: VariableResolveDefinition,
        first: VariableResolveDefinition,
        last: VariableResolveDefinition,
        after: VariableResolveDefinition,
        before: VariableResolveDefinition,
        // (relation_name, parent_pk)
        // TODO: turn this into a struct
        nested: Option<(String, String)>,
    },
    /// A Query based on the PK and the relation name.
    ///
    /// # Returns
    ///
    /// We expect this resolver to return a Value with this type, if for example.
    /// This resolver should ALWAYS be used for Unique Results.
    ///
    /// With an Author edge:
    ///
    /// ```json
    /// {
    ///   data: {
    ///     "Author": Vec<HashMap<String, AttributeValue>>,
    ///   }
    /// }
    /// ```
    ///
    /// Even if the relation is unique, we'll return a Vec, it's the purpose of the EdgeResolver
    /// to determine if the schema is coherent and to fallback an error if it's not.
    QuerySingleRelation {
        parent_pk: String,
        relation_name: String,
    },
    /// A Query based on any unique field (id or a field marked with @unique)
    ///
    /// # Returns
    ///
    /// We expect this resolver to return a Value with this type, if for example.
    /// This resolver should ALWAYS be used for Unique Results.
    ///
    ///
    /// This resolver should ALWAYS be used for Unique Results.
    ///
    /// With a Blog example with an Author edge:
    ///
    /// (Note: this query returns edges only for by id queries)F
    ///
    /// ```json
    /// {
    ///   data: {
    ///     "Blog": HashMap<String, AttributeValue>,
    ///     "Author": Vec<HashMap<String, AttributeValue>>,
    ///     "Edge2": Vec<HashMap<String, AttributeValue>>,
    ///   }
    /// }
    /// ```
    QueryBy { by: VariableResolveDefinition },
}

#[async_trait::async_trait]
impl ResolverTrait for DynamoResolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        const PAGINATION_LIMIT: usize = 100;

        let batchers = &ctx.data::<Arc<DynamoDBBatchersData>>()?;
        let loader_item = &batchers.loader;
        let query_loader = &batchers.query;
        let query_loader_fat_paginated = &batchers.paginated_query_fat;
        let query_loader_single_relation = &batchers.query_single_relation;

        let ctx_ty = resolver_ctx
            .ty
            .ok_or_else(|| Error::new("Internal Error: Failed process the associated schema."))?;
        let current_ty = ctx_ty.name();

        match self {
            DynamoResolver::ListResultByTypePaginated {
                r#type,
                before,
                after,
                last,
                first,
                nested,
            } => {
                let pk = r#type
                    .expect_string(ctx, last_resolver_value.map(|x| x.data_resolved.borrow()))?;

                // TODO: optimize single edges for the top level
                // TODO: put selected relations
                let relations_selected: IndexMap<&str, &MetaRelation> = IndexMap::new();
                // When we query a Node with the Query Dataloader, we have to indicate
                // which Edges should be getted with it because we are able to retreive
                // a Node with his edges in one network request.
                // We could also request to have only the node edges and not the node
                // data.
                //
                // We add the Node to the edges to also ask for the Node Data.
                let edges: Vec<String> = relations_selected
                    .iter()
                    // we won't be able to load any `ToMany` relations with the original item
                    // since they need to be paginated
                    .filter(|relation| {
                        relation.1.kind == MetaRelationKind::ManyToOne
                            || relation.1.kind == MetaRelationKind::OneToOne
                    })
                    .map(|(_, x)| x)
                    .fold(Vec::new(), |mut acc, cur| {
                        acc.push(cur.name.clone());
                        acc
                    })
                    .into_iter()
                    .unique()
                    .collect();

                let first = first.expect_opt_int(
                    ctx,
                    last_resolver_value.map(|x| x.data_resolved.borrow()),
                    Some(PAGINATION_LIMIT),
                )?;
                let after = after.expect_opt_cursor(
                    ctx,
                    last_resolver_value.map(|x| x.data_resolved.borrow()),
                )?;
                let before = before.expect_opt_cursor(
                    ctx,
                    last_resolver_value.map(|x| x.data_resolved.borrow()),
                )?;
                let last = last.expect_opt_int(
                    ctx,
                    last_resolver_value.map(|x| x.data_resolved.borrow()),
                    Some(PAGINATION_LIMIT),
                )?;
                let len = edges.len();

                let cursor =
                    PaginatedCursor::from_graphql(first, last, after, before, nested.clone())?;
                let mut pagination = ResolvedPaginationInfo::new(
                    ResolvedPaginationDirection::from_paginated_cursor(&cursor),
                );

                let result = query_loader_fat_paginated
                    .load_one(QueryTypePaginatedKey::new(pk.clone(), edges, cursor))
                    .await?;

                let result = result.ok_or_else(|| {
                    Error::new("Internal Error: Failed to fetch the associated nodes.")
                })?;

                pagination = pagination
                    .with_start(
                        result
                            .values
                            .iter()
                            .next()
                            .map(|(pk, _)| PaginationCursor { pk: pk.to_string() }),
                    )
                    .with_end(
                        result
                            .values
                            .iter()
                            .last()
                            .map(|(pk, _)| PaginationCursor { pk: pk.to_string() }),
                    )
                    .with_more_data(result.last_evaluated_key.is_some());

                let result: Vec<serde_json::Value> = result
                    .values
                    .iter()
                    .map(|(_, query_value)| {
                        let mut value_result: Map<String, serde_json::Value> =
                            query_value.edges.iter().fold(
                                Map::with_capacity(len),
                                |mut acc, (edge_key, dyna_value)| {
                                    let value = serde_json::to_value(dyna_value);

                                    match value {
                                        Ok(value) => {
                                            acc.insert(edge_key.to_string(), value);
                                        }
                                        Err(err) => {
                                            acc.insert(
                                                edge_key.to_string(),
                                                serde_json::Value::Null,
                                            );
                                            ctx.add_error(
                                                Error::new_with_source(err)
                                                    .into_server_error(ctx.item.pos),
                                            );
                                        }
                                    }
                                    acc
                                },
                            );

                        match serde_json::to_value(&query_value.node) {
                            Ok(value) => {
                                value_result.insert(pk.clone(), value);
                            }
                            Err(err) => {
                                value_result.insert(pk.clone(), serde_json::Value::Null);
                                ctx.add_error(
                                    Error::new_with_source(err).into_server_error(ctx.item.pos),
                                );
                            }
                        };

                        serde_json::Value::Object(value_result)
                    })
                    .collect();

                Ok(
                    ResolvedValue::new(Arc::new(serde_json::Value::Array(result)))
                        .with_pagination(pagination),
                )
            }
            DynamoResolver::QueryPKSK { pk, sk } => {
                let pk = match pk
                    .param(ctx, last_resolver_value.map(|x| x.data_resolved.borrow()))?
                    .expect("can't fail")
                {
                    Value::String(inner) => inner,
                    _ => {
                        return Err(Error::new("Internal Error: failed to infer key"));
                    }
                };

                let sk = match sk
                    .param(ctx, last_resolver_value.map(|x| x.data_resolved.borrow()))?
                    .expect("can't fail")
                {
                    Value::String(inner) => inner,
                    _ => {
                        return Err(Error::new("Internal Error: failed to infer key"));
                    }
                };

                let selected: HashSet<&str> = ctx
                    .field()
                    .selection_set()
                    .map(|field| field.name())
                    .collect();

                // TODO: optimize single edges for the top level
                let relations_selected: IndexMap<&str, &MetaRelation> = ctx_ty
                    .relations()
                    .into_iter()
                    .filter(|(key, _val)| selected.contains(key))
                    .collect();

                let relations_len = relations_selected.len();
                if relations_len == 0 {
                    match loader_item.load_one((pk.clone(), sk)).await? {
                        Some(dyna) => {
                            let value = serde_json::to_value(dyna)
                                .map_err(|err| Error::new(err.to_string()))?;
                            return Ok(ResolvedValue::new(Arc::new(serde_json::json!({
                                current_ty: value,
                            }))));
                        }
                        // If we do not have any value inside our fetch, it's not an
                        // error, it's only we didn't found the value.
                        None => {
                            return Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null))
                                .with_early_return());
                        }
                    }
                }

                // When we query a Node with the Query Dataloader, we have to indicate
                // which Edges should be getted with it because we are able to retreive
                // a Node with his edges in one network request.
                // We could also request to have only the node edges and not the node
                // data.
                //
                // We add the Node to the edges to also ask for the Node Data.
                let edges: Vec<String> = relations_selected
                    .iter()
                    // we won't be able to load any `ToMany` relations with the original item
                    // since they need to be paginated
                    .filter(|relation| {
                        relation.1.kind == MetaRelationKind::ManyToOne
                            || relation.1.kind == MetaRelationKind::OneToOne
                    })
                    .map(|(_, x)| x)
                    .fold(Vec::new(), |mut acc, cur| {
                        acc.push(cur.name.clone());
                        acc
                    })
                    .into_iter()
                    .unique()
                    .collect();

                let query_result = query_loader
                    .load_one(QueryKey::new(pk.clone(), edges.clone()))
                    .await?
                    .map(|x| x.values)
                    .ok_or_else(|| {
                        Error::new("Internal Error: Failed to fetch the associated nodes.")
                    })?;

                let len = query_result.len();

                // If we do not have any value inside our fetch, it's not an
                // error, it's only we didn't found the value.
                if len == 0 {
                    return Ok(
                        ResolvedValue::new(Arc::new(serde_json::Value::Null)).with_early_return()
                    );
                }

                let result: Map<String, serde_json::Value> =
                    query_result
                        .into_iter()
                        .fold(Map::with_capacity(len), |mut acc, (_, b)| {
                            acc.insert(
                                current_ty.to_string(),
                                serde_json::to_value(b.node).expect("can't fail"),
                            );

                            for (edge, val) in b.edges {
                                acc.insert(edge, serde_json::to_value(val).expect("can't fail"));
                            }

                            acc
                        });

                Ok(ResolvedValue::new(Arc::new(serde_json::Value::Object(
                    result,
                ))))
            }
            DynamoResolver::QuerySingleRelation {
                parent_pk,
                relation_name,
            } => {
                let query_result = query_loader_single_relation
                    .load_one(QuerySingleRelationKey::new(
                        parent_pk.to_string(),
                        relation_name.to_string(),
                    ))
                    .await?
                    .map(|x| x.values)
                    .ok_or_else(|| {
                        Error::new("Internal Error: Failed to fetch the associated nodes.")
                    })?;

                let len = query_result.len();

                // If we do not have any value inside our fetch, it's not an
                // error, it's only we didn't found the value.
                if len == 0 {
                    return Ok(
                        ResolvedValue::new(Arc::new(serde_json::Value::Null)).with_early_return()
                    );
                }

                let result: Map<String, serde_json::Value> =
                    query_result
                        .into_iter()
                        .fold(Map::with_capacity(len), |mut acc, (_, b)| {
                            acc.insert(
                                current_ty.to_string(),
                                serde_json::to_value(b.node).expect("can't fail"),
                            );

                            for (edge, val) in b.edges {
                                acc.insert(edge, serde_json::to_value(val).expect("can't fail"));
                            }

                            acc
                        });

                Ok(ResolvedValue::new(Arc::new(serde_json::Value::Object(
                    result,
                ))))
            }
            DynamoResolver::QueryBy { by } => {
                let by = match by
                    .param(ctx, last_resolver_value.map(|x| x.data_resolved.borrow()))?
                    .expect("can't fail")
                {
                    Value::Object(inner) => inner,
                    _ => {
                        return Err(Error::new("Internal Error: failed to infer key"));
                    }
                };

                let (key, value) = by.first().expect("must exist");

                let key = key.to_string();

                let by_id = key == "id";

                let (pk, sk) = if by_id {
                    let value: String =
                        dynaql_value::from_value(value.clone()).expect("cannot fail");

                    let pk = value.clone();
                    let sk = value;

                    (pk, sk)
                } else {
                    let pk = ConstraintID::from_owned(
                        current_ty.to_string(),
                        key.clone(),
                        value.clone().into_json().expect("cannot fail"),
                    )
                    .to_string();
                    let sk = pk.clone();
                    (pk, sk)
                };

                // TODO: optimize single edges for the top level
                let relations_selected: IndexMap<&str, &MetaRelation> = ctx_ty.relations();
                let relations_len = relations_selected.len();
                if relations_len == 0 || !by_id {
                    match loader_item.load_one((pk.clone(), sk)).await? {
                        Some(mut dyna) => {
                            if !by_id {
                                // Populate the original SK to get the correct ID
                                // TODO: consider a declarative way to do this
                                dyna.insert(
                                    SK.to_string(),
                                    dyna.get(INVERTED_INDEX_PK).expect("must exist").clone(),
                                );
                            }

                            let value = serde_json::to_value(dyna)
                                .map_err(|err| Error::new(err.to_string()))?;

                            return Ok(ResolvedValue::new(Arc::new(serde_json::json!({
                                current_ty: value,
                            }))));
                        }
                        // Return early if no value was found
                        None => {
                            return Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null))
                                .with_early_return());
                        }
                    }
                }

                // When we query a Node with the QueryBy Dataloader, we have to indicate
                // which edges should be queried with it, since we are able to retreive
                // a node with its edges in one network request.
                // We could also request to receive only the node edges and not the node
                // data.
                //
                // We include the node in the edge list to also ask for the node data.

                // FIXME: currently this is unused for non ID queries
                // but we'll need to differentiate edge querying for
                // constraints once we optimize nested pagination as we don't have the PK at this point
                let edges: Vec<String> = relations_selected
                    .iter()
                    // we won't be able to load any `ToMany` relations with the original item
                    // since they need to be paginated
                    .filter(|relation| {
                        relation.1.kind == MetaRelationKind::ManyToOne
                            || relation.1.kind == MetaRelationKind::OneToOne
                    })
                    .map(|(_, x)| x)
                    .fold(Vec::new(), |mut acc, cur| {
                        acc.push(cur.name.clone());
                        acc
                    })
                    .into_iter()
                    .unique()
                    .collect();

                let query_result = query_loader
                    .load_one(QueryKey::new(pk.clone(), edges))
                    .await?
                    .map(|x| x.values)
                    .ok_or_else(|| {
                        Error::new("Internal Error: Failed to fetch the associated nodes.")
                    })?;

                let len = query_result.len();

                // Return early if no value was found
                if len == 0 {
                    return Ok(
                        ResolvedValue::new(Arc::new(serde_json::Value::Null)).with_early_return()
                    );
                }

                let result: Map<String, serde_json::Value> =
                    query_result
                        .into_iter()
                        .fold(Map::with_capacity(len), |mut acc, (_, b)| {
                            acc.insert(
                                current_ty.to_string(),
                                serde_json::to_value(b.node).expect("can't fail"),
                            );

                            for (edge, val) in b.edges {
                                acc.insert(edge, serde_json::to_value(val).expect("can't fail"));
                            }

                            acc
                        });

                Ok(ResolvedValue::new(Arc::new(serde_json::Value::Object(
                    result,
                ))))
            }
        }
    }
}
