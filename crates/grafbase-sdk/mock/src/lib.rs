//! Provides a dynamic GraphQL schema and subgraph implementation that can be built and executed at runtime.
//!
//! This crate allows creating GraphQL schemas dynamically from SDL (Schema Definition Language) strings
//! and executing queries against them. It also provides functionality for running mock GraphQL servers
//! using these dynamic schemas.

#![deny(missing_docs)]

mod builder;
mod entity_resolver;
mod resolver;
mod server;

use std::{
    hash::{DefaultHasher, Hasher as _},
    sync::Arc,
};

pub use async_graphql::dynamic::ResolverContext;
pub use builder::GraphqlSubgraphBuilder;
pub use entity_resolver::EntityResolverContext;
pub use server::MockGraphQlServer;

/// A dynamic subgraph implementation that can be started as a mock GraphQL server.
#[derive(Debug, Clone)]
pub struct GraphqlSubgraph {
    executable_schema: async_graphql::dynamic::Schema,
    schema: String,
    name: String,
}

impl GraphqlSubgraph {
    /// Creates a builder for constructing a new dynamic subgraph schema from SDL.
    ///
    /// # Arguments
    ///
    /// * `sdl` - GraphQL schema definition language string to build from
    pub fn with_schema(sdl: impl AsRef<str>) -> GraphqlSubgraphBuilder {
        let sdl = sdl.as_ref();
        GraphqlSubgraphBuilder::new(sdl.to_string(), anonymous_name(sdl))
    }

    /// Starts this subgraph as a mock GraphQL server.
    ///
    /// Returns a handle to the running server that can be used to stop it.
    pub async fn start(self) -> MockGraphQlServer {
        MockGraphQlServer::new(self.name, Arc::new((self.executable_schema, self.schema))).await
    }

    /// Returns the GraphQL schema in SDL (Schema Definition Language)
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the name of this subgraph
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A subgraph that only contains extension definitions. We do not spawn a GraphQL server for this subgraph.
#[derive(Debug, Clone)]
pub struct VirtualSubgraph {
    schema: String,
    name: String,
}

impl VirtualSubgraph {
    /// Creates a new virtual subgraph with the given SDL and name.
    pub fn new(name: &str, schema: &str) -> Self {
        VirtualSubgraph {
            name: name.to_string(),
            schema: schema.to_string(),
        }
    }

    /// Returns the GraphQL schema in SDL (Schema Definition Language)
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the name of this subgraph
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A mock subgraph that can either be a full dynamic GraphQL service or just extension definitions.
#[derive(Debug, Clone)]
pub enum Subgraph {
    /// A full dynamic subgraph that can be started as a GraphQL server
    Graphql(GraphqlSubgraph),
    /// A subgraph that only contains extension definitions and is not started as a server
    Virtual(VirtualSubgraph),
}

impl From<GraphqlSubgraph> for Subgraph {
    fn from(subgraph: GraphqlSubgraph) -> Self {
        Subgraph::Graphql(subgraph)
    }
}

impl From<GraphqlSubgraphBuilder> for Subgraph {
    fn from(builder: GraphqlSubgraphBuilder) -> Self {
        builder.build().into()
    }
}

impl From<VirtualSubgraph> for Subgraph {
    fn from(subgraph: VirtualSubgraph) -> Self {
        Subgraph::Virtual(subgraph)
    }
}

impl<T: Into<Subgraph>> From<(String, T)> for Subgraph {
    fn from((name, subgraph): (String, T)) -> Self {
        (name.as_str(), subgraph).into()
    }
}

impl<T: Into<Subgraph>> From<(&str, T)> for Subgraph {
    fn from((name, subgraph): (&str, T)) -> Self {
        match subgraph.into() {
            Subgraph::Graphql(mut graphql_subgraph) => {
                graphql_subgraph.name = name.to_string();
                Subgraph::Graphql(graphql_subgraph)
            }
            Subgraph::Virtual(mut virtual_subgraph) => {
                virtual_subgraph.name = name.to_string();
                Subgraph::Virtual(virtual_subgraph)
            }
        }
    }
}

impl From<String> for Subgraph {
    fn from(sdl: String) -> Self {
        sdl.as_str().into()
    }
}

impl From<&str> for Subgraph {
    fn from(schema: &str) -> Self {
        Subgraph::Virtual(VirtualSubgraph::new(&anonymous_name(schema), schema))
    }
}

fn anonymous_name(schema: &str) -> String {
    let mut hasher = DefaultHasher::default();
    hasher.write(schema.as_bytes());
    format!("anonymous{:X}", hasher.finish())
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[tokio::test]
    async fn echo_header() {
        let subgraph = GraphqlSubgraph::with_schema(r#"type Query { header(name: String!): String }"#)
            .with_resolver("Query", "header", |ctx: ResolverContext<'_>| {
                ctx.data_unchecked::<http::HeaderMap>()
                    .get(ctx.args.get("name").unwrap().string().unwrap())
                    .map(|value| value.to_str().unwrap().to_owned().into())
            })
            .build();

        let server = subgraph.start().await;

        let response = reqwest::Client::new()
            .post(server.url().clone())
            .body(serde_json::to_vec(&serde_json::json!({"query":r#"query { header(name: "hi") }"#})).unwrap())
            .header("hi", "John")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let response: serde_json::Value = serde_json::from_str(&response).unwrap_or_else(|err| {
            panic!(
                "Failed to parse response as JSON: {}\nResponse body:\n{}",
                err, response
            )
        });
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "John"
          }
        }
        "#);
    }
}
