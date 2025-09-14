mod complexity_control;
mod response_extensions;
mod retry;
mod trusted_documents;

use std::sync::Arc;

pub use complexity_control::*;
use gateway_config::Config;
pub use response_extensions::*;
pub use retry::*;
pub use trusted_documents::*;

/// For legacy reasons we have this intermediate object.
#[derive(Clone)]
pub struct SchemaConfig {
    pub config: Arc<Config>,
    pub timeout: std::time::Duration,
    pub operation_limits: gateway_config::OperationLimitsConfig,
    pub disable_introspection: bool,
    pub retry: Option<RetryConfig>,
    pub batching: gateway_config::BatchingConfig,
    pub complexity_control: ComplexityControl,
    pub response_extension: ResponseExtensionConfig,
    pub apq_enabled: bool,
    pub executable_document_limit_bytes: usize,
    pub trusted_documents: TrustedDocumentsConfig,
    pub websocket_forward_connection_init_payload: bool,
    pub contract_cache_max_size: usize,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Arc::new(Config::default()).into()
    }
}

impl From<Arc<Config>> for SchemaConfig {
    fn from(config: Arc<Config>) -> Self {
        Self {
            timeout: config.gateway.timeout,
            operation_limits: config.operation_limits.unwrap_or_default(),
            disable_introspection: !config.graph.introspection.unwrap_or_default(),
            retry: config.gateway.retry.enabled.then_some(config.gateway.retry.into()),
            batching: config.gateway.batching.clone(),
            complexity_control: (&config.complexity_control).into(),
            response_extension: config
                .telemetry
                .exporters
                .response_extension
                .clone()
                .unwrap_or_default()
                .into(),
            apq_enabled: config.apq.enabled,
            executable_document_limit_bytes: config
                .executable_document_limit
                .bytes()
                .try_into()
                .expect("executable document limit should not be negative"),
            trusted_documents: config.trusted_documents.clone().into(),
            websocket_forward_connection_init_payload: config.websockets.forward_connection_init_payload,
            contract_cache_max_size: config.graph.contracts.cache.max_size,
            config,
        }
    }
}
