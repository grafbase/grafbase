//! This crate implements the logic for partial caching.
//!
//! To make things easier to test this crate is side effect free - the crate that integrates this
//! (gateway_core currently) is resposible for reading & writing to the cache, and running the
//! actual execution.  This crate just determines what to read/write/execute/return to the
//! user.
//!
//! The process is a little bit involved, so it's broken up into phases:
//!
//! 1. First we build a CachingPlan from the incoming query.
//! 2. Then we move into the CacheFetchPhase, which provides keys to a consumer,
//! 3. Then we run an ExecutionPhase if it's required.
//! 4. Then we have a CacheUpdatePhase that runs after the response is retunred

use std::fmt;

use cynic_parser::{
    executable::{ids::OperationDefinitionId, OperationDefinition},
    ExecutableDocument,
};
use registry_for_cache::CacheControl;

mod execution;
mod fetching;
mod headers;
mod hit;
mod output;
mod parser_extensions;
mod planning;
mod query_subset;
mod response;
pub mod type_relationships;
mod updating;

pub use self::{
    execution::{ExecutionPhase, StreamingExecutionPhase},
    fetching::CacheFetchPhase,
    planning::build_plan,
    query_subset::QuerySubset,
    response::Response,
    type_relationships::TypeRelationships,
    updating::CacheUpdatePhase,
};

pub struct CachingPlan {
    pub document: ExecutableDocument,
    pub cache_partitions: Vec<(CacheControl, QuerySubset)>,
    pub nocache_partition: QuerySubset,
    operation_id: OperationDefinitionId,
    defers: Vec<planning::defers::DeferRecord>,
}

impl CachingPlan {
    fn operation(&self) -> OperationDefinition<'_> {
        self.document.read(self.operation_id)
    }
}

impl fmt::Debug for CachingPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachingPlan")
            .field("num_cache_partitions", &self.cache_partitions.len())
            .field("nocache_partition_present", &!self.nocache_partition.is_empty())
            .finish()
    }
}

/// The output of the fetch phase of partial caching
pub enum FetchPhaseResult {
    /// We've only fetched some of the query from the cache, so we need
    /// to enter an ExecutionPhase
    PartialHit(ExecutionPhase),

    /// We fetched all the results from the cache, so can just return a response
    CompleteHit(hit::CompleteHit),
}

#[cfg(test)]
mod tests {
    use insta as _;
    use parser_sdl as _;
    use registry_upgrade as _;
}
