use std::{borrow::Borrow, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use grafbase_runtime::search::{self, Cursor};

use crate::{registry::variables::VariableResolveDefinition, Context, Error};

use super::{
    dynamo_querying::{DynamoResolver, PAGINATION_LIMIT},
    ResolvedValue, ResolverContext, ResolverTrait,
};

mod search_parser;

pub const SEARCH_RESOLVER_EDGES: &str = "edges";
pub const SEARCH_RESOLVER_EDGE_CURSOR: &str = "#cursor";
pub const SEARCH_RESOLVER_EDGE_SCORE: &str = "#score";
pub const SEARCH_RESOLVER_TOTAL_HITS: &str = "totalHits";

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum QueryResolver {
    Search {
        type_name: String,
        entity_type: String,
        query: VariableResolveDefinition,
        fields: VariableResolveDefinition,
        filter: VariableResolveDefinition,
        first: VariableResolveDefinition,
        last: VariableResolveDefinition,
        after: VariableResolveDefinition,
        before: VariableResolveDefinition,
    },
}

impl QueryResolver {
    pub async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        match self {
            QueryResolver::Search {
                type_name,
                entity_type,
                query,
                fields,
                filter,
                first,
                last,
                after,
                before,
            } => {
                let schema = &ctx
                    .registry()
                    .search_config
                    .indices
                    .get(entity_type)
                    .expect("Search query shouldn't be available without a schema.")
                    .schema;
                let search_engine = ctx.data::<search::SearchEngine>()?;
                let last_val = last_resolver_value.map(|resolved| resolved.data_resolved.borrow());

                let first = first.expect_opt_int(ctx, last_val, Some(PAGINATION_LIMIT))?;
                let last = last.expect_opt_int(ctx, last_val, Some(PAGINATION_LIMIT))?;
                let before: Option<Cursor> = before.resolve(ctx, last_val)?;
                let after: Option<Cursor> = after.resolve(ctx, last_val)?;

                let response = search_engine
                    .search(
                        ctx.data()?,
                        search::Request {
                            query: search::GraphqlQuery {
                                text: query.resolve(ctx, last_val)?,
                                fields: fields.resolve(ctx, last_val)?,
                                filter: match filter.resolve::<Value>(ctx, last_val)? {
                                    Value::Null => None,
                                    value => Some(search_parser::parse_filter(schema, value)?),
                                },
                            },
                            pagination: search_parser::parse_pagination(
                                first, before, last, after,
                            )?,
                            entity_type: entity_type.clone(),
                        },
                    )
                    .await?;

                let resolved_pagination = super::ResolvedPaginationInfo {
                    start_cursor: response.hits.first().map(|hit| hit.cursor.clone()),
                    end_cursor: response.hits.last().map(|hit| hit.cursor.clone()),
                    has_next_page: response.info.has_next_page,
                    has_previous_page: response.info.has_previous_page,
                };

                // TODO: We shouldn't call directly a resolver like that IMHO. But currently,
                // it's the only simple way to pass our custom cursor & score.
                let edges: Vec<serde_json::Value> = {
                    let data_resolved = DynamoResolver::QueryIds {
                        ids: response.hits.iter().map(|hit| hit.id.clone()).collect(),
                        type_name: type_name.to_string(),
                    }
                    .resolve(ctx, resolver_ctx, None)
                    .await?
                    .data_resolved;
                    // We should be the only one having this data, but just in case do a copy
                    // to avoid a panic.
                    match Arc::try_unwrap(data_resolved).unwrap_or_else(|arc| (*arc).clone()) {
                        Value::Array(items) => items
                            .into_iter()
                            .zip(response.hits)
                            .map(|(item, hit)| match item {
                                Value::Object(mut fields) => {
                                    fields.insert(
                                        SEARCH_RESOLVER_EDGE_SCORE.to_string(),
                                        serde_json::to_value(hit.score)?,
                                    );
                                    fields.insert(
                                        SEARCH_RESOLVER_EDGE_CURSOR.to_string(),
                                        serde_json::to_value(hit.cursor)?,
                                    );
                                    Ok(Value::Object(fields))
                                }
                                _ => Err(Error::new("Unexpected data from DynamoDB")),
                            })
                            .collect::<Result<Vec<_>, _>>(),
                        _ => Err(Error::new("Unexpected data from DynamoDB")),
                    }?
                };

                Ok(ResolvedValue::new(Arc::new(json!({
                    SEARCH_RESOLVER_EDGES: edges,
                    SEARCH_RESOLVER_TOTAL_HITS: response.info.total_hits
                })))
                .with_pagination(resolved_pagination))
            }
        }
    }
}
