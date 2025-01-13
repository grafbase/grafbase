use ascii::AsciiString;
use serde_dynamic_string::DynamicString;

use crate::LogLevel;

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct TrustedDocumentsConfig {
    /// If true, the engine will resolve trusted documents ids in queries, with or without `query` key in the request. Default: false.
    pub enabled: bool,
    /// Accept only trusted document queries. These can include a string in query, the extension with the doc id (relay or apollo), or both. Default: false.
    pub enforced: bool,
    /// See [BypassHeader]
    #[serde(flatten)]
    pub bypass_header: BypassHeader,
    /// The log level to emit logs when a request contains a trusted document id, but the trusted document is not found in GDN. Default: INFO.
    pub document_id_unknown_log_level: LogLevel,
    /// The log level to emit logs when a request contains a trusted document id and an inline document in `query`, but the trusted document body does not match the inline document. Default: INFO.
    pub document_id_and_query_mismatch_log_level: LogLevel,
    /// The log level to emit logs when a request contains only an inline document but it does not correspond to any trusted document. Default: INFO.
    pub inline_document_unknown_log_level: LogLevel,
}

impl Default for TrustedDocumentsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            enforced: false,
            bypass_header: Default::default(),
            document_id_unknown_log_level: LogLevel::Info,
            document_id_and_query_mismatch_log_level: LogLevel::Info,
            inline_document_unknown_log_level: LogLevel::Info,
        }
    }
}

/// An optional header that can be passed by clients to bypass trusted documents enforcement, allowing arbitrary queries.
#[derive(Debug, serde::Deserialize, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BypassHeader {
    /// Name of the optional header that can be set to bypass trusted documents enforcement, when `enabled = true`. Only meaningful in combination with `bypass_header_value`.
    pub bypass_header_name: Option<AsciiString>,
    /// Value of the optional header that can be set to bypass trusted documents enforcement, when `enabled = true`. Only meaningful in combination with `bypass_header_value`.
    pub bypass_header_value: Option<DynamicString<String>>,
}
