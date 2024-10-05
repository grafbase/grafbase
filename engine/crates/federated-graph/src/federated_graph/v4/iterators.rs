use super::{View, ViewNested};

impl super::FederatedGraph {
    /// Precondition: `items` is sorted by `key`.
    pub(super) fn iter_by_sort_key<'a, Key, Record>(
        &'a self,
        key: Key,
        items: &'a [Record],
        extract_key: impl Fn(&Record) -> Key + Clone + 'static,
    ) -> impl Iterator<Item = ViewNested<'a, Key, Record>> + Clone
    where
        Key: From<usize> + Clone + PartialOrd + 'static,
    {
        let start = items.partition_point(|record| extract_key(record) < key);

        items[start..]
            .iter()
            .take_while(move |record| extract_key(record) == key)
            .enumerate()
            .map(move |(idx, record)| ViewNested {
                graph: self,
                view: View {
                    id: Key::from(start + idx),
                    record,
                },
            })
    }
}
