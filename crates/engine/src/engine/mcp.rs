use schema::DirectiveSiteId;

#[derive(Clone, Debug)]
pub struct McpRequestContext {
    pub execute_mutations: bool,
}

#[derive(Clone, Debug)]
pub struct McpResponseExtension {
    pub site_ids: Vec<DirectiveSiteId>,
}
