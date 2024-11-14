use std::str::FromStr;

use ascii::AsciiString;
use serde::Deserialize;
use serde_dynamic_string::DynamicString;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ResponseExtensionExporterConfig {
    /// Whether the traceId is exposed in the grafbase response extension. Defaults to true.
    pub trace_id: bool,
    /// Defines under which conditions the grafbase response extension will be added.
    /// Defaults to a simple header rule, the presence of `x-grafbase-telemetry` is enough.
    pub access_control: Vec<AccessControl>,
}

impl Default for ResponseExtensionExporterConfig {
    fn default() -> Self {
        Self {
            trace_id: true,
            access_control: vec![AccessControl::Header(HeaderAccessControl {
                name: DynamicString::from(AsciiString::from_str("x-grafbase-telemetry").unwrap()),
                value: None,
            })],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "rule")]
pub enum AccessControl {
    #[serde(rename = "header")]
    Header(HeaderAccessControl),
    #[serde(rename = "deny")]
    Deny,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeaderAccessControl {
    /// Name of the header that must be present.
    pub name: DynamicString<AsciiString>,
    /// Expected value of the header. If not provided any value will be accepted.
    pub value: Option<DynamicString<AsciiString>>,
}
