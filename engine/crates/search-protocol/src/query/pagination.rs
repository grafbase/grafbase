use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, UrlSafe},
    formats::Unpadded,
    serde_as,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pagination {
    Forward { first: u64, after: Option<GraphqlCursor> },
    Backward { last: u64, before: GraphqlCursor },
}

// Should be in some common library instead.
#[serde_as]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphqlCursor(#[serde_as(as = "Base64<UrlSafe, Unpadded>")] Vec<u8>);

impl GraphqlCursor {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl<T: AsRef<[u8]>> From<T> for GraphqlCursor {
    fn from(value: T) -> Self {
        GraphqlCursor(value.as_ref().to_vec())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginatedHits<Id> {
    pub hits: Vec<Hit<Id>>,
    pub info: Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Info {
    pub has_previous_page: bool,
    pub has_next_page: bool,
    pub total_hits: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hit<Id> {
    pub id: Id,
    pub cursor: GraphqlCursor,
    pub score: f32,
}
