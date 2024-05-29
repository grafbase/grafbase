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
//! 4. TBC after that (need to write the code first)

use cynic_parser::ExecutableDocument;
use registry_for_cache::CacheControl;

mod execution;
mod fetching;
mod hit;
mod planning;
mod query_subset;

pub use self::{execution::ExecutionPhase, planning::build_plan, query_subset::QuerySubset};

// Renaming this because we have registry_for_cache::CacheControl & headers::CacheControl and
// it's confusing when you're working with both of them.  Hopefully the alias doesn't add
// it's own confusion :|
pub use headers::CacheControl as CacheControlHeaders;

pub struct CachingPlan {
    pub document: ExecutableDocument,
    pub cache_partitions: Vec<(CacheControl, QuerySubset)>,
    pub nocache_partition: QuerySubset,
}

/// The output of the fetch phase of partial caching
pub enum FetchPhaseResult {
    /// We've only fetched some of the query from the cache, so we need
    /// to enter an ExecutionPhase
    PartialHit(ExecutionPhase),

    /// We fetched all the results from the cache, so can just return a response
    ///
    /// Note that I've not implemented this bit yet - it'll come later.
    CompleteHit(hit::CompleteHit),
}

#[cfg(test)]
mod tests {
    use insta as _;
    use parser_sdl as _;
    use registry_upgrade as _;
}
