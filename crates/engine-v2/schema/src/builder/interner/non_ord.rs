use std::marker::PhantomData;

use regex::Regex;

/// An interner made for types that do not implement Ord (and therefore Hash), and which are expensive
/// to create (looking at you Regex). In many cases using the Interner in the parent module is what
/// you want. This one allocates more and in general should be used in cases where you have no other
/// choice.
pub struct ProxyKeyInterner<T, Id>(indexmap::IndexMap<Vec<u8>, T, fnv::FnvBuildHasher>, PhantomData<Id>);

impl<T, Id> Default for ProxyKeyInterner<T, Id> {
    fn default() -> Self {
        Self(
            indexmap::IndexMap::with_hasher(fnv::FnvBuildHasher::default()),
            PhantomData,
        )
    }
}

pub trait ToKey {
    fn to_key(&self) -> Vec<u8>;
}

impl ToKey for Regex {
    fn to_key(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl<T, Id> ProxyKeyInterner<T, Id>
where
    T: ToKey,
    Id: Copy + From<usize> + Into<usize>,
{
    pub fn from_vec(existing: Vec<T>) -> Self {
        let iter = existing.into_iter().map(|t| (t.to_key(), t)).collect();
        Self(iter, PhantomData)
    }

    pub fn get_by_id(&self, id: Id) -> Option<&T> {
        self.0.get_index(id.into()).map(|(_, t)| t)
    }

    pub fn insert(&mut self, value: T) -> Id {
        self.0.insert_full(value.to_key(), value).0.into()
    }

    pub fn extend(&mut self, other: impl IntoIterator<Item = T>) {
        self.0.extend(other.into_iter().map(|t| (t.to_key(), t)))
    }

    pub fn get_or_insert(&mut self, value: T) -> Id {
        let key = value.to_key();
        self.0
            .get_full(&key)
            .map(|(id, _, _)| id.into())
            .unwrap_or_else(|| self.0.insert_full(key, value).0.into())
    }
}

impl<T, Id: Into<usize>> std::ops::Index<Id> for ProxyKeyInterner<T, Id> {
    type Output = T;

    fn index(&self, index: Id) -> &T {
        &self.0[index.into()]
    }
}

impl<T, Id> IntoIterator for ProxyKeyInterner<T, Id> {
    type Item = (Vec<u8>, T);
    type IntoIter = <indexmap::IndexMap<Vec<u8>, T, fnv::FnvBuildHasher> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, Id> From<ProxyKeyInterner<T, Id>> for Vec<T> {
    fn from(interner: ProxyKeyInterner<T, Id>) -> Self {
        interner.into_iter().map(|(_, t)| t).collect()
    }
}
