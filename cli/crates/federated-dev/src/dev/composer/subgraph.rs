use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use async_graphql_parser::types::ServiceDocument;
use url::Url;

use crate::dev::admin::Header;

#[derive(Debug, Clone)]
pub(crate) struct Subgraph {
    url: Url,
    headers: Vec<Header>,
    schema: ServiceDocument,
    hash: u64,
}

impl Subgraph {
    pub(crate) fn new(url: Url, headers: Vec<Header>, schema: ServiceDocument) -> Self {
        let hash = Self::hash_schema(&schema);

        Self {
            url,
            headers,
            schema,
            hash,
        }
    }

    pub(crate) fn hash(&self) -> u64 {
        self.hash
    }

    pub(crate) fn url(&self) -> &Url {
        &self.url
    }

    pub(crate) fn schema(&self) -> &ServiceDocument {
        &self.schema
    }

    pub(crate) fn headers(&self) -> &[Header] {
        &self.headers
    }

    pub(crate) fn hash_schema(schema: &ServiceDocument) -> u64 {
        let mut hasher = DefaultHasher::new();
        format!("{schema:?}").hash(&mut hasher);

        hasher.finish()
    }
}
