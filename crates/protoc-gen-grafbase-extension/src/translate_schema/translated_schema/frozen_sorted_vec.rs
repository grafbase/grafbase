use super::View;
use std::ops::Deref;

/// A frozen, sorted vec. We use it to represent items with a stable id in the schema.
///
/// Frozen means it is append-only. This gives us stability of ids.
///
/// Sorted means it can be binary searched.
#[derive(Debug)]
pub(crate) struct FrozenSortedVec<T> {
    items: Vec<T>,
}

impl<T> Default for FrozenSortedVec<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T> Deref for FrozenSortedVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T: PartialOrd + std::fmt::Debug> FrozenSortedVec<T> {
    /// Panics if insertion is not in order. Returns the index of the inserted item.
    pub fn push(&mut self, item: T) -> usize {
        if let Some(last_item) = self.items.last() {
            assert!(last_item <= &item);
        };

        let id = self.items.len();

        self.items.push(item);

        id
    }

    /// Iterate over a range of items with a given sort prefix. The prefix must at the beginning of the sort key for items. That is not enforced.
    pub fn iter_with_prefix<P: PartialOrd, Id>(
        &self,
        prefix: P,
        project_prefix: impl Fn(&T) -> P,
    ) -> impl Iterator<Item = View<'_, Id, T>>
    where
        Id: From<usize>,
    {
        let partition_point = self.partition_point(|item| project_prefix(item) < prefix);

        self[partition_point..]
            .iter()
            .take_while(move |item| project_prefix(item) == prefix)
            .enumerate()
            .map(|(idx, record)| View::new(idx, record))
    }
}
