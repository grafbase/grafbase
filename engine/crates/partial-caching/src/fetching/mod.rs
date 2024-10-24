//! This module handles the fetching phase of partial caching, where we're looking up things
//! in the cache and refining the exeuction query further based on what we find.

mod keys;

use std::{fmt, sync::Arc};

use common_types::auth::ExecutionAuth;
use engine_value::Variables;
use headers::HeaderMapExt;
use runtime::cache::Entry;

use self::keys::build_cache_keys;
use super::CachingPlan;
use crate::{
    execution::ExecutionPhase, headers::RequestCacheControl, hit::CompleteHit, FetchPhaseResult, TypeRelationships,
};

impl CachingPlan {
    pub fn start_fetch_phase(
        self,
        auth: &ExecutionAuth,
        headers: &http::HeaderMap,
        variables: &Variables,
    ) -> CacheFetchPhase {
        let cache_headers = headers
            .typed_get::<headers::CacheControl>()
            .unwrap_or_else(headers::CacheControl::new);

        CacheFetchPhase {
            cache_keys: build_cache_keys(&self, auth, headers, variables),
            cache_entries: std::iter::repeat_with(|| Entry::Miss)
                .take(self.cache_partitions.len())
                .collect(),
            plan: self,
            request_cache_control: cache_headers.into(),
        }
    }
}

/// This struct should be used to manage the cache fetching phase.
pub struct CacheFetchPhase {
    /// The CachingPlan that we're doing a fetch for.
    pub(crate) plan: CachingPlan,

    /// The keys for each cache_query in the plan.
    ///
    /// Will be None if we couldn't determine a key for whatever reason, in
    /// which case we'll always query the executor for those fields
    pub(crate) cache_keys: Vec<Option<String>>,

    /// The cache control headers a user has provided
    pub(crate) request_cache_control: RequestCacheControl,

    pub(crate) cache_entries: Vec<Entry<serde_json::Value>>,
}

/// The externally visible representation of a cache key
#[derive(Clone, Debug)]
pub struct CacheKey {
    index: usize,
    key: String,
}

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key)
    }
}

impl CacheFetchPhase {
    /// The keys that we need to fetch from the cache
    pub fn cache_keys(&self) -> Vec<CacheKey> {
        if !self.request_cache_control.should_read_from_cache {
            return vec![];
        }

        self.cache_keys
            .iter()
            .enumerate()
            .filter_map(|(index, key)| {
                Some(CacheKey {
                    index,
                    key: key.as_ref()?.to_string(),
                })
            })
            .collect()
    }

    /// Records the response from the cache for a given key
    pub fn record_cache_entry(&mut self, key: &CacheKey, entry: Entry<serde_json::Value>) {
        self.cache_entries[key.index] = entry;
    }

    pub fn finish(self, subtypes: Arc<dyn TypeRelationships>) -> FetchPhaseResult {
        if self.cache_entries.iter().any(|entry| entry.is_miss()) || !self.plan.nocache_partition.is_empty() {
            FetchPhaseResult::PartialHit(Box::new(ExecutionPhase::new(self, subtypes)))
        } else {
            FetchPhaseResult::CompleteHit(CompleteHit::new(self.cache_entries))
        }
    }
}
