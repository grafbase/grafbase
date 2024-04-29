use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
/// A half open range of Ids.
pub struct IdRange<Id> {
    pub(crate) start: Id,
    pub(crate) end: Id,
}

pub trait IdOperations: Copy {
    fn forward(self) -> Option<Self>;
    fn back(self) -> Option<Self>;
    fn cmp(self, other: Self) -> Ordering;
    fn distance(lhs: Self, rhs: Self) -> usize;
}

impl<Id> IdRange<Id> {
    pub(crate) fn new(start: Id, end: Id) -> Self {
        IdRange { start, end }
    }

    pub(crate) fn next(&self) -> Option<Id>
    where
        Id: IdOperations,
    {
        let next = self.start;
        matches!(next.cmp(self.end), Ordering::Less).then_some(next)
    }
}

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
