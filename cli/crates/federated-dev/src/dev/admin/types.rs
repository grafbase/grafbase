use async_graphql::{InputObject, SimpleObject};
use url::Url;

#[derive(InputObject, Clone, Debug)]
pub(crate) struct Header {
    key: String,
    value: String,
}

impl Header {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(InputObject)]
pub(crate) struct PublishSubgraphInput {
    pub(crate) name: String,
    pub(crate) url: Url,
    pub(crate) headers: Vec<Header>,
}

#[derive(SimpleObject)]
pub(crate) struct PublishSubgraphSuccess {
    __typename: &'static str,
}

impl Default for PublishSubgraphSuccess {
    fn default() -> Self {
        Self {
            __typename: "PublishSubgraphSuccess",
        }
    }
}
