use std::iter::FusedIterator;

use crate::{Registry, RegistryId};
use engine_id_newtypes::{IdOperations, IdRange};

/// Iterator for readers
///
/// T indicates the type that will be yielded by the Iterator
#[derive(Clone, Copy)]
pub struct Iter<'a, T>
where
    T: IdReader,
{
    range: IdRange<T::Id>,
    registry: &'a crate::Registry,
}

impl<'a, T> Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
{
    pub(crate) fn new(range: IdRange<T::Id>, registry: &'a Registry) -> Self {
        Iter { range, registry }
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
        let next = self.range.next()?;

        Some(self.registry.read(next))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
    IdRange<T::Id>: ExactSizeIterator,
{
}

impl<'a, T> FusedIterator for Iter<'a, T>
where
    T: IdReader,
    T::Id: IdOperations,
    IdRange<T::Id>: FusedIterator,
{
}
