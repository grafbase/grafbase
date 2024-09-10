use grafbase_workspace_hack as _;

mod standard;
mod subgraph;

pub async fn introspect(url: &str, headers: &[(impl AsRef<str>, impl AsRef<str>)]) -> Result<String, String> {
    if let Ok(result) = subgraph::introspect(url, headers).await {
        return Ok(prettify(result));
    };

    standard::introspect(url, headers).await.map(prettify)
}

fn prettify(graphql: String) -> String {
    match cynic_parser::parse_type_system_document(&graphql) {
        Ok(parsed) => parsed.to_sdl_pretty(),
        Err(_) => {
            // Don't really want to error out just because we couldn't prettify
            // so return the original string.
            // Definitely possible that it's broken, but at least if we return it
            // a user can potentially fix it manually or w/e
            graphql
        }
    }
}
