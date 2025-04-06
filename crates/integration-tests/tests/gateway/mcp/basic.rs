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

        let stream = engine.mcp("/mcp").await;
        stream.server_info()
    });

    insta::assert_json_snapshot!(&server_info, @r#"
    {
      "result": {
        "protocolVersion": "2024-11-05",
        "capabilities": {
          "tools": {}
        },
        "serverInfo": {
          "name": "rmcp",
          "version": "0.1.5"
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

        let mut stream = engine.mcp("/mcp").await;
        stream.list_tools().await
    });

    insta::assert_json_snapshot!(&tools, @r##"
    {
      "result": {
        "tools": [
          {
            "name": "introspect",
            "description": "Provides detailed information about GraphQL type definition. Always use `search` first to identify relevant fields before if information on a specific type was not explicitly requested. If you're not certain whether a field exist on a type, always use this tool first.",
            "inputSchema": {
              "$schema": "http://json-schema.org/draft-07/schema#",
              "title": "IntrospectionParameters",
              "type": "object",
              "required": [
                "types"
              ],
              "properties": {
                "types": {
                  "type": "array",
                  "items": {
                    "type": "string"
                  }
                }
              }
            }
          },
          {
            "name": "search",
            "description": "Search for relevant fields to use in a GraphQL query. Each matching GraphQL field will have all of its ancestor fields up to a root type. Ancestors are provided in depth order, so the first one is a field a on the root type. Always use `introspect` tool afterwards to get more informations on types if you need additional fields.",
            "inputSchema": {
              "$schema": "http://json-schema.org/draft-07/schema#",
              "title": "SearchParameters",
              "type": "object",
              "required": [
                "keywords"
              ],
              "properties": {
                "keywords": {
                  "type": "array",
                  "items": {
                    "type": "string"
                  }
                }
              }
            }
          },
          {
            "name": "verify",
            "description": "Validates a GraphQL request. A list of errors is returned if there are any.",
            "inputSchema": {
              "$schema": "http://json-schema.org/draft-07/schema#",
              "title": "Request",
              "type": "object",
              "required": [
                "query",
                "variables"
              ],
              "properties": {
                "query": {
                  "type": "string"
                },
                "variables": {
                  "type": "object",
                  "additionalProperties": true
                }
              }
            }
          },
          {
            "name": "execute",
            "description": "Executes a GraphQL request and returns the response. Always validate with `verify` tool before executing a request.",
            "inputSchema": {
              "$schema": "http://json-schema.org/draft-07/schema#",
              "title": "Request",
              "type": "object",
              "required": [
                "query",
                "variables"
              ],
              "properties": {
                "query": {
                  "type": "string"
                },
                "variables": {
                  "type": "object",
                  "additionalProperties": true
                }
              }
            }
          }
        ]
      }
    }
    "##);
}
