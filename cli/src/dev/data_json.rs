//! The local app expects a `data.json` endpoint located next to its `index.html`. That data has to be populated on startup, and then reloaded anytime there is a change in schemas.

use std::sync::Arc;

use super::subgraphs::CachedSubgraph;
use chrono::{DateTime, Utc};
use serde::{Serialize, Serializer};

/// The format of the data.json endpoint located next to the index.html.
#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) struct DataJson<'a> {
    #[serde(rename = "updatedAt")]
    pub(super) updated_at: DateTime<Utc>,
    pub(super) graphql_api_url: &'a str,
    pub(super) mcp_server_url: Option<&'a str>,
    pub(super) schemas: &'a Schemas,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub(crate) enum Schemas {
    Data {
        api_schema: Option<String>,
        federated_schema: Option<String>,
        #[serde(serialize_with = "serialize_cached_subgraph")]
        subgraphs: Vec<Arc<CachedSubgraph>>,
    },
    Errors {
        errors: Vec<Error>,
    },
}

impl Default for Schemas {
    fn default() -> Self {
        Schemas::Data {
            api_schema: None,
            federated_schema: None,
            subgraphs: Vec::new(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Error {
    pub(super) message: String,
    pub(super) severity: &'static str,
}

fn serialize_cached_subgraph<S>(subgraphs: &[Arc<CachedSubgraph>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Subgraph<'a> {
        name: &'a str,
        sdl: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        owners: Option<Vec<Owner<'a>>>,
    }

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Owner<'a> {
        name: &'a str,
    }

    serializer.collect_seq(subgraphs.iter().map(|subgraph| {
        Subgraph {
            name: &subgraph.name,
            sdl: &subgraph.sdl,
            owners: subgraph
                .owners
                .as_ref()
                .map(|owners| owners.iter().map(|owner| Owner { name: &owner.name }).collect()),
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_json_serialization_with_data() {
        let updated_at = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let subgraph1 = Arc::new(CachedSubgraph {
            name: "users".to_string(),
            sdl: "type User { id: ID! name: String! }".to_string(),
            url: Some("http://localhost:5001/graphql".to_string()),
            owners: Some(vec![
                crate::dev::subgraphs::SubgraphOwner {
                    name: "backend-team".to_string(),
                },
                crate::dev::subgraphs::SubgraphOwner {
                    name: "api-team".to_string(),
                },
            ]),
        });

        let subgraph2 = Arc::new(CachedSubgraph {
            name: "products".to_string(),
            sdl: "type Product { id: ID! price: Float! }".to_string(),
            url: None,
            owners: None,
        });

        let schemas = Schemas::Data {
            api_schema: Some("type Query { hello: String }".to_string()),
            federated_schema: Some("type Query { hello: String user: User }".to_string()),
            subgraphs: vec![subgraph1, subgraph2],
        };

        let data_json = DataJson {
            updated_at,
            graphql_api_url: "http://localhost:4000/graphql",
            mcp_server_url: Some("http://localhost:4001/mcp"),
            schemas: &schemas,
        };

        insta::assert_json_snapshot!(data_json, @r#"
        {
          "updatedAt": "2024-01-15T10:30:00Z",
          "GRAPHQL_API_URL": "http://localhost:4000/graphql",
          "MCP_SERVER_URL": "http://localhost:4001/mcp",
          "SCHEMAS": {
            "api_schema": "type Query { hello: String }",
            "federated_schema": "type Query { hello: String user: User }",
            "subgraphs": [
              {
                "name": "users",
                "sdl": "type User { id: ID! name: String! }",
                "owners": [
                  {
                    "name": "backend-team"
                  },
                  {
                    "name": "api-team"
                  }
                ]
              },
              {
                "name": "products",
                "sdl": "type Product { id: ID! price: Float! }"
              }
            ]
          }
        }
        "#);
    }

    #[test]
    fn test_data_json_serialization_with_errors() {
        let updated_at = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let schemas = Schemas::Errors {
            errors: vec![
                Error {
                    message: "Failed to parse schema".to_string(),
                    severity: "error",
                },
                Error {
                    message: "Deprecated field usage".to_string(),
                    severity: "warning",
                },
            ],
        };

        let data_json = DataJson {
            updated_at,
            graphql_api_url: "http://localhost:4000/graphql",
            mcp_server_url: None,
            schemas: &schemas,
        };

        insta::assert_json_snapshot!(data_json, @r#"
        {
          "updatedAt": "2024-01-15T10:30:00Z",
          "GRAPHQL_API_URL": "http://localhost:4000/graphql",
          "MCP_SERVER_URL": null,
          "SCHEMAS": {
            "errors": [
              {
                "message": "Failed to parse schema",
                "severity": "error"
              },
              {
                "message": "Deprecated field usage",
                "severity": "warning"
              }
            ]
          }
        }
        "#);
    }

    #[test]
    fn test_data_json_serialization_empty_data() {
        let updated_at = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let schemas = Schemas::default();

        let data_json = DataJson {
            updated_at,
            graphql_api_url: "http://localhost:4000/graphql",
            mcp_server_url: None,
            schemas: &schemas,
        };

        insta::assert_json_snapshot!(data_json, @r#"
        {
          "updatedAt": "2024-01-15T10:30:00Z",
          "GRAPHQL_API_URL": "http://localhost:4000/graphql",
          "MCP_SERVER_URL": null,
          "SCHEMAS": {
            "api_schema": null,
            "federated_schema": null,
            "subgraphs": []
          }
        }
        "#);
    }
}
