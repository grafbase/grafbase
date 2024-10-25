use url::Url;

use super::{
    admin::Header,
    bus::{RefreshBus, RemoveSubgraph},
    composer::Subgraph,
};

pub(crate) struct RefreshMessage {
    pub(crate) name: String,
    pub(crate) url: Url,
    pub(crate) headers: Vec<Header>,
    pub(crate) hash: u64,
}

pub(crate) struct Refresher {
    bus: RefreshBus,
}

impl Refresher {
    pub(crate) fn new(bus: RefreshBus) -> Self {
        Self { bus }
    }

    pub(crate) async fn handler(mut self) -> Result<(), crate::Error> {
        log::trace!("starting the refresher handler");

        while let Some(graphs) = self.bus.recv().await {
            for message in graphs {
                let schema = match self
                    .bus
                    .introspect_schema(&message.name, message.url.clone(), message.headers.clone())
                    .await
                {
                    Ok(schema) if Subgraph::hash_schema(&schema) != message.hash => schema,
                    Ok(_) => continue,
                    Err(e) => {
                        log::error!("error in introspection: {e}");
                        self.bus.send_composer(RemoveSubgraph::new(message.name)).await?;
                        continue;
                    }
                };

                log::trace!("subgraph changed, composing a new federated graph");

                if let Err(e) = self
                    .bus
                    .compose_graph(message.name, message.url, message.headers, schema)
                    .await
                {
                    log::error!("error in composition: {e}");
                }
            }
        }

        Ok(())
    }
}
