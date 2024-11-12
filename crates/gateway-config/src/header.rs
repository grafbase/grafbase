use ascii::AsciiString;
use regex::Regex;
use serde::Deserialize;

use serde_dynamic_string::DynamicString;

/// A header name can be provided either as a regex or as a static name.
#[derive(Deserialize, Debug, Clone)]
pub enum NameOrPattern {
    /// A regex pattern matching multiple headers.
    #[serde(with = "serde_regex", rename = "pattern")]
    Pattern(Regex),
    /// A static single name.
    #[serde(rename = "name")]
    Name(DynamicString<AsciiString>),
}

/// Defines a header rule, executed in order before anything else in the engine.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "rule")]
pub enum HeaderRule {
    /// Forward the header to the subgraphs.
    #[serde(rename = "forward")]
    Forward(HeaderForward),
    /// Insert a new static header.
    #[serde(rename = "insert")]
    Insert(HeaderInsert),
    /// Remove the header.
    #[serde(rename = "remove")]
    Remove(HeaderRemove),
    /// Forward the header to the subgraphs together with a renamed copy.
    #[serde(rename = "rename_duplicate")]
    RenameDuplicate(RenameDuplicate),
}

/// Header forwarding rules.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RenameDuplicate {
    /// Name or pattern of the header to be forwarded.
    pub name: DynamicString<AsciiString>,
    /// If header is not present, insert this value.
    pub default: Option<DynamicString<AsciiString>>,
    /// Use this name instead of the original when forwarding.
    pub rename: DynamicString<AsciiString>,
}

/// Header forwarding rules.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HeaderForward {
    /// Name or pattern of the header to be forwarded.
    #[serde(flatten)]
    pub name: NameOrPattern,
    /// If header is not present, insert this value.
    pub default: Option<DynamicString<AsciiString>>,
    /// Use this name instead of the original when forwarding.
    pub rename: Option<DynamicString<AsciiString>>,
}

/// Header insertion rules.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HeaderInsert {
    /// The name of the header.
    pub name: DynamicString<AsciiString>,
    /// The value of the header.
    pub value: DynamicString<AsciiString>,
}

/// Header removal rules
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HeaderRemove {
    /// Removes the header with a static name or matching a regex pattern.
    #[serde(flatten)]
    pub name: NameOrPattern,
}
