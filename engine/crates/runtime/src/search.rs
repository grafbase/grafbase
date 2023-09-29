use std::{ops::Deref, sync::Arc};

pub use search_protocol::{
    config,
    config::*,
    query::{graphql::*, GraphqlCursor, Hit, Info, PaginatedHits, Pagination, QueryError, ScalarValue},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub query: GraphqlQuery,
    pub pagination: Pagination,
    pub index: String,
}

pub type Response = Result<PaginatedHits<String>, QueryError>;

#[async_trait::async_trait]
pub trait SearchEngineInner {
    async fn query(&self, ctx: &crate::Context, request: Request) -> Response;
}

type BoxedSearchEngineImpl = Box<dyn SearchEngineInner + Send + Sync>;

pub struct SearchEngine {
    inner: Arc<BoxedSearchEngineImpl>,
}

impl SearchEngine {
    pub fn new(engine: BoxedSearchEngineImpl) -> SearchEngine {
        SearchEngine {
            inner: Arc::new(engine),
        }
    }
}

impl Deref for SearchEngine {
    type Target = BoxedSearchEngineImpl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
