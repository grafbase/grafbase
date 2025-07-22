use std::time::Duration;

use duct::cmd;
use graphql_mocks::Subgraph as _;
use rand::random;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::StreamableHttpClientTransport,
};
use tokio::time::timeout;

use crate::cargo_bin;

#[tokio::test]
async fn test_mcp() {
    let subgraph = graphql_mocks::EchoSchema::default().start().await;

    // Pick a port number in the dynamic range.
    let port = random::<u16>() | 0xc000;

    let handle = cmd(
        cargo_bin("grafbase"),
        &[
            "mcp",
            subgraph.url().as_str(),
            "--port",
            &port.to_string(),
            "--transport",
            "streaming-http",
        ],
    )
    .unchecked()
    .start()
    .unwrap();

    let url = format!("http://127.0.0.1:{port}/mcp").parse::<url::Url>().unwrap();
    // Wait for the MCP server to start.
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if reqwest::Client::new()
            .get(url.clone())
            .header(http::header::ACCEPT, "text/event-stream")
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .is_err()
        {
            continue;
        };
    }

    let transport = StreamableHttpClientTransport::from_uri(url.as_str());
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "test sse client".to_string(),
            version: "0.0.1".to_string(),
        },
    };

    let client = client_info.serve(transport).await.unwrap();

    // Initialize
    let server_info = client.peer_info();
    insta::assert_json_snapshot!(server_info, @r#"
    {
      "protocolVersion": "2025-03-26",
      "capabilities": {
        "tools": {}
      },
      "serverInfo": {
        "name": "rmcp",
        "version": "0.3.0"
      }
    }
    "#);

    // List tools
    let tools = client.list_tools(Default::default()).await.unwrap();
    insta::assert_json_snapshot!(tools, @r##"
    {
      "tools": [
        {
          "name": "introspect",
          "description": "Provide the complete GraphQL SDL for the requested types. Always use `search` first to identify relevant fields before if information on a specific type was not explicitly requested. Continue using this tool until you have the definition of all nested types you intend to query.",
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
          },
          "annotations": {
            "readOnlyHint": true
          }
        },
        {
          "name": "search",
          "description": "Search for relevant fields to use in a GraphQL query. A list of matching fields with their score is returned with partial GraphQL SDL indicating how to query them. Use `introspect` tool to request additional information on children field types if necessary to refine the selection set.",
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
          },
          "annotations": {
            "readOnlyHint": true
          }
        },
        {
          "name": "execute",
          "description": "Executes a GraphQL request. Additional GraphQL SDL may be provided upon errors.",
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
          },
          "annotations": {
            "destructiveHint": true,
            "openWorldHint": true
          }
        }
      ]
    }
    "##);

    let tool_result = timeout(
        Duration::from_secs(20),
        client.call_tool(CallToolRequestParam {
            name: "search".into(),
            arguments: serde_json::json!({"keywords": ["header"]}).as_object().cloned(),
        }),
    )
    .await
    .unwrap()
    .unwrap();
    insta::assert_json_snapshot!(tool_result, @r##"
    {
      "content": [
        {
          "type": "text",
          "text": "# Incomplete fields\ntype Query {\n  headers: [Header!]!\n  responseHeader(name: String!, value: String!): Boolean\n  header(name: String!): String\n}\n\ntype Header {\n  name: String!\n  value: String!\n}\n\n"
        }
      ]
    }
    "##);

    let tool_result = timeout(
        Duration::from_secs(20),
        client.call_tool(CallToolRequestParam {
            name: "execute".into(),
            arguments: serde_json::json!({"query": "query { __typename }"})
                .as_object()
                .cloned(),
        }),
    )
    .await
    .unwrap()
    .unwrap();
    insta::assert_json_snapshot!(tool_result, @r#"
    {
      "content": [
        {
          "type": "text",
          "text": "{\"data\":{\"__typename\":\"Query\"}}"
        }
      ]
    }
    "#);

    client.cancel().await.unwrap();
    handle.kill().unwrap();
}
