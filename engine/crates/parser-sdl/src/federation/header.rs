use std::cmp::Ordering;

use regex::Regex;

#[derive(Clone, Debug)]
pub enum NameOrPattern {
    Pattern(Regex),
    Name(String),
}

impl PartialEq for NameOrPattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NameOrPattern::Pattern(left), NameOrPattern::Pattern(right)) => left.as_str().eq(right.as_str()),
            (NameOrPattern::Pattern(_), NameOrPattern::Name(_)) => false,
            (NameOrPattern::Name(_), NameOrPattern::Pattern(_)) => false,
            (NameOrPattern::Name(left), NameOrPattern::Name(right)) => left.eq(right),
        }
    }
}

impl Eq for NameOrPattern {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for NameOrPattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (NameOrPattern::Pattern(left), NameOrPattern::Pattern(right)) => left.as_str().partial_cmp(right.as_str()),
            (NameOrPattern::Pattern(_), NameOrPattern::Name(_)) => Some(Ordering::Less),
            (NameOrPattern::Name(_), NameOrPattern::Pattern(_)) => Some(Ordering::Greater),
            (NameOrPattern::Name(left), NameOrPattern::Name(right)) => left.partial_cmp(right),
        }
    }
}

impl Ord for NameOrPattern {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (NameOrPattern::Pattern(left), NameOrPattern::Pattern(right)) => left.as_str().cmp(right.as_str()),
            (NameOrPattern::Pattern(_), NameOrPattern::Name(_)) => Ordering::Less,
            (NameOrPattern::Name(_), NameOrPattern::Pattern(_)) => Ordering::Greater,
            (NameOrPattern::Name(left), NameOrPattern::Name(right)) => left.cmp(right),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubgraphHeaderRule {
    /// Forward the header to the subgraphs.
    Forward(SubgraphHeaderForward),
    /// Insert a new static header.
    Insert(SubgraphHeaderInsert),
    /// Remove the header.
    Remove(SubgraphHeaderRemove),
    /// Duplicate the header with a new name.
    RenameDuplicate(SubgraphRenameDuplicate),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubgraphHeaderForward {
    /// Name or pattern of the header to be forwarded.
    pub name: NameOrPattern,
    /// If header is not present, insert this value.
    pub default: Option<String>,
    /// Use this name instead of the original when forwarding.
    pub rename: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubgraphHeaderInsert {
    /// The name of the header.
    pub name: String,
    /// The value of the header.
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubgraphHeaderRemove {
    /// Removes the header with a static name or matching a regex pattern.
    pub name: NameOrPattern,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubgraphRenameDuplicate {
    /// Name or pattern of the header to be forwarded.
    pub name: String,
    /// If header is not present, insert this value.
    pub default: Option<String>,
    /// Use this name instead of the original when forwarding.
    pub rename: String,
}

impl From<gateway_config::NameOrPattern> for NameOrPattern {
    fn from(value: gateway_config::NameOrPattern) -> Self {
        match value {
            gateway_config::NameOrPattern::Pattern(regex) => NameOrPattern::Pattern(regex),
            gateway_config::NameOrPattern::Name(name) => NameOrPattern::Name(name.to_string()),
        }
    }
}

impl From<gateway_config::HeaderRule> for SubgraphHeaderRule {
    fn from(value: gateway_config::HeaderRule) -> Self {
        match value {
            gateway_config::HeaderRule::Forward(fwd) => Self::Forward(fwd.into()),
            gateway_config::HeaderRule::Insert(insert) => Self::Insert(insert.into()),
            gateway_config::HeaderRule::Remove(remove) => Self::Remove(remove.into()),
            gateway_config::HeaderRule::RenameDuplicate(rename) => Self::RenameDuplicate(rename.into()),
        }
    }
}

impl From<gateway_config::RenameDuplicate> for SubgraphRenameDuplicate {
    fn from(value: gateway_config::RenameDuplicate) -> Self {
        Self {
            name: value.name.to_string(),
            default: value.default.as_ref().map(ToString::to_string),
            rename: value.rename.to_string(),
        }
    }
}

impl From<gateway_config::HeaderForward> for SubgraphHeaderForward {
    fn from(value: gateway_config::HeaderForward) -> Self {
        Self {
            name: value.name.into(),
            default: value.default.as_ref().map(ToString::to_string),
            rename: value.rename.as_ref().map(ToString::to_string),
        }
    }
}

impl From<gateway_config::HeaderInsert> for SubgraphHeaderInsert {
    fn from(value: gateway_config::HeaderInsert) -> Self {
        Self {
            name: value.name.to_string(),
            value: value.value.to_string(),
        }
    }
}

impl From<gateway_config::HeaderRemove> for SubgraphHeaderRemove {
    fn from(value: gateway_config::HeaderRemove) -> Self {
        Self {
            name: value.name.into(),
        }
    }
}
