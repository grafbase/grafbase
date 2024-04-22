use std::iter::FusedIterator;

use crate::{
    common::{IdOperations, IdRange},
    Registry, RegistryId,
};

/// Iterator for readers
///
/// T indicates the type that will be yielded by the Iterator
#[derive(Clone, Copy)]
pub struct Iter<'a, T>
where
    T: IdReader,
{
    range: IdRange<T::Id>,
    current: Option<T::Id>,
    document: &'a crate::Registry,
}

impl<'a, T> Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    pub(crate) fn new(range: IdRange<T::Id>, document: &'a Registry) -> Self {
        Iter {
            current: (IdOperations::distance(range.start, range.end) > 0).then_some(range.start),
            range,
            document,
        }
    }
}

pub trait IdReader {
    type Id: RegistryId;
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    type Item = <T::Id as RegistryId>::Reader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        let next = self.range.next(current);
        self.current = next;

        Some(self.document.read(current))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Some(current) = self.current else {
            return (0, Some(0));
        };
        let remaining = IdOperations::distance(current, self.range.end);
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}

impl<'a, T> FusedIterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
}
