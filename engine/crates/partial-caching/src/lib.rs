use cynic_parser::ExecutableDocument;
use registry_for_cache::CacheControl;

mod planning;
mod query_subset;

pub use self::{planning::build_plan, query_subset::QuerySubset};

pub struct CachingPlan {
    pub document: ExecutableDocument,
    pub cache_queries: Vec<(CacheControl, QuerySubset)>,
    pub executor_query: QuerySubset,
}

#[cfg(test)]
mod tests {
    use insta as _;
    use parser_sdl as _;
    use registry_upgrade as _;
}
