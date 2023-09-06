use std::{
    cmp::Ordering,
    ops::{Bound, RangeBounds},
};

use serde::{Deserialize, Serialize};

use super::ScalarValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range<T> {
    pub start: Bound<T>,
    pub end: Bound<T>,
}

impl std::fmt::Display for Range<ScalarValue> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.start {
            Bound::Included(start) => write!(f, "[{start},")?,
            Bound::Excluded(start) => write!(f, "]{start},")?,
            Bound::Unbounded => write!(f, "]..,")?,
        }
        match &self.end {
            Bound::Included(end) => write!(f, "{end}]"),
            Bound::Excluded(end) => write!(f, "{end}["),
            Bound::Unbounded => write!(f, "..["),
        }
    }
}

impl<T: PartialEq> PartialEq for Range<T> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
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

fn map_bound<T, U, F: FnOnce(T) -> U>(bound: Bound<T>, f: F) -> Bound<U> {
    match bound {
        Bound::Unbounded => Bound::Unbounded,
        Bound::Included(x) => Bound::Included(f(x)),
        Bound::Excluded(x) => Bound::Excluded(f(x)),
    }
}

impl Range<ScalarValue> {
    pub fn of<T, R>(range: R) -> Self
    where
        T: Clone + Into<ScalarValue>,
        R: RangeBounds<T>,
    {
        let clone_into = |e: &T| e.clone().into();
        Range {
            start: map_bound(range.start_bound(), clone_into),
            end: map_bound(range.end_bound(), clone_into),
        }
    }

    pub fn is_empty(&self) -> bool {
        use Bound::{Excluded, Included};
        match (&self.start, &self.end) {
            (Included(s), Included(e)) => matches!(
                e.partial_cmp(s).expect("We build Range ourselves at this point."),
                Ordering::Less
            ),
            (Included(s) | Excluded(s), Excluded(e)) | (Excluded(s), Included(e)) => {
                matches!(
                    e.partial_cmp(s).expect("We build Range ourselves at this point."),
                    Ordering::Less | Ordering::Equal
                )
            }
            _ => false,
        }
    }

    pub fn intersection(a: &Range<ScalarValue>, b: &Range<ScalarValue>) -> Option<Range<ScalarValue>> {
        Some(Range {
            start: highest_start(a.start.clone(), b.start.clone())?,
            end: lowest_end(a.end.clone(), b.end.clone())?,
        })
    }
}

fn highest_start(a: Bound<ScalarValue>, b: Bound<ScalarValue>) -> Option<Bound<ScalarValue>> {
    use Bound::{Excluded, Included};
    match (a, b) {
        (Included(a), Included(b)) => partial_max(a, b).map(Included),
        (Excluded(a), Excluded(b)) => partial_max(a, b).map(Excluded),
        (Included(inc), Excluded(exc)) | (Excluded(exc), Included(inc)) => inc.partial_cmp(&exc).map(|cmp| {
            if matches!(cmp, Ordering::Less | Ordering::Equal) {
                Excluded(exc)
            } else {
                Included(inc)
            }
        }),
        (a, Bound::Unbounded) | (Bound::Unbounded, a) => Some(a),
    }
}

fn lowest_end(a: Bound<ScalarValue>, b: Bound<ScalarValue>) -> Option<Bound<ScalarValue>> {
    use Bound::{Excluded, Included};
    match (a, b) {
        (Included(a), Included(b)) => partial_min(a, b).map(Included),
        (Excluded(a), Excluded(b)) => partial_min(a, b).map(Excluded),
        (Included(inc), Excluded(exc)) | (Excluded(exc), Included(inc)) => exc.partial_cmp(&inc).map(|cmp| {
            if matches!(cmp, Ordering::Less | Ordering::Equal) {
                Excluded(exc)
            } else {
                Included(inc)
            }
        }),
        (a, Bound::Unbounded) | (Bound::Unbounded, a) => Some(a),
    }
}

fn partial_max<T: PartialOrd>(a: T, b: T) -> Option<T> {
    a.partial_cmp(&b)
        .map(|cmp| if matches!(cmp, Ordering::Less) { b } else { a })
}

fn partial_min<T: PartialOrd>(a: T, b: T) -> Option<T> {
    a.partial_cmp(&b)
        .map(|cmp| if matches!(cmp, Ordering::Less) { a } else { b })
}
