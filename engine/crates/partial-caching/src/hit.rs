use graph_entities::QueryResponse;
use runtime::cache::Entry;

use crate::{execution::merge_json, CacheUpdatePhase};

/// This struct is used when we found everything we needed in the cache
/// and don't need to make a call to the executor at all.
pub struct CompleteHit {
    cache_entries: Vec<Entry<serde_json::Value>>,
}

impl CompleteHit {
    pub(crate) fn new(cache_entries: Vec<Entry<serde_json::Value>>) -> Self {
        CompleteHit { cache_entries }
    }

    pub fn response_and_updates(self) -> (QueryResponse, Option<CacheUpdatePhase>) {
        let mut response = QueryResponse::default();
        let mut cache_entries = self.cache_entries.into_iter();

        let Some(first_entry) = cache_entries.next() else {
            // This really shouldn't happen, but not much else we can do.
            // I'd rather not panic for this case as its not an obvious invariant
            return (response, None);
        };

        match first_entry {
            Entry::Hit(value) => {
                let root_id = response.from_serde_value(value);
                response.set_root_unchecked(root_id);
            }
            Entry::Miss => {
                // This is an obvious invariant so lets panic
                unreachable!("a complete hit should have no misses")
            }
            Entry::Stale(stale) => {
                let root_id = response.from_serde_value(stale.value);
                response.set_root_unchecked(root_id);
            }
        }

        // Merge in the rest of the entries
        for entry in cache_entries {
            match entry {
                Entry::Hit(value) => merge_json(&mut response, value),
                Entry::Miss => {
                    // This is an obvious invariant so lets panic
                    unreachable!("a complete hit should have no misses")
                }
                Entry::Stale(stale) => merge_json(&mut response, stale.value),
            }
        }

        (response, None)
    }
}
