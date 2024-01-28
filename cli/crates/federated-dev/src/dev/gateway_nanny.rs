use std::sync::Arc;

use crate::ConfigReceiver;

use super::bus::{GatewaySender, GraphReceiver};
use engine_v2::EngineEnv;
use futures_concurrency::stream::Merge;
use futures_util::{stream::BoxStream, StreamExt};
use gateway_v2::{Gateway, GatewayEnv};
use tokio_stream::wrappers::WatchStream;

/// The GatewayNanny looks after the `Gateway` - on updates to the graph or config it'll
/// create a new `Gateway` and publish it on the gateway channel
pub(crate) struct GatewayNanny {
    graph: GraphReceiver,
    config: ConfigReceiver,
    sender: GatewaySender,
}

impl GatewayNanny {
    pub fn new(graph: GraphReceiver, config: ConfigReceiver, sender: GatewaySender) -> Self {
        Self { graph, config, sender }
    }

    pub async fn handler(self) {
        log::trace!("starting the gateway nanny");

        let streams: [BoxStream<'static, NannyMessage>; 2] = [
            Box::pin(WatchStream::new(self.graph.clone()).map(|_| NannyMessage::GraphUpdated)),
            Box::pin(WatchStream::new(self.config.clone()).map(|_| NannyMessage::ConfigUpdated)),
        ];

        let mut stream = streams.merge();

        while let Some(message) = stream.next().await {
            log::trace!("nanny received a {message:?}");
            if let Err(error) = self.sender.send(new_gateway(&self.graph, &self.config).await) {
                log::error!("Couldn't publish new gateway: {error:?}");
            }
        }
    }
}

async fn new_gateway(graph: &GraphReceiver, config: &ConfigReceiver) -> Option<Arc<Gateway>> {
    let graph = graph.borrow().clone()?;

    let config = engine_config_builder::build_config(&config.borrow(), graph);
    Some(Arc::new(Gateway::new(
        config.into_latest().try_into().ok()?,
        EngineEnv {
            fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        },
        GatewayEnv {
            kv: runtime_local::InMemoryKvStore::runtime(),
            cache: runtime_local::InMemoryCache::runtime(runtime::cache::GlobalCacheConfig {
                common_cache_tags: vec![],
                enabled: true,
                subdomain: "localhost".to_string(),
            }),
        },
    )))
}

#[derive(Debug)]
enum NannyMessage {
    GraphUpdated,
    ConfigUpdated,
}
