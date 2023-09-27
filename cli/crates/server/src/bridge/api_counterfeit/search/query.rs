use std::ops::{Bound, Not};

use serde::{Deserialize, Serialize};

use runtime::search::ScalarValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Query {
    Intersection(Vec<Query>),
    Union(Vec<Query>),
    Not(Box<Query>),
    Range { field: String, range: Range<ScalarValue> },
    In { field: String, values: Vec<ScalarValue> },
    Regex { field: String, pattern: String },
    All,
    Empty,
    IsNull { field: String },
    Text { value: String, fields: Option<Vec<String>> },
}

impl Not for Query {
    type Output = Query;

    fn not(self) -> Self::Output {
        Query::Not(Box::new(self))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range<T> {
    pub start: Bound<T>,
    pub end: Bound<T>,
}

impl<T: PartialEq> PartialEq for Range<T> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl<T> Range<T> {
    pub fn unbounded() -> Self {
        Self::default()
    }

    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> Range<U> {
        Range {
            start: map_bound(self.start, &f),
            end: map_bound(self.end, &f),
        }
    }
}

impl<T> Default for Range<T> {
    fn default() -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
        }
    }
}

fn map_bound<T, U, F: FnOnce(T) -> U>(bound: Bound<T>, f: F) -> Bound<U> {
    use Bound::{Excluded, Included, Unbounded};
    match bound {
        Unbounded => Unbounded,
        Included(x) => Included(f(x)),
        Excluded(x) => Excluded(f(x)),
    }
}
