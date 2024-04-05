use async_graphql::InputObject;
use url::Url;

#[derive(InputObject, Clone, Debug)]
pub(crate) struct Header {
    pub key: String,
    pub value: String,
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
