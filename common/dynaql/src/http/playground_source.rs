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
    r##"
      <!DOCTYPE html>
      <html lang="en">
        <head>
          <meta charset="utf-8" />
          <meta name="viewport" content="width=device-width,initial-scale=1" />
          <title>Grafbase Playground</title>
          <link rel="shortcut icon" href="https://grafbase.com/images/other/grafbase-logo-circle.png" />
          <script>
            window.GRAPHQL_URL = "{{GRAPHQL_URL}}";
          </script>
          <script defer="defer" src="{{ASSET_URL}}/main.js"></script>
          <link href="{{ASSET_URL}}/main.css" rel="stylesheet" />
          <link
            href="https://cdn.jsdelivr.net/npm/@grafbase/graphiql@2.0.2/dist/index.css"
            rel="stylesheet"
          />
        </head>
        <body>
          <noscript>You need to enable JavaScript to run the Grafbase playground</noscript>
          <div id="root"></div>
        </body>
      </html>    
    "##
    .replace("{{GRAPHQL_URL}}", config.endpoint)
    .replace(
        "{{ASSET_URL}}",
        "https://assets.grafbase.com/playground",
    )
}

/// Config for GraphQL Playground
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLPlaygroundConfig<'a> {
    endpoint: &'a str,
}

impl<'a> GraphQLPlaygroundConfig<'a> {
    /// Create a config for GraphQL playground.
    pub fn new(endpoint: &'a str) -> Self {
        Self { endpoint }
    }
}
