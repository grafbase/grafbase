mod complexity_control;
mod response_extensions;
mod retry;
mod trusted_documents;

pub use complexity_control::*;
pub use response_extensions::*;
pub use retry::*;
pub use trusted_documents::*;

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartialConfig {
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
