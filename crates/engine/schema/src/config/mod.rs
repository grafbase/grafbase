mod auth;
mod complexity_control;
mod response_extensions;
mod retry;
mod trusted_documents;

use id_newtypes::IdRange;

use crate::HeaderRuleId;
pub use auth::*;
pub use complexity_control::*;
pub use response_extensions::*;
pub use retry::*;
pub use trusted_documents::*;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct PartialConfig {
    pub(crate) default_header_rules: IdRange<HeaderRuleId>,

    pub timeout: std::time::Duration,
    pub auth_config: Option<AuthConfig>,
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
}
