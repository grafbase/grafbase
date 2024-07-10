use regex::Regex;

use super::StringId;

/// Defines a header rule, executed in order before anything else in the engine.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum Header {
    /// Forward the header to the subgraphs.
    Forward(HeaderForward),
    /// Insert a new static header.
    Insert(HeaderInsert),
    /// Remove the header.
    Remove(HeaderRemove),
}

/// A header name can be provided either as a regex or as a static name.
#[derive(serde::Deserialize, serde::Serialize)]
pub enum NameOrPattern {
    /// A regex pattern matching multiple headers.
    #[serde(with = "serde_regex", rename = "pattern")]
    Pattern(Regex),
    /// A static single name.
    #[serde(rename = "name")]
    Name(StringId),
}

/// Header forwarding rules.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HeaderForward {
    /// Name or pattern of the header to be forwarded.
    #[serde(flatten)]
    pub name: NameOrPattern,
    /// If header is not present, insert this value.
    pub default: Option<AsciiString>,
    /// Use this name instead of the original when forwarding.
    pub rename: Option<AsciiString>,
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct HeaderId(pub usize);

impl std::ops::Index<HeaderId> for super::Config {
    type Output = Header;

    fn index(&self, index: HeaderId) -> &Header {
        &self.headers[index.0]
    }
}
