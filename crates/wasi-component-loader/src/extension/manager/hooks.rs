use std::sync::Arc;

use engine_schema::Schema;
use enumflags2::BitFlag;
use extension_catalog::{Extension, ExtensionId};
use gateway_config::Config;
use strum::IntoDiscriminant;

use super::{ExtensionConfig, ExtensionLoader, Pool, PoolConfig, WasmConfig};

#[derive(Default, Clone)]
pub struct WasmHooks(Arc<WasiHooksInner>);

#[derive(Default)]
struct WasiHooksInner {
    pool: Option<Pool>,
    event_filter: Option<event_queue::EventFilter>,
}

impl WasmHooks {
    pub async fn new(gateway_config: &Config, extension: Option<Extension>) -> crate::Result<Self> {
        let Some(extension) = extension else {
            return Ok(Default::default());
        };

        let event_filter = extension.manifest.event_filter.as_ref().map(convert_event_filter);

        let mut selected_config = None;

        for (name, config) in gateway_config.extensions.iter() {
            if name != extension.manifest.name() {
                continue;
            }

            selected_config = Some(config);
        }

        let extension_config = ExtensionConfig {
            id: ExtensionId::from(0_u16),
            manifest_id: extension.manifest.id.clone(),
            r#type: extension.manifest.r#type.discriminant(),
            sdk_version: extension.manifest.sdk_version.clone(),
            pool: PoolConfig {
                max_size: selected_config.and_then(|c| c.max_pool_size()),
            },
            wasm: WasmConfig {
                location: extension.wasm_path.clone(),
                networking: extension.manifest.network_enabled(),
                stdout: extension.manifest.stdout_enabled(),
                stderr: extension.manifest.stderr_enabled(),
                environment_variables: extension.manifest.environment_variables_enabled(),
            },
            guest_config: selected_config.and_then(|c| c.config().cloned()),
            // FIXME: Rely on hook configuration to define whether events can be skipped or not
            can_skip_sending_events: false,
        };

        let max_pool_size = extension_config.pool.max_size;
        let schema = Schema::empty().await;
        let loader = ExtensionLoader::new(Arc::new(schema), extension_config)?;
        let pool = Pool::new(loader, max_pool_size);

        Ok(Self(Arc::new(WasiHooksInner {
            pool: Some(pool),
            event_filter,
        })))
    }

    pub(crate) fn pool(&self) -> Option<&Pool> {
        self.0.pool.as_ref()
    }

    pub(crate) fn event_filter(&self) -> Option<event_queue::EventFilter> {
        self.0.event_filter
    }
}

fn convert_event_filter(filter: &extension_catalog::EventFilter) -> event_queue::EventFilter {
    match filter {
        extension_catalog::EventFilter::All => event_queue::EventFilter::All,
        extension_catalog::EventFilter::Types(types) => {
            let mut out = event_queue::EventFilterType::empty();
            for ty in types {
                out.insert(match ty {
                    extension_catalog::EventType::Operation => event_queue::EventFilterType::Operation,
                    extension_catalog::EventType::SubgraphRequest => event_queue::EventFilterType::SubgraphRequest,
                    extension_catalog::EventType::HttpRequest => event_queue::EventFilterType::HttpRequest,
                    extension_catalog::EventType::Extension => event_queue::EventFilterType::Extension,
                });
            }
            event_queue::EventFilter::Types(out)
        }
    }
}
