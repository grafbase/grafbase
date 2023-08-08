/// Generate the page for Pathfinder
///
/// # Example
///
/// ```rust
/// use dynaql::http::*;
///
/// pathfinder_source(PathfinderConfig::new("http://localhost:8000"));
/// ```
pub fn pathfinder_source(config: PathfinderConfig) -> String {
    include_str!("pathfinder.html")
        .replace("{{GRAPHQL_URL}}", config.endpoint)
        .replace("{{ASSET_URL}}", "https://assets.grafbase.com/cli/pathfinder")
}

/// Config for Pathfinder
pub struct PathfinderConfig<'a> {
    endpoint: &'a str,
}

impl<'a> PathfinderConfig<'a> {
    /// Create a config for Pathfinder
    pub fn new(endpoint: &'a str) -> Self {
        Self { endpoint }
    }
}
