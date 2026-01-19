use std::time::Duration;

use hive_console_sdk::supergraph_fetcher::{SupergraphFetcher, async_fetcher::SupergraphFetcherAsyncState};

use tokio::sync::mpsc;

use crate::{events::UpdateEvent, graph::Graph};

pub struct HiveConsoleUpdater {
    fetcher: SupergraphFetcher<SupergraphFetcherAsyncState>,
    poll_interval: Duration,
    sender: mpsc::Sender<UpdateEvent>,
}

impl HiveConsoleUpdater {
    pub fn new(
        endpoints: Vec<String>,
        key: Option<String>,
        poll_interval: Duration,
        sender: mpsc::Sender<UpdateEvent>,
    ) -> crate::Result<Self> {
        let mut builder = SupergraphFetcher::builder().user_agent("grafbase-gateway".to_string());
        for endpoint in endpoints {
            builder = builder.add_endpoint(endpoint);
        }
        if let Some(key) = key {
            builder = builder.key(key);
        }
        let fetcher = builder
            .build_async()
            .map_err(|e| crate::Error::InternalError(format!("Failed to create SupergraphFetcher: {}", e)))?;
        Ok(HiveConsoleUpdater {
            fetcher,
            poll_interval,
            sender,
        })
    }
    pub async fn poll(&self) {
        let mut interval = tokio::time::interval(self.poll_interval);

        loop {
            match self.fetcher.fetch_supergraph().await {
                Ok(Some(sdl)) => {
                    if self
                        .sender
                        .send(UpdateEvent::Graph(Graph::FromText { sdl }))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(None) => {
                    // No update available
                }
                Err(e) => {
                    tracing::error!("Error fetching supergraph SDL: {}", e);
                }
            };

            interval.tick().await;
        }
    }
}
