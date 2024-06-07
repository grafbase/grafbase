use graph_entities::QueryResponse;
use runtime::cache::Entry;

use crate::{execution::merge_json, response::MaxAge, CacheUpdatePhase, Response};

/// This struct is used when we found everything we needed in the cache
/// and don't need to make a call to the executor at all.
pub struct CompleteHit {
    cache_entries: Vec<Entry<serde_json::Value>>,
}

impl CompleteHit {
    pub(crate) fn new(cache_entries: Vec<Entry<serde_json::Value>>) -> Self {
        CompleteHit { cache_entries }
    }

    pub fn response_and_updates(self) -> (Response, Option<CacheUpdatePhase>) {
        let mut body = QueryResponse::default();
        let mut cache_entries = self.cache_entries.into_iter();

        let Some(first_entry) = cache_entries.next() else {
            // This really shouldn't happen, but not much else we can do.
            // I'd rather not panic for this case as its not an obvious invariant
            return (Response::hit(body, MaxAge::None), None);
        };

        let mut response_max_age = MaxAge::default();

        match first_entry {
            Entry::Hit(value, time_till_miss) => {
                let root_id = body.from_serde_value(value);
                body.set_root_unchecked(root_id);
                response_max_age.merge(time_till_miss);
            }
            Entry::Miss => {
                // This is an obvious invariant so lets panic
                unreachable!("a complete hit should have no misses")
            }
            Entry::Stale(stale) => {
                let root_id = body.from_serde_value(stale.value);
                body.set_root_unchecked(root_id);

                // This entry was stale so instruct downstreams not to cache
                // until we have revalidated
                response_max_age.set_none();
            }
        }

        // Merge in the rest of the entries
        for entry in cache_entries {
            match entry {
                Entry::Hit(value, time_till_miss) => {
                    merge_json(&mut body, value);

                    response_max_age.merge(time_till_miss);
                }
                Entry::Miss => {
                    // This is an obvious invariant so lets panic
                    unreachable!("a complete hit should have no misses")
                }
                Entry::Stale(stale) => {
                    merge_json(&mut body, stale.value);

                    // This entry was stale so clear the current maxAge
                    // until we have revalidated
                    response_max_age.set_none();
                }
            }
        }

        (Response::hit(body, response_max_age), None)
    }
}
