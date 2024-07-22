use regex::Regex;

use super::{HeaderId, StringId};

/// A header name can be provided either as a regex or as a static name.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum NameOrPattern {
    /// A regex pattern matching multiple headers.
    #[serde(with = "serde_regex", rename = "pattern")]
    Pattern(Regex),
    /// A static single name.
    #[serde(rename = "name")]
    Name(StringId),
}

impl From<StringId> for NameOrPattern {
    fn from(value: StringId) -> Self {
        Self::Name(value)
    }
}

/// Defines a header rule, executed in order before anything else in the engine.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
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
    /// Duplicate the header with a new name.
    #[serde(rename = "rename_duplicate")]
    RenameDuplicate(HeaderRenameDuplicate),
}

/// Header forwarding rules.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct HeaderForward {
    /// Name or pattern of the header to be forwarded.
    #[serde(flatten)]
    pub name: NameOrPattern,
    /// If header is not present, insert this value.
    pub default: Option<StringId>,
    /// Use this name instead of the original when forwarding.
    pub rename: Option<StringId>,
}

/// Header insertion rules.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct HeaderInsert {
    /// The name of the header.
    pub name: StringId,
    /// The value of the header.
    pub value: StringId,
}

/// Header removal rules
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct HeaderRemove {
    /// Removes the header with a static name or matching a regex pattern.
    #[serde(flatten)]
    pub name: NameOrPattern,
}

/// Header forwarding rules.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct HeaderRenameDuplicate {
    /// Name or pattern of the header to be duplicated.
    pub name: StringId,
    /// If header is not present, insert this value.
    pub default: Option<StringId>,
    /// Use this name for the copy.
    pub rename: StringId,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct HeaderRuleId(pub usize);

impl From<HeaderId> for HeaderRuleId {
    fn from(value: HeaderId) -> Self {
        Self(value.0)
    }
}

impl std::ops::Index<HeaderRuleId> for super::Config {
    type Output = HeaderRule;

    fn index(&self, index: HeaderRuleId) -> &Self::Output {
        &self.header_rules[index.0]
    }
}
