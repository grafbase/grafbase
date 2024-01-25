use super::ComposeSender;
use crate::{dev::admin::Header, error::Error};
use async_graphql_parser::types::ServiceDocument;
use url::Url;

pub(crate) enum AdminBus {
    DynamicGraph { compose_sender: ComposeSender },
    StatisGraph,
}

impl AdminBus {
    pub fn new_dynamic(compose_sender: ComposeSender) -> Self {
        Self::DynamicGraph { compose_sender }
    }

    pub fn new_static() -> Self {
        Self::StatisGraph
    }

    pub async fn compose_graph(
        &self,
        name: String,
        url: Url,
        headers: Vec<Header>,
        schema: ServiceDocument,
    ) -> Result<(), Error> {
        match self {
            AdminBus::DynamicGraph { compose_sender } => {
                super::compose_graph(compose_sender, name, url, headers, schema).await
            }
            AdminBus::StatisGraph => Err(Error::internal("Cannot compose a new subgraph with a schema file.")),
        }
    }

    pub async fn introspect_schema(
        &self,
        name: &str,
        url: Url,
        headers: Vec<Header>,
    ) -> Result<ServiceDocument, Error> {
        match self {
            AdminBus::DynamicGraph { compose_sender } => {
                super::introspect_schema(compose_sender, name, url, headers).await
            }
            AdminBus::StatisGraph => Err(Error::internal("Nothing to introspect")),
        }
    }
}
