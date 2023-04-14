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
        <script>
          window.GRAPHQL_URL = "{{GRAPHQL_URL}}";
        </script>
        <script>
          self["MonacoEnvironment"] = (function (paths) {
            return {
              globalAPI: false,
              getWorkerUrl: function (moduleId, label) {
                var result = paths[label];
                if (/^((http:)|(https:)|(file:)|(\/\/))/.test(result)) {
                  var currentUrl = String(window.location);
                  var currentOrigin = currentUrl.substr(
                    0,
                    currentUrl.length -
                      window.location.hash.length -
                      window.location.search.length -
                      window.location.pathname.length
                  );
                  if (result.substring(0, currentOrigin.length) !== currentOrigin) {
                    var js = "/*" + label + '*/importScripts("' + result + '");';
                    var blob = new Blob([js], { type: "application/javascript" });
                    return URL.createObjectURL(blob);
                  }
                }
                return result;
              },
            };
          })({
            json: "{{ASSET_URL}}/monacoeditorwork/json.worker.bundle.js",
            editorWorkerService:
              "{{ASSET_URL}}/monacoeditorwork/editor.worker.bundle.js",
            graphql: "{{ASSET_URL}}/monacoeditorwork/graphql.worker.bundle.js",
          });
        </script>
    
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link
          rel="shortcut icon"
          href="https://grafbase.com/images/other/grafbase-logo-circle.png"
        />
    
        <title>Grafbase Playground</title>
        <script
          type="module"
          crossorigin
          src="{{ASSET_URL}}/assets/index.js"
        ></script>
        <link rel="stylesheet" href="{{ASSET_URL}}/assets/index.css" />
      </head>
      <body>
        <div id="root"></div>
      </body>
    </html>    
    "##
    .replace("{{GRAPHQL_URL}}", config.endpoint)
    .replace(
        "{{ASSET_URL}}",
        "https://assets.grafbase.com/cli/pathfinder",
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
