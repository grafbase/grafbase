pub use gateway_config::LogLevel;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrustedDocumentsConfig {
    pub document_id_unknown_log_level: LogLevel,
    pub document_id_and_query_mismatch_log_level: LogLevel,
    pub inline_document_unknown_log_level: LogLevel,
}

impl Default for TrustedDocumentsConfig {
    fn default() -> Self {
        gateway_config::TrustedDocumentsConfig::default().into()
    }
}

impl From<gateway_config::TrustedDocumentsConfig> for TrustedDocumentsConfig {
    fn from(
        gateway_config::TrustedDocumentsConfig {
            document_id_unknown_log_level,
            document_id_and_query_mismatch_log_level,
            inline_document_unknown_log_level,
            ..
        }: gateway_config::TrustedDocumentsConfig,
    ) -> Self {
        TrustedDocumentsConfig {
            document_id_unknown_log_level,
            document_id_and_query_mismatch_log_level,
            inline_document_unknown_log_level,
        }
    }
}
