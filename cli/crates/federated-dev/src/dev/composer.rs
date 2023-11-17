mod subgraph;

pub(crate) use self::subgraph::Subgraph;

use super::{
    bus::{ComposeBus, ComposeMessage, ComposeSchema, IntrospectSchema, RemoveSubgraph},
    refresher::RefreshMessage,
};
use crate::error::Error;
use async_graphql_parser::parse_schema;
use grafbase_graphql_introspection::introspect;
use graphql_composition::{compose, Subgraphs};
use std::collections::BTreeMap;

pub(crate) struct Composer {
    bus: ComposeBus,
    graphs: BTreeMap<String, Subgraph>,
}

impl Composer {
    pub(crate) fn new(bus: ComposeBus) -> Self {
        Self {
            bus,
            graphs: BTreeMap::default(),
        }
    }

    pub(crate) async fn handler(mut self) {
        log::trace!("starting the composer handler");

        loop {
            let result = match self.bus.recv().await {
                Some(ComposeMessage::Introspect(message)) => {
                    log::trace!("composer handling introspection for subgraph '{}'", message.name());
                    self.handle_introspect(message).await
                }
                Some(ComposeMessage::Compose(message)) => {
                    log::trace!("composer handling composition for subgraph '{}'", message.name());
                    self.handle_compose(message).await
                }
                Some(ComposeMessage::RemoveSubgraph(message)) => {
                    log::trace!("composer handling removing a subgraph '{}'", message.name());
                    self.handle_remove_subgraph(message).await
                }
                Some(ComposeMessage::Recompose) => {
                    log::trace!("composer handling recomposition");
                    self.handle_recompose().await
                }
                Some(ComposeMessage::InitializeRefresh) => {
                    log::trace!("composer initializing a refresh");
                    self.handle_init_refresh().await
                }
                None => break,
            };

            if let Err(error) = result {
                log::warn!("Error in composer: {error:?}");
            }
        }
    }

    async fn handle_introspect(&mut self, message: IntrospectSchema) -> Result<(), crate::Error> {
        let (name, url, headers, responder) = message.into_parts();

        let headers = headers
            .iter()
            .map(|header| (header.key(), header.value()))
            .collect::<Vec<_>>();

        let result = introspect(url.as_str(), headers.as_slice())
            .await
            .and_then(|sdl| parse_schema(sdl).map_err(|error| error.to_string()));

        match result {
            Ok(schema) => {
                responder
                    .send(Ok(schema))
                    .map_err(|_| Error::internal("oneshot channel dead"))?;
            }
            Err(error) => {
                let error = Error::introspection(error.to_string());

                responder
                    .send(Err(error))
                    .map_err(|_| Error::internal("oneshot channel dead"))?;

                self.bus.send_composer(RemoveSubgraph::new(&name)).await?;
            }
        }

        Ok(())
    }

    fn ingest_subgraphs(&self, add_new: Option<(&str, &Subgraph)>) -> Subgraphs {
        let mut subgraphs = Subgraphs::default();

        for (name, subgraph) in &self.graphs {
            if add_new.is_some_and(|(added_name, _)| added_name == name) {
                continue;
            }

            subgraphs.ingest(subgraph.schema(), name, subgraph.url().as_str());
        }

        if let Some((name, subgraph)) = add_new {
            subgraphs.ingest(subgraph.schema(), name, subgraph.url().as_str());
        }

        subgraphs
    }

    async fn handle_compose(&mut self, message: ComposeSchema) -> Result<(), crate::Error> {
        let subgraphs = self.ingest_subgraphs(Some(message.parts()));
        let (name, subgraph, responder) = message.into_parts();

        let graph = match compose(&subgraphs).into_result() {
            Ok(graph) => graph,
            Err(error) => {
                responder
                    .send(Err(Error::composition(&error)))
                    .map_err(|_| Error::internal("compose channel is dead"))?;

                self.bus.send_composer(RemoveSubgraph::new(&name)).await?;

                return Ok(());
            }
        };

        self.graphs.insert(name, subgraph);
        self.bus.send_graph(graph).await?;

        responder
            .send(Ok(()))
            .map_err(|_| Error::internal("compose channel is dead"))?;

        Ok(())
    }

    async fn handle_remove_subgraph(&mut self, message: RemoveSubgraph) -> Result<(), crate::Error> {
        if self.graphs.remove(message.name()).is_some() {
            self.bus.send_composer(ComposeMessage::Recompose).await?;
        }

        Ok(())
    }

    async fn handle_recompose(&mut self) -> Result<(), crate::Error> {
        if self.graphs.is_empty() {
            // Composing an empty set of graphs is going to fail, so lets not do that.
            self.bus.clear_graph().await?;
            return Ok(());
        }

        let subgraphs = self.ingest_subgraphs(None);

        match compose(&subgraphs).into_result() {
            Ok(graph) => self.bus.send_graph(graph).await?,
            Err(error) => {
                log::warn!("Recomposition failed: {error:?}");
                return Err(crate::Error::internal(
                    "Fatal: couldn't recompose existing subgraphs".to_string(),
                ));
            }
        };

        Ok(())
    }

    async fn handle_init_refresh(&mut self) -> Result<(), crate::Error> {
        let graphs = self
            .graphs
            .iter()
            .map(|(name, subgraph)| RefreshMessage {
                name: name.clone(),
                url: subgraph.url().clone(),
                headers: subgraph.headers().to_vec(),
                hash: subgraph.hash(),
            })
            .collect();

        self.bus.send_refresh(graphs).await?;

        Ok(())
    }
}
