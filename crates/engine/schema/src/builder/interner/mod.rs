mod proxy;

pub use proxy::ProxyKeyInterner;

use std::{borrow::Borrow, marker::PhantomData};

#[derive(Debug)]
pub struct Interner<T, Id>(indexmap::IndexSet<T, rapidhash::fast::RandomState>, PhantomData<Id>);

impl<T, Id> Default for Interner<T, Id> {
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}

impl<T: core::hash::Hash + PartialEq + Eq, Id: Copy + From<usize> + Into<usize>> Interner<T, Id> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(
            indexmap::IndexSet::with_capacity_and_hasher(capacity, Default::default()),
            PhantomData,
        )
    }

    pub fn get_by_id(&self, id: Id) -> Option<&T> {
        self.0.get_index(id.into())
    }

    pub fn insert(&mut self, value: T) -> Id {
        self.0.insert_full(value).0.into()
    }

    pub fn extend(&mut self, other: impl IntoIterator<Item = T>) {
        self.0.extend(other)
    }

    pub fn get_or_insert(&mut self, value: T) -> Id {
        self.0
            .get_full(&value)
            .map(|(id, _)| id.into())
            .unwrap_or_else(|| self.insert(value))
    }

    pub fn get_or_new<Q>(&mut self, value: &Q) -> Id
    where
        T: Borrow<Q> + for<'a> From<&'a Q>,
        Q: ?Sized + Eq + std::hash::Hash,
    {
        self.0
            .get_full(value.borrow())
            .map(|(id, _)| id.into())
            .unwrap_or_else(|| self.insert(value.into()))
    }
}

impl<T, Id: Into<usize>> std::ops::Index<Id> for Interner<T, Id> {
    type Output = T;

    fn index(&self, index: Id) -> &T {
        &self.0[index.into()]
    }
}

impl<T, Id> IntoIterator for Interner<T, Id> {
    type Item = T;
    type IntoIter = <indexmap::IndexSet<T, rapidhash::fast::RandomState> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, Id> From<Interner<T, Id>> for Vec<T> {
    fn from(interner: Interner<T, Id>) -> Self {
        interner.into_iter().collect()
    }
}
