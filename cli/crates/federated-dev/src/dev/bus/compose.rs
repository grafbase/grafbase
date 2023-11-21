use super::{ComposeMessage, ComposeReceiver, ComposeSender, GraphSender, RefreshMessage, RefreshSender};
use crate::error::Error;
use graphql_composition::FederatedGraph;

pub(crate) struct ComposeBus {
    graph_sender: GraphSender,
    refresh_sender: RefreshSender,
    compose_sender: ComposeSender,
    compose_receiver: ComposeReceiver,
}

impl ComposeBus {
    pub fn new(
        graph_sender: GraphSender,
        refresh_sender: RefreshSender,
        compose_sender: ComposeSender,
        compose_receiver: ComposeReceiver,
    ) -> Self {
        Self {
            graph_sender,
            refresh_sender,
            compose_sender,
            compose_receiver,
        }
    }

    pub async fn recv(&mut self) -> Option<ComposeMessage> {
        self.compose_receiver.recv().await
    }

    pub async fn send_composer(&self, message: impl Into<ComposeMessage>) -> Result<(), Error> {
        Ok(self.compose_sender.send(message.into()).await?)
    }

    pub async fn send_graph(&self, message: FederatedGraph) -> Result<(), Error> {
        Ok(self.graph_sender.send(Some(message)).await?)
    }

    pub async fn clear_graph(&self) -> Result<(), Error> {
        Ok(self.graph_sender.send(None).await?)
    }

    pub async fn send_refresh(&self, graphs: Vec<RefreshMessage>) -> Result<(), Error> {
        Ok(self.refresh_sender.send(graphs).await?)
    }
}
