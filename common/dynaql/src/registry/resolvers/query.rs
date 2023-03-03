use std::{borrow::Borrow, cmp, sync::Arc};

use grafbase_runtime::search::{SearchEngine, SearchRequest};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{registry::variables::VariableResolveDefinition, Context, Error};

use super::{ResolvedValue, ResolverContext};

pub use grafbase_runtime::search;

pub const MATCHING_RECORDS_ID_KEY: &str = "ids";

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum QueryResolver {
    Search {
        query: VariableResolveDefinition,
        limit: VariableResolveDefinition,
        r#type: String,
        schema: search::Schema,
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
                query,
                limit,
                r#type,
                schema,
            } => {
                let search_engine = ctx.data::<SearchEngine>()?;
                let last_resolver_value =
                    last_resolver_value.map(|resolved| resolved.data_resolved.borrow());
                let matching_records_id: Vec<String> = search_engine
                    .search(
                        ctx.data()?,
                        SearchRequest {
                            raw_query: query.resolve(ctx, last_resolver_value)?,
                            limit: cmp::max(0, limit.resolve::<i64>(ctx, last_resolver_value)?)
                                .unsigned_abs(),
                            database: String::new(),
                            // FIXME: At several places the lowercase for the id & entity_type is
                            // used. A single code path should handle that.
                            entity_type: r#type.to_lowercase().to_string(),
                            schema: schema.clone(),
                        },
                    )
                    .await?
                    .matching_records;
                Ok(ResolvedValue::new(Arc::new(json!({
                    MATCHING_RECORDS_ID_KEY: matching_records_id
                }))))
            }
        }
    }
}
