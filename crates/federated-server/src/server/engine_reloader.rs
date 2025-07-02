mod builder;

pub use builder::GatewayEngineReloaderBuilder;

use std::sync::Arc;

use engine::CachedOperation;
use tokio::sync::mpsc;

use super::gateway::{EngineSender, EngineWatcher, GatewayRuntime, GraphDefinition};

/// Handles graph and config updates by constructing a new engine
pub(super) struct GatewayEngineReloader {
    engine_watcher: EngineWatcher<GatewayRuntime>,
}

pub(crate) type GraphSender = mpsc::Sender<GraphDefinition>;

pub enum Update {
    Graph(GraphDefinition),
    Config(Box<gateway_config::Config>),
}

impl GatewayEngineReloader {
    pub fn builder<'a>() -> GatewayEngineReloaderBuilder<'a> {
        GatewayEngineReloaderBuilder::default()
    }

    pub fn engine_watcher(&self) -> EngineWatcher<GatewayRuntime> {
        self.engine_watcher.clone()
    }
}

fn extract_operations_to_warm(
    config: &gateway_config::Config,
    engine_sender: &EngineSender,
) -> Vec<Arc<CachedOperation>> {
    if !config.operation_caching.enabled || !config.operation_caching.warm_on_reload {
        return vec![];
    }

    let (operations, cache_count) = {
        let cache = &engine_sender.borrow().runtime.operation_cache;

        (cache.values().collect(), cache.entry_count())
    };

    if config.operation_caching.warming_percent >= 100 {
        return operations;
    }

    operations
        .into_iter()
        .take(cache_count * (config.operation_caching.warming_percent as usize / 100))
        .collect()
}
