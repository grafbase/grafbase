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
