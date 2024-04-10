use super::{ComposeSender, RemoveSubgraph};
use crate::{dev::admin::Header, error::Error};
use async_graphql_parser::types::ServiceDocument;
use url::Url;

#[derive(Clone)]
pub(crate) struct SubgraphConfigWatcherBus {
    compose_sender: ComposeSender,
}

impl SubgraphConfigWatcherBus {
    pub fn new(compose_sender: ComposeSender) -> Self {
        Self { compose_sender }
    }

    pub async fn compose_graph(
        &self,
        name: String,
        url: Url,
        headers: Vec<Header>,
        schema: ServiceDocument,
    ) -> Result<(), Error> {
        super::compose_graph(&self.compose_sender, name, url, headers, schema).await
    }

    pub async fn introspect_schema(
        &self,
        name: &str,
        url: Url,
        headers: Vec<Header>,
    ) -> Result<ServiceDocument, Error> {
        super::introspect_schema(&self.compose_sender, name, url, headers).await
    }

    pub async fn remove_subgraph(&self, name: &str) -> Result<(), Error> {
        self.compose_sender
            .send(super::ComposeMessage::RemoveSubgraph(RemoveSubgraph::new(name)))
            .await
            .map_err(|_| Error::internal("compose channel closed".to_string()))
    }
}
