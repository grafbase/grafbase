use serde::Serialize;

/// Generate the page for GraphQL Playground
///
/// # Example
///
/// ```rust
/// use dynaql::http::*;
///
/// playground_source(GraphQLPlaygroundConfig::new("http://localhost:8000"));
/// ```
pub fn playground_source(config: GraphQLPlaygroundConfig) -> String {
    include_str!("playground.html")
        .replace("{{GRAPHQL_URL}}", config.endpoint)
        .replace(
            "{{ASSET_URL}}",
            "https://assets.grafbase.com/cli/pathfinder",
        )
}

/// Config for GraphQL Playground
pub struct GraphQLPlaygroundConfig<'a> {
    endpoint: &'a str,
}

impl<'a> GraphQLPlaygroundConfig<'a> {
    /// Create a config for GraphQL playground.
    pub fn new(endpoint: &'a str) -> Self {
        Self { endpoint }
    }
}
