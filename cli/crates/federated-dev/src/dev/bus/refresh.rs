use super::{ComposeMessage, ComposeSender, RefreshMessage, RefreshReceiver};
use crate::{dev::admin::Header, error::Error};
use async_graphql_parser::types::ServiceDocument;
use url::Url;

pub(crate) struct RefreshBus {
    refresh_receiver: RefreshReceiver,
    compose_sender: ComposeSender,
}

impl RefreshBus {
    pub fn new(refresh_receiver: RefreshReceiver, compose_sender: ComposeSender) -> Self {
        Self {
            refresh_receiver,
            compose_sender,
        }
    }

    pub async fn recv(&mut self) -> Option<Vec<RefreshMessage>> {
        self.refresh_receiver.recv().await
    }

    pub async fn send_composer(&self, message: impl Into<ComposeMessage>) -> Result<(), Error> {
        Ok(self.compose_sender.send(message.into()).await?)
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
}
