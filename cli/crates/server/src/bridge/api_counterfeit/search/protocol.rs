use serde::{Deserialize, Serialize};

use super::{cursor::Cursor, query::Query, SearchError};

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryExecutionRequest {
    pub query: Query,
    pub pagination: Pagination,
    pub entity_type: String,
    pub database: String,
}

pub type QueryExecutionResponse = Result<PaginatedHits<String>, SearchError>;

#[derive(Debug, Serialize, Deserialize)]
pub enum Pagination {
    Forward { first: u64, after: Option<Cursor> },
    Backward { last: u64, before: Cursor },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedHits<Id> {
    pub hits: Vec<Hit<Id>>,
    pub info: Info,
}

impl<T> PaginatedHits<T> {
    pub fn map_id<U, F: Fn(T) -> U>(self, f: F) -> PaginatedHits<U> {
        let PaginatedHits { hits, info } = self;
        PaginatedHits {
            hits: hits
                .into_iter()
                .map(|Hit { id, cursor, score }| Hit {
                    id: f(id),
                    cursor,
                    score,
                })
                .collect(),
            info,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Info {
    pub has_previous_page: bool,
    pub has_next_page: bool,
    pub total_hits: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hit<Id> {
    pub id: Id,
    pub cursor: Cursor,
    pub score: f32,
}
