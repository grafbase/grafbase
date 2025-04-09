use schema::DirectiveSiteId;

#[derive(Clone, Debug)]
pub struct McpRequestContext {
    pub can_mutate: bool,
}

#[derive(Clone, Debug)]
pub struct McpResponseExtension {
    pub site_ids: Vec<DirectiveSiteId>,
}
