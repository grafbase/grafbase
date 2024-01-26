use std::marker::PhantomData;

use crate::{CacheConfigId, StringId};

pub struct Interner<T, Id>(indexmap::IndexSet<T, fnv::FnvBuildHasher>, PhantomData<Id>);

impl<T, Id> Default for Interner<T, Id> {
    fn default() -> Self {
        Self(
            indexmap::IndexSet::with_hasher(fnv::FnvBuildHasher::default()),
            PhantomData,
        )
    }
}

impl<T: core::hash::Hash + PartialEq + Eq, Id: Copy + From<usize> + Into<usize>> Interner<T, Id> {
    pub fn from_vec(existing: Vec<T>) -> Self {
        Self(existing.into_iter().collect(), PhantomData)
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
}

impl<T, Id: Into<usize>> std::ops::Index<Id> for Interner<T, Id> {
    type Output = T;

    fn index(&self, index: Id) -> &T {
        &self.0[index.into()]
    }
}

impl Interner<config::latest::CacheConfig, CacheConfigId> {
    pub fn get_or_insert(&mut self, value: &config::latest::CacheConfig) -> CacheConfigId {
        self.0
            .get_full(value)
            .map(|(id, _)| id.into())
            .unwrap_or_else(|| self.insert(value.clone()))
    }
}

impl Interner<String, StringId> {
    pub fn get_or_insert(&mut self, value: &str) -> StringId {
        self.0
            .get_full(value)
            .map(|(id, _)| id.into())
            .unwrap_or_else(|| self.insert(value.to_string()))
    }
}

impl<T, Id> IntoIterator for Interner<T, Id> {
    type Item = T;
    type IntoIter = <indexmap::IndexSet<T, fnv::FnvBuildHasher> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, Id> From<Interner<T, Id>> for Vec<T> {
    fn from(interner: Interner<T, Id>) -> Self {
        interner.into_iter().collect()
    }
}
