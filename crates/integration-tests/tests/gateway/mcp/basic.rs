use integration_tests::{gateway::Gateway, runtime};

#[test]
fn server_info() {
    let server_info = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type User {
                    id: ID!
                    name: String!
                }

                type Query {
                    user: User
                }
            "#,
            )
            .with_toml_config(
                r#"
            [mcp]
            enabled = true
            "#,
            )
            .build()
            .await;

        let stream = engine.mcp_http("/mcp").await;
        stream.server_info()
    });

    insta::assert_json_snapshot!(&server_info, @r#"
    {
      "result": {
        "protocolVersion": "2025-03-26",
        "capabilities": {
          "tools": {}
        },
        "serverInfo": {
          "name": "rmcp",
          "version": "0.3.1"
        },
        "instructions": null
      }
    }
    "#);
}

#[test]
fn list_tools() {
    let tools = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type User {
                    id: ID!
                    name: String!
                }

                type Query {
                    user: User
                }
            "#,
            )
            .with_toml_config(
                r#"
            [mcp]
            enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp_http("/mcp").await;
        stream.list_tools().await
    });

    insta::assert_json_snapshot!(&tools, @r#"
    {
      "result": {
        "tools": [
          {
            "name": "introspect",
            "description": "Provide the complete GraphQL SDL for the requested types. Always use `search` first to identify relevant fields before if information on a specific type was not explicitly requested. Continue using this tool until you have the definition of all nested types you intend to query.",
            "inputSchema": {
              "$schema": "https://json-schema.org/draft/2020-12/schema",
              "title": "IntrospectionParameters",
              "type": "object",
              "properties": {
                "types": {
                  "type": "array",
                  "items": {
                    "type": "string"
                  }
                }
              },
              "required": [
                "types"
              ]
            },
            "annotations": {
              "readOnlyHint": true
            }
          },
          {
            "name": "search",
            "description": "Search for relevant fields to use in a GraphQL query. A list of matching fields with their score is returned with partial GraphQL SDL indicating how to query them. Use `introspect` tool to request additional information on children field types if necessary to refine the selection set.",
            "inputSchema": {
              "$schema": "https://json-schema.org/draft/2020-12/schema",
              "title": "SearchParameters",
              "type": "object",
              "properties": {
                "keywords": {
                  "type": "array",
                  "items": {
                    "type": "string"
                  }
                }
              },
              "required": [
                "keywords"
              ]
            },
            "annotations": {
              "readOnlyHint": true
            }
          },
          {
            "name": "execute",
            "description": "Executes a GraphQL request. Additional GraphQL SDL may be provided upon errors.",
            "inputSchema": {
              "$schema": "https://json-schema.org/draft/2020-12/schema",
              "title": "Request",
              "type": "object",
              "properties": {
                "query": {
                  "type": "string"
                },
                "variables": {
                  "type": "object",
                  "additionalProperties": true,
                  "default": {}
                }
              },
              "required": [
                "query"
              ]
            },
            "annotations": {
              "destructiveHint": true,
              "openWorldHint": true
            }
          }
        ]
      }
    }
    "#);
}
