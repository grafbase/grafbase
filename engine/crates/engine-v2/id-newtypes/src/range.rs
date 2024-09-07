use std::ops::Range;

use walker::{Walk, WalkIterator};

// Not necessary anymore when Rust stabilize std::iter::Step
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct IdRange<Id: Copy> {
    pub start: Id,
    pub end: Id,
}

pub trait IdOperations: Copy + From<usize> + Into<usize> {}

impl<Id: IdOperations> Default for IdRange<Id> {
    fn default() -> Self {
        Self {
            start: Id::from(0),
            end: Id::from(0),
        }
    }
}

impl<Id: IdOperations> From<IdRange<Id>> for Range<usize> {
    fn from(value: IdRange<Id>) -> Self {
        Range {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

impl<Id: IdOperations> IdRange<Id> {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.end.into() - self.start.into()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, i: usize) -> Option<Id> {
        let i = self.start.into() + i;
        if i < self.end.into() {
            Some(Id::from(i))
        } else {
            None
        }
    }

    pub fn index_of(&self, id: Id) -> Option<usize> {
        let id = id.into();
        let start = self.start.into();
        if id >= start && id < self.end.into() {
            Some(id - start)
        } else {
            None
        }
    }

    // An From/Into would be nicer, but it's dangerously also works for (usize, usize)
    // which could also be interpreted as a range (start, end).
    pub fn from_start_and_length<Src>((start, len): (Src, usize)) -> Self
    where
        Id: From<Src>,
    {
        let start = Id::from(start);
        Self {
            start,
            end: Id::from(start.into() + len),
        }
    }

    pub fn from_single(id: Id) -> Self {
        Self {
            start: id,
            end: Id::from(id.into() + 1),
        }
    }

    pub fn from_slice(ids: &[Id]) -> Option<Self> {
        let mut ids = ids.iter();
        let Some(first) = ids.next() else {
            return Some(Self::empty());
        };
        let start: usize = (*first).into();
        let mut end = start;
        for id in ids {
            if (*id).into() != end + 1 {
                return None;
            }
            end += 1;
        }
        Some(Self {
            start: *first,
            end: Id::from(end + 1),
        })
    }

    pub fn start(&self) -> Id {
        self.start
    }

    pub fn end(&self) -> Id {
        self.end
    }
}

impl<Id, T> From<Range<T>> for IdRange<Id>
where
    Id: From<T> + Copy,
{
    fn from(Range { start, end }: Range<T>) -> Self {
        Self {
            start: Id::from(start),
            end: Id::from(end),
        }
    }
}

impl<G, Id: Walk<G> + IdOperations + 'static> Walk<G> for IdRange<Id> {
    type Walker<'a> = WalkIterator<'a, IdRangeIterator<Id>, G>
    where G: 'a;

    fn walk<'a>(self, graph: &'a G) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        WalkIterator::new(self.into_iter(), graph)
    }
}

impl<Id: IdOperations> IntoIterator for IdRange<Id> {
    type Item = Id;
    type IntoIter = IdRangeIterator<Id>;

    fn into_iter(self) -> Self::IntoIter {
        IdRangeIterator(self)
    }
}

#[derive(Clone)]
pub struct IdRangeIterator<Id: Copy>(IdRange<Id>);

impl<Id: IdOperations> Iterator for IdRangeIterator<Id> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.0.is_empty() {
            let id = self.0.start;
            self.0.start = Id::from(id.into() + 1);
            Some(id)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }
}

impl<Id: IdOperations> ExactSizeIterator for IdRangeIterator<Id> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<Id: IdOperations> DoubleEndedIterator for IdRangeIterator<Id> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if !self.0.is_empty() {
            self.0.end = Id::from(self.0.end.into() - 1);
            Some(self.0.end)
        } else {
            None
        }
    }
}

impl<Id: IdOperations> std::iter::FusedIterator for IdRangeIterator<Id> {}
