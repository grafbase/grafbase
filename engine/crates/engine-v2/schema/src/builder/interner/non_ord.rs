use std::marker::PhantomData;

use regex::Regex;

/// An interner made for types that do not implement Ord (and therefore Hash), and which are expensive
/// to create (looking at you Regex). In many cases using the Interner in the parent module is what
/// you want. This one allocates more and in general should be used in cases where you have no other
/// choice.
pub struct NonOrdInterner<T, Id>(indexmap::IndexMap<String, T, fnv::FnvBuildHasher>, PhantomData<Id>);

impl<T, Id> Default for NonOrdInterner<T, Id> {
    fn default() -> Self {
        Self(
            indexmap::IndexMap::with_hasher(fnv::FnvBuildHasher::default()),
            PhantomData,
        )
    }
}

impl<T, Id> NonOrdInterner<T, Id>
where
    T: ToString,
    Id: Copy + From<usize> + Into<usize>,
{
    pub fn from_vec(existing: Vec<T>) -> Self {
        let iter = existing.into_iter().map(|t| (t.to_string(), t)).collect();
        Self(iter, PhantomData)
    }

    pub fn get_by_id(&self, id: Id) -> Option<&T> {
        self.0.get_index(id.into()).map(|(_, t)| t)
    }

    pub fn insert(&mut self, value: T) -> Id {
        self.0.insert_full(value.to_string(), value).0.into()
    }

    pub fn extend(&mut self, other: impl IntoIterator<Item = T>) {
        self.0.extend(other.into_iter().map(|t| (t.to_string(), t)))
    }
}

impl<T, Id: Into<usize>> std::ops::Index<Id> for NonOrdInterner<T, Id> {
    type Output = T;

    fn index(&self, index: Id) -> &T {
        &self.0[index.into()]
    }
}

impl<Id> NonOrdInterner<Regex, Id>
where
    Id: Copy + From<usize> + Into<usize>,
{
    pub fn get_or_insert(&mut self, value: &Regex) -> Id {
        self.0
            .get_full(value.as_str())
            .map(|(id, _, _)| id.into())
            .unwrap_or_else(|| self.insert(value.clone()))
    }
}

impl<T, Id> IntoIterator for NonOrdInterner<T, Id> {
    type Item = (String, T);
    type IntoIter = <indexmap::IndexMap<String, T, fnv::FnvBuildHasher> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, Id> From<NonOrdInterner<T, Id>> for Vec<T> {
    fn from(interner: NonOrdInterner<T, Id>) -> Self {
        interner.into_iter().map(|(_, t)| t).collect()
    }
}
