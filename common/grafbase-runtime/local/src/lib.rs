mod bridge;
pub mod search;
pub use search::LocalSearchEngine;

pub struct ExecutionContext {
    pub request_id: String,
}
