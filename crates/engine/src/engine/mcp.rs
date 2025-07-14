use std::sync::Arc;

use schema::{DirectiveSiteId, Schema};

#[derive(Clone, Debug)]
pub struct McpRequestContext {
    pub execute_mutations: bool,
}

#[derive(Clone, Debug)]
pub struct McpResponseExtension {
    pub schema: Arc<Schema>,
    pub site_ids: Vec<DirectiveSiteId>,
}
