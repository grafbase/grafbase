use std::fmt;

use internment::ArcIntern;
use serde::{Deserialize, Serialize};

/// A path to the current location in a query
///
/// We use `im::Vector` to store this as we'll be making a ton of these with slight
/// changes, and its structural sharing should make that more efficient with memory.
///
/// At one point this was just a reverse linked-list of references, but that was a
/// real pain to integrate with defer & stream as the lifetimes wouldn't last long
/// enough.
#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct QueryPath(im::Vector<QueryPathSegment>);

impl QueryPath {
    pub fn empty() -> Self {
        QueryPath::default()
    }

    pub fn push(&mut self, segment: impl Into<QueryPathSegment>) {
        self.0.push_back(segment.into());
    }

    pub fn last(&self) -> Option<&QueryPathSegment> {
        self.0.last()
    }

    pub fn child(self, segment: impl Into<QueryPathSegment>) -> Self {
        let mut child = self.clone();
        child.push(segment.into());
        child
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &QueryPathSegment> {
        self.0.iter()
    }
}

impl IntoIterator for QueryPath {
    type Item = QueryPathSegment;

    type IntoIter = im::vector::ConsumingIter<QueryPathSegment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::fmt::Debug for QueryPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl fmt::Display for QueryPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, segment) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ".")?;
            }
            write!(f, "{segment}")?;
        }
        Ok(())
    }
}

/// A segment in the path to the current query.
///
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum QueryPathSegment {
    /// We are currently resolving an element in a list.
    Index(usize),
    // We are currently resolving a field in an object.
    Field(ArcIntern<String>),
}

impl fmt::Display for QueryPathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryPathSegment::Index(idx) => write!(f, "{idx}"),
            QueryPathSegment::Field(name) => write!(f, "{name}"),
        }
    }
}

impl From<usize> for QueryPathSegment {
    fn from(value: usize) -> Self {
        QueryPathSegment::Index(value)
    }
}

impl From<&str> for QueryPathSegment {
    fn from(value: &str) -> Self {
        QueryPathSegment::Field(ArcIntern::from_ref(value))
    }
}
