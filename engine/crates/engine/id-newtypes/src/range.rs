use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
/// A half open range of Ids.
pub struct IdRange<Id> {
    pub start: Id,
    pub end: Id,
}

pub trait IdOperations: Copy {
    fn empty_range() -> IdRange<Self>;
    fn forward(self) -> Option<Self>;
    fn back(self) -> Option<Self>;
    fn cmp(self, other: Self) -> Ordering;
    fn distance(lhs: Self, rhs: Self) -> usize;
}

impl<Id> IdRange<Id> {
    pub fn new(start: Id, end: Id) -> Self {
        IdRange { start, end }
    }

    pub fn is_empty(&self) -> bool
    where
        Id: IdOperations,
    {
        IdOperations::distance(self.start, self.end) == 0
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = Id>
    where
        Id: IdOperations,
    {
        *self
    }
}

impl<Id> Default for IdRange<Id>
where
    Id: IdOperations,
{
    fn default() -> Self {
        Id::empty_range()
    }
}

impl<Id> Iterator for IdRange<Id>
where
    Id: IdOperations,
{
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if IdOperations::cmp(self.start, self.end).is_eq() {
            return None;
        }
        let current = self.start;
        self.start = self.start.forward()?;
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = IdOperations::distance(self.start, self.end);
        (size, Some(size))
    }
}

impl<Id> ExactSizeIterator for IdRange<Id> where Id: IdOperations {}

impl<T> serde::Serialize for IdRange<T>
where
    T: Serialize + Copy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        [self.start, self.end].serialize(serializer)
    }
}

impl<'de, T> serde::Deserialize<'de> for IdRange<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let [start, end] = <[T; 2]>::deserialize(deserializer)?;

        Ok(IdRange { start, end })
    }
}
