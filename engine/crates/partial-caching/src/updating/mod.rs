//! This module handles the cache update phase that runs after we've called the executor

mod ser;

use cynic_parser::ExecutableDocument;
use graph_entities::QueryResponse;
use registry_for_cache::CacheControl;

use crate::QuerySubset;

pub(crate) struct PartitionIndex(pub(crate) usize);

pub struct CacheUpdatePhase {
    document: ExecutableDocument,

    cache_partitions: Vec<(CacheControl, QuerySubset)>,
    keys_to_write: Vec<(String, PartitionIndex)>,

    /// The response from the executor.  Currently this should not contain
    /// any of the data fetched from cache as part of our cache phase
    /// (that's not a requirement, it was just easiest to implement on a deadline)
    response: QueryResponse,
}

pub struct CacheUpdate<'a> {
    pub key: &'a str,
    pub cache_control: &'a CacheControl,

    document: &'a ExecutableDocument,
    subset: &'a QuerySubset,
    response: &'a QueryResponse,
}

impl CacheUpdatePhase {
    pub(crate) fn new(
        document: ExecutableDocument,
        cache_partitions: Vec<(CacheControl, QuerySubset)>,
        keys_to_write: Vec<(String, PartitionIndex)>,
        response: QueryResponse,
    ) -> Self {
        CacheUpdatePhase {
            document,
            cache_partitions,
            keys_to_write,
            response,
        }
    }

    pub fn updates(&self) -> impl Iterator<Item = CacheUpdate<'_>> + '_ {
        self.keys_to_write.iter().filter_map(|(key, index)| {
            let (cache_control, subset) = self.cache_partitions.get(index.0)?;

            Some(CacheUpdate {
                key,
                document: &self.document,
                cache_control,
                subset,
                response: &self.response,
            })
        })
    }
}
