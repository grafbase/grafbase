use std::{borrow::Borrow, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use grafbase_runtime::search;

use crate::{registry::variables::VariableResolveDefinition, Context, Error};

use super::{
    dynamo_querying::PAGINATION_LIMIT, ResolvedPaginationDirection, ResolvedValue, ResolverContext,
};

mod search_parser;

pub const SEARCH_RESOLVER_HIT_IDS: &str = "ids";
pub const SEARCH_RESOLVER_TOTAL_HITS: &str = "totalHits";

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum QueryResolver {
    Search {
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
        _resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        match self {
            QueryResolver::Search {
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
                let before: Option<String> = before.resolve(ctx, last_val)?;
                let after: Option<String> = after.resolve(ctx, last_val)?;
                let pagination = search_parser::parse_pagination(first, before, last, after)?;
                let direction = match pagination {
                    search::Pagination::Forward { .. } => ResolvedPaginationDirection::Forward,
                    search::Pagination::Backward { .. } => ResolvedPaginationDirection::Backward,
                };

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
                            pagination,
                            // TODO: Where should database be defined? In the ExecutionContext
                            // available for everything in grafbase-runtime or specific to search?
                            database: String::new(),
                            entity_type: entity_type.clone(),
                        },
                    )
                    .await?;

                let resolved_pagination = super::ResolvedPaginationInfo {
                    direction,
                    // TODO: Add Cursor & has_previous/has_next_page
                    end_cursor: None,
                    start_cursor: None,
                    more_data: false,
                };
                Ok(ResolvedValue::new(Arc::new(json!({
                    SEARCH_RESOLVER_HIT_IDS: response.hits.into_iter().map(|hit| hit.id).collect::<Vec<_>>(),
                    SEARCH_RESOLVER_TOTAL_HITS: response.total_hits
                })))
                .with_pagination(resolved_pagination))
            }
        }
    }
}
