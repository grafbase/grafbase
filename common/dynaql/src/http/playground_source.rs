use crate::Value;
use serde::Serialize;
use std::collections::HashMap;

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
          <script>
            window.GRAPHQL_URL = "{{GRAPHQL_URL}}";
          </script>
          <script defer="defer" src="{{ASSETS_URL}}/main.js"></script>
          <link href="{{ASSETS_URL}}/main.css" rel="stylesheet" />
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
        "{{ASSETS_URL}}",
        "https://temp-artifact-storage.s3.eu-west-3.amazonaws.com",
    )
}

/// Config for GraphQL Playground
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLPlaygroundConfig<'a> {
    endpoint: &'a str,
    subscription_endpoint: Option<&'a str>,
    headers: Option<HashMap<&'a str, &'a str>>,
    settings: Option<HashMap<&'a str, Value>>,
}

impl<'a> GraphQLPlaygroundConfig<'a> {
    /// Create a config for GraphQL playground.
    pub fn new(endpoint: &'a str) -> Self {
        Self {
            endpoint,
            subscription_endpoint: None,
            headers: Default::default(),
            settings: Default::default(),
        }
    }

    /// Set subscription endpoint, for example: `ws://localhost:8000`.
    #[must_use]
    pub fn subscription_endpoint(mut self, endpoint: &'a str) -> Self {
        self.subscription_endpoint = Some(endpoint);
        self
    }

    /// Set HTTP header for per query.
    #[must_use]
    pub fn with_header(mut self, name: &'a str, value: &'a str) -> Self {
        if let Some(headers) = &mut self.headers {
            headers.insert(name, value);
        } else {
            let mut headers = HashMap::new();
            headers.insert(name, value);
            self.headers = Some(headers);
        }
        self
    }

    /// Set Playground setting for per query.
    ///
    /// ```
    /// # use dynaql::Value;
    /// # use dynaql::http::GraphQLPlaygroundConfig;
    /// GraphQLPlaygroundConfig::new("/api/graphql")
    ///     .with_setting("setting", false)
    ///     .with_setting("other", Value::Null);
    /// ```
    #[must_use]
    pub fn with_setting(mut self, name: &'a str, value: impl Into<Value>) -> Self {
        let value = value.into();

        if let Some(settings) = &mut self.settings {
            settings.insert(name, value);
        } else {
            let mut settings = HashMap::new();
            settings.insert(name, value);
            self.settings = Some(settings);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_with_setting_can_use_any_json_value() {
        let settings = GraphQLPlaygroundConfig::new("")
            .with_setting("string", "string")
            .with_setting("bool", false)
            .with_setting("number", 10)
            .with_setting("null", Value::Null)
            .with_setting("array", Vec::from([1, 2, 3]))
            .with_setting("object", IndexMap::new());

        let json = serde_json::to_value(settings).unwrap();
        let settings = json["settings"].as_object().unwrap();

        assert!(settings["string"].as_str().is_some());
        assert!(settings["bool"].as_bool().is_some());
        assert!(settings["number"].as_u64().is_some());
        assert!(settings["null"].as_null().is_some());
        assert!(settings["array"].as_array().is_some());
        assert!(settings["object"].as_object().is_some());
    }
}
