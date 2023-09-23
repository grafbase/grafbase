use std::collections::HashMap;

pub mod cache;
pub mod kv;
pub mod log;
pub mod search;
pub mod udf;

/// Context specific to the request, usable by any service.
#[derive(Clone, Debug)]
pub struct GraphqlRequestExecutionContext {
    /// Used to track request across services.
    pub ray_id: String,
    pub fetch_log_endpoint_url: Option<String>,
    pub headers: HashMap<String, String>,
}
