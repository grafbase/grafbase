//! Implements the execution phase of caching - quite a simple one this, it just
//! takes the original query, removes any parts for which we have cache it and
//! provides whatever is left.  This can be passed to the executor to run the
//! query.

use cynic_parser::ExecutableDocument;
use registry_for_cache::CacheControl;

use crate::QuerySubset;

use super::fetching::CacheFetchPhase;

#[allow(unused)] // Going to update things to use this later
pub struct ExecutionPhase {
    document: ExecutableDocument,
    cache_partitions: Vec<(CacheControl, QuerySubset)>,
    executor_subset: QuerySubset,
}

impl ExecutionPhase {
    pub(crate) fn new(fetch_phase: CacheFetchPhase) -> Self {
        let plan = fetch_phase.plan;

        let mut executor_subset = plan.nocache_partition;
        for (entry, (_, partition_subset)) in fetch_phase.cache_entries.iter().zip(plan.cache_partitions.iter()) {
            if entry.is_miss() {
                executor_subset.extend(partition_subset);
            }
        }

        Self {
            document: plan.document,
            cache_partitions: plan.cache_partitions,
            executor_subset,
        }
    }

    pub fn query(&self) -> String {
        self.executor_subset
            .as_display(&self.document)
            .include_query_name()
            .to_string()
    }
}
