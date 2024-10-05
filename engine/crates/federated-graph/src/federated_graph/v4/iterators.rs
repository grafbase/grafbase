use super::{View, ViewNested};

impl super::FederatedGraph {
    /// Precondition: `items` is sorted by `key`.
    pub(super) fn iter_by_sort_key<'a, ParentKey, RecordKey, Record>(
        &'a self,
        key: ParentKey,
        items: &'a [Record],
        extract_key: impl Fn(&Record) -> ParentKey + Clone + 'static,
    ) -> impl Iterator<Item = ViewNested<'a, RecordKey, Record>> + Clone
    where
        ParentKey: Clone + PartialOrd + 'static,
        RecordKey: From<usize> + Clone + 'static,
    {
        let start = items.partition_point(|record| extract_key(record) < key);

        items[start..]
            .iter()
            .take_while(move |record| extract_key(record) == key)
            .enumerate()
            .map(move |(idx, record)| ViewNested {
                graph: self,
                view: View {
                    id: RecordKey::from(start + idx),
                    record,
                },
            })
    }
}
