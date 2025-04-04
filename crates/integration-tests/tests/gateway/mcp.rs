use graphql_mocks::dynamic::DynamicSchema;
use indoc::indoc;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

static CONFIG: &str = r#"
    [graph]
    introspection = true

    [mcp]
    enabled = true
    name = "Test MCP Server"
    instructions = "This is a test MCP server with no mutations."
"#;

static MUT_CONFIG: &str = r#"
    [graph]
    introspection = true

    [mcp]
    enabled = true
    enable_mutations = true
    name = "Test MCP Server"
    instructions = "This is a test MCP server with mutations."
"#;

#[test]
fn server_info_no_mutations() {
    let subgraph = indoc! {r#"
        type User {
            id: ID!
            name: String!
        }

        type Query {
            user: User
        }
    "#};

    let resolver = json!({
        "id": "1",
        "name": "Alice"
    });

    let subgraph = DynamicSchema::builder(subgraph)
        .with_resolver("Query", "user", resolver)
        .into_subgraph("a");

    let server_info = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph)
            .with_toml_config(CONFIG)
            .build()
            .await;

        let stream = engine.mcp("/mcp").await;
        stream.server_info()
    });

    insta::assert_snapshot!(&server_info.instructions, @r"
    This is a test MCP server with no mutations.

    This is a GraphQL server that provides tools to access certain selected operations.
    The operation requires certain arguments, and always a selection. You can construct the
    correct selection by first looking into the description of the query tool, finding the
    return type, and then calling the introspect-type tool with the name of the type.

    This tool will provide you all the information to construct a correct selection for the query. You always have to
    call the introspect-type tool first, and only after that you can call the correct query tool.

    Queries are suffixed with Query and mutations with Mutation.
    ");
}

#[test]
fn server_info_mutations() {
    let subgraph = indoc! {r#"
        type User {
            id: ID!
            name: String!
        }

        type Query {
            user: User
        }
    "#};

    let resolver = json!({
        "id": "1",
        "name": "Alice"
    });

    let subgraph = DynamicSchema::builder(subgraph)
        .with_resolver("Query", "user", resolver)
        .into_subgraph("a");

    let server_info = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph)
            .with_toml_config(MUT_CONFIG)
            .build()
            .await;

        let stream = engine.mcp("/mcp").await;
        stream.server_info()
    });

    insta::assert_snapshot!(&server_info.instructions, @r"
    This is a test MCP server with mutations.

    This is a GraphQL server that provides tools to access certain selected operations.
    The operation requires certain arguments, and always a selection. You can construct the
    correct selection by first looking into the description of the query tool, finding the
    return type, and then calling the introspect-type tool with the name of the type.

    This tool will provide you all the information to construct a correct selection for the query. You always have to
    call the introspect-type tool first, and only after that you can call the correct query tool.

    Queries are suffixed with Query and mutations with Mutation.
    ");
}

#[test]
fn list_no_mutations() {
    let subgraph = indoc! {r#"
        type User {
            id: ID!
            name: String!
        }

        input FindUser {
            id: ID!
        }

        input UserCreateInput {
            name: String!
        }

        type Query {
            users: [User!]!
            user(id: ID!): User
            otherUser(filter: FindUser!): User
        }

        type Mutation {
            createUser(input: UserCreateInput!): User!
        }
    "#};

    let resolver = json!({
        "id": "1",
        "name": "Alice"
    });

    let subgraph = DynamicSchema::builder(subgraph)
        .with_resolver("Query", "user", resolver)
        .into_subgraph("a");

    let tools = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph)
            .with_toml_config(CONFIG)
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;
        stream.list_tools().await.unwrap()
    });

    insta::assert_json_snapshot!(&tools, @r#"
    {
      "tools": [
        {
          "name": "introspect-type",
          "description": "Use this tool before executing any query tools. This tool provides information how to construct\na selection for a specific query. You first select a query you want to execute, see its return\ntype from the description, use this tool to get information about the type and _only then_ you\ncall the query tool with the correct selection set and arguments.\n\nRemember, THIS IS IMPORTANT: you can ONLY select the fields that are returned by this query. There\nare no other fields that can be selected.\n\nYou don't need to use this API for scalar types, input types or enum values, but only when you need\nto build a selection set for a query or mutation. Use the returned value to build a selection set.\nIf a field of a type is either object, interface, or union, you can call this tool repeatedly with\nthe name of the type to introspect its fields.\n\nIf the type is an object, it will have fields defined that you can use as the selection.\nThe fields might have arguments, and if they are required, you need to provide them in the\nselection set.\n\nIf the type is an interface or a union, it will have only the fields that are defined in the\ninterface. You can check the possibleTypes of the interface to see what fields you can use for\neach possible type. Remember to use fragment syntax to select fields from the possible types.\n",
          "inputSchema": {
            "type": "object",
            "properties": {
              "name": {
                "type": "string",
                "description": "The name of the type, interface, or union to introspect."
              }
            },
            "required": [
              "name"
            ]
          }
        },
        {
          "name": "query/otherUser",
          "description": "This query returns a object named User. It is a nullable item.\nProvide a GraphQL selection set for the query (e.g., '{ id name }').\n\nYou must determine the fields of the type by calling the `introspect-type` tool first in\nthis MCP server. It will return the needed information for you to build the selection.\n\nDo NOT call this query before running introspection and knowing exactly what fields you can select.\n",
          "inputSchema": {
            "type": "object",
            "properties": {
              "filter": {
                "type": "object",
                "properties": {
                  "id": {
                    "type": "string",
                    "description": ""
                  }
                },
                "required": [
                  "id"
                ],
                "description": ""
              },
              "__selection": {
                "type": "string",
                "description": "This value is written in the syntax of a GraphQL selection set. Example: '{ id name }'.\n\nBefore generating this field, call the `introspect-type` tool with type name: User\n\nThe `introspect-type` tool returns with a GraphQL introspection response format, and tells you\nif the return type is an object, a union or an interface.\n\nIf it's an object, you have to select at least one field from the type.\n\nIf it's a union, you can select any fields from any of the possible types. Remember to use fragment spreads,\nif needed.\n\nIf it's an interface, you can select any fields from any of the possible types, or with fields\nfrom the interface itself. Remember to use fragment spreads, if needed.\n"
              }
            },
            "required": [
              "filter",
              "__selection"
            ]
          }
        },
        {
          "name": "query/user",
          "description": "This query returns a object named User. It is a nullable item.\nProvide a GraphQL selection set for the query (e.g., '{ id name }').\n\nYou must determine the fields of the type by calling the `introspect-type` tool first in\nthis MCP server. It will return the needed information for you to build the selection.\n\nDo NOT call this query before running introspection and knowing exactly what fields you can select.\n",
          "inputSchema": {
            "type": "object",
            "properties": {
              "id": {
                "type": "string",
                "description": ""
              },
              "__selection": {
                "type": "string",
                "description": "This value is written in the syntax of a GraphQL selection set. Example: '{ id name }'.\n\nBefore generating this field, call the `introspect-type` tool with type name: User\n\nThe `introspect-type` tool returns with a GraphQL introspection response format, and tells you\nif the return type is an object, a union or an interface.\n\nIf it's an object, you have to select at least one field from the type.\n\nIf it's a union, you can select any fields from any of the possible types. Remember to use fragment spreads,\nif needed.\n\nIf it's an interface, you can select any fields from any of the possible types, or with fields\nfrom the interface itself. Remember to use fragment spreads, if needed.\n"
              }
            },
            "required": [
              "id",
              "__selection"
            ]
          }
        },
        {
          "name": "query/users",
          "description": "This query returns a object named User. It is a non-nullable array of non-nullable items.\nProvide a GraphQL selection set for the query (e.g., '{ id name }').\n\nYou must determine the fields of the type by calling the `introspect-type` tool first in\nthis MCP server. It will return the needed information for you to build the selection.\n\nDo NOT call this query before running introspection and knowing exactly what fields you can select.\n",
          "inputSchema": {
            "type": "object",
            "properties": {
              "__selection": {
                "type": "string",
                "description": "This value is written in the syntax of a GraphQL selection set. Example: '{ id name }'.\n\nBefore generating this field, call the `introspect-type` tool with type name: User\n\nThe `introspect-type` tool returns with a GraphQL introspection response format, and tells you\nif the return type is an object, a union or an interface.\n\nIf it's an object, you have to select at least one field from the type.\n\nIf it's a union, you can select any fields from any of the possible types. Remember to use fragment spreads,\nif needed.\n\nIf it's an interface, you can select any fields from any of the possible types, or with fields\nfrom the interface itself. Remember to use fragment spreads, if needed.\n"
              }
            },
            "required": [
              "__selection"
            ]
          }
        }
      ]
    }
    "#);
}

#[test]
fn list_with_mutations() {
    let subgraph = indoc! {r#"
        type User {
            id: ID!
            name: String!
        }

        input FindUser {
            id: ID!
        }

        input UserCreateInput {
            name: String!
        }

        type Query {
            users: [User!]!
            user(id: ID!): User
            otherUser(filter: FindUser!): User
        }

        type Mutation {
            createUser(input: UserCreateInput!): User!
        }
    "#};

    let resolver = json!({
        "id": "1",
        "name": "Alice"
    });

    let subgraph = DynamicSchema::builder(subgraph)
        .with_resolver("Query", "user", resolver)
        .into_subgraph("a");

    let tools = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph)
            .with_toml_config(MUT_CONFIG)
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;
        stream.list_tools().await.unwrap()
    });

    insta::assert_json_snapshot!(&tools, @r#"
    {
      "tools": [
        {
          "name": "introspect-type",
          "description": "Use this tool before executing any query tools. This tool provides information how to construct\na selection for a specific query. You first select a query you want to execute, see its return\ntype from the description, use this tool to get information about the type and _only then_ you\ncall the query tool with the correct selection set and arguments.\n\nRemember, THIS IS IMPORTANT: you can ONLY select the fields that are returned by this query. There\nare no other fields that can be selected.\n\nYou don't need to use this API for scalar types, input types or enum values, but only when you need\nto build a selection set for a query or mutation. Use the returned value to build a selection set.\nIf a field of a type is either object, interface, or union, you can call this tool repeatedly with\nthe name of the type to introspect its fields.\n\nIf the type is an object, it will have fields defined that you can use as the selection.\nThe fields might have arguments, and if they are required, you need to provide them in the\nselection set.\n\nIf the type is an interface or a union, it will have only the fields that are defined in the\ninterface. You can check the possibleTypes of the interface to see what fields you can use for\neach possible type. Remember to use fragment syntax to select fields from the possible types.\n",
          "inputSchema": {
            "type": "object",
            "properties": {
              "name": {
                "type": "string",
                "description": "The name of the type, interface, or union to introspect."
              }
            },
            "required": [
              "name"
            ]
          }
        },
        {
          "name": "query/otherUser",
          "description": "This query returns a object named User. It is a nullable item.\nProvide a GraphQL selection set for the query (e.g., '{ id name }').\n\nYou must determine the fields of the type by calling the `introspect-type` tool first in\nthis MCP server. It will return the needed information for you to build the selection.\n\nDo NOT call this query before running introspection and knowing exactly what fields you can select.\n",
          "inputSchema": {
            "type": "object",
            "properties": {
              "filter": {
                "type": "object",
                "properties": {
                  "id": {
                    "type": "string",
                    "description": ""
                  }
                },
                "required": [
                  "id"
                ],
                "description": ""
              },
              "__selection": {
                "type": "string",
                "description": "This value is written in the syntax of a GraphQL selection set. Example: '{ id name }'.\n\nBefore generating this field, call the `introspect-type` tool with type name: User\n\nThe `introspect-type` tool returns with a GraphQL introspection response format, and tells you\nif the return type is an object, a union or an interface.\n\nIf it's an object, you have to select at least one field from the type.\n\nIf it's a union, you can select any fields from any of the possible types. Remember to use fragment spreads,\nif needed.\n\nIf it's an interface, you can select any fields from any of the possible types, or with fields\nfrom the interface itself. Remember to use fragment spreads, if needed.\n"
              }
            },
            "required": [
              "filter",
              "__selection"
            ]
          }
        },
        {
          "name": "query/user",
          "description": "This query returns a object named User. It is a nullable item.\nProvide a GraphQL selection set for the query (e.g., '{ id name }').\n\nYou must determine the fields of the type by calling the `introspect-type` tool first in\nthis MCP server. It will return the needed information for you to build the selection.\n\nDo NOT call this query before running introspection and knowing exactly what fields you can select.\n",
          "inputSchema": {
            "type": "object",
            "properties": {
              "id": {
                "type": "string",
                "description": ""
              },
              "__selection": {
                "type": "string",
                "description": "This value is written in the syntax of a GraphQL selection set. Example: '{ id name }'.\n\nBefore generating this field, call the `introspect-type` tool with type name: User\n\nThe `introspect-type` tool returns with a GraphQL introspection response format, and tells you\nif the return type is an object, a union or an interface.\n\nIf it's an object, you have to select at least one field from the type.\n\nIf it's a union, you can select any fields from any of the possible types. Remember to use fragment spreads,\nif needed.\n\nIf it's an interface, you can select any fields from any of the possible types, or with fields\nfrom the interface itself. Remember to use fragment spreads, if needed.\n"
              }
            },
            "required": [
              "id",
              "__selection"
            ]
          }
        },
        {
          "name": "query/users",
          "description": "This query returns a object named User. It is a non-nullable array of non-nullable items.\nProvide a GraphQL selection set for the query (e.g., '{ id name }').\n\nYou must determine the fields of the type by calling the `introspect-type` tool first in\nthis MCP server. It will return the needed information for you to build the selection.\n\nDo NOT call this query before running introspection and knowing exactly what fields you can select.\n",
          "inputSchema": {
            "type": "object",
            "properties": {
              "__selection": {
                "type": "string",
                "description": "This value is written in the syntax of a GraphQL selection set. Example: '{ id name }'.\n\nBefore generating this field, call the `introspect-type` tool with type name: User\n\nThe `introspect-type` tool returns with a GraphQL introspection response format, and tells you\nif the return type is an object, a union or an interface.\n\nIf it's an object, you have to select at least one field from the type.\n\nIf it's a union, you can select any fields from any of the possible types. Remember to use fragment spreads,\nif needed.\n\nIf it's an interface, you can select any fields from any of the possible types, or with fields\nfrom the interface itself. Remember to use fragment spreads, if needed.\n"
              }
            },
            "required": [
              "__selection"
            ]
          }
        },
        {
          "name": "mutation/createUser",
          "description": "This mutation returns a object named User. It is a non-nullable item.\nProvide a GraphQL selection set for the query (e.g., '{ id name }').\n\nYou must determine the fields of the type by calling the `introspect-type` tool first in\nthis MCP server. It will return the needed information for you to build the selection.\n\nDo NOT call this mutation before running introspection and knowing exactly what fields you can select.\n",
          "inputSchema": {
            "type": "object",
            "properties": {
              "input": {
                "type": "object",
                "properties": {
                  "name": {
                    "type": "string",
                    "description": ""
                  }
                },
                "required": [
                  "name"
                ],
                "description": ""
              },
              "__selection": {
                "type": "string",
                "description": "This value is written in the syntax of a GraphQL selection set. Example: '{ id name }'.\n\nBefore generating this field, call the `introspect-type` tool with type name: User\n\nThe `introspect-type` tool returns with a GraphQL introspection response format, and tells you\nif the return type is an object, a union or an interface.\n\nIf it's an object, you have to select at least one field from the type.\n\nIf it's a union, you can select any fields from any of the possible types. Remember to use fragment spreads,\nif needed.\n\nIf it's an interface, you can select any fields from any of the possible types, or with fields\nfrom the interface itself. Remember to use fragment spreads, if needed.\n"
              }
            },
            "required": [
              "input",
              "__selection"
            ]
          }
        }
      ]
    }
    "#);
}

#[test]
fn introspect_type() {
    let subgraph = indoc! {r#"
        type User {
            id: ID!
            name: String!
        }

        type Query {
            user: User
        }
    "#};

    let resolver = json!({
        "id": "1",
        "name": "Alice"
    });

    let subgraph = DynamicSchema::builder(subgraph)
        .with_resolver("Query", "user", resolver)
        .into_subgraph("a");

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph)
            .with_toml_config(CONFIG)
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        stream
            .call_tool("introspect-type", json!({"name": "User"}))
            .await
            .unwrap()
    });

    insta::assert_json_snapshot!(&response, @r#"
    {
      "name": "User",
      "kind": "OBJECT",
      "fields": [
        {
          "name": "id",
          "type": {
            "kind": "NON_NULL",
            "ofType": {
              "name": "ID",
              "kind": "SCALAR"
            }
          },
          "isDeprecated": false,
          "args": []
        },
        {
          "name": "name",
          "type": {
            "kind": "NON_NULL",
            "ofType": {
              "name": "String",
              "kind": "SCALAR"
            }
          },
          "isDeprecated": false,
          "args": []
        }
      ],
      "interfaces": []
    }
    "#);
}

#[test]
fn run_query_no_params() {
    let subgraph = indoc! {r#"
        type User {
            id: ID!
            name: String!
        }

        type Query {
            user: User
        }
    "#};

    let resolver = json!({
        "id": "1",
        "name": "Alice"
    });

    let subgraph = DynamicSchema::builder(subgraph)
        .with_resolver("Query", "user", resolver)
        .into_subgraph("a");

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph)
            .with_toml_config(CONFIG)
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let args = json!({
            "__selection": "{ id name }"
        });

        stream.call_tool("query/user", args).await.unwrap()
    });

    insta::assert_json_snapshot!(&response, @r#"
    {
      "id": "1",
      "name": "Alice"
    }
    "#);
}

#[test]
fn run_query_with_params() {
    let subgraph = indoc! {r#"
        type User {
            id: ID!
            name: String!
        }

        type Query {
            user(id: ID!): User
        }
    "#};

    let resolver = json!({
        "id": "1",
        "name": "Alice"
    });

    let subgraph = DynamicSchema::builder(subgraph)
        .with_resolver("Query", "user", resolver)
        .into_subgraph("a");

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph)
            .with_toml_config(CONFIG)
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let args = json!({
            "__selection": "{ id name }",
            "id": "1"
        });

        stream.call_tool("query/user", args).await.unwrap()
    });

    insta::assert_json_snapshot!(&response, @r#"
    {
      "id": "1",
      "name": "Alice"
    }
    "#);
}

#[test]
fn mutation_rejected_when_disabled() {
    let subgraph = indoc! {r#"
        type User {
            id: ID!
            name: String!
        }

        input UserCreateInput {
            name: String!
        }

        type Query {
            user: User
        }

        type Mutation {
            createUser(input: UserCreateInput!): User!
        }
    "#};

    let resolver = json!({
        "id": "1",
        "name": "Alice"
    });

    let subgraph = DynamicSchema::builder(subgraph)
        .with_resolver("Query", "user", resolver.clone())
        .with_resolver("Mutation", "createUser", resolver)
        .into_subgraph("a");

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph)
            .with_toml_config(CONFIG) // Using the config where mutations are not enabled
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let args = json!({
            "__selection": "{ id name }",
            "input": {
                "name": "Bob"
            }
        });

        // Attempt to call a mutation tool when mutations are disabled
        stream.call_tool("mutation/createUser", args).await.unwrap_err()
    });

    insta::assert_debug_snapshot!(&response, @r#"
    McpError {
        code: -32602,
        message: "Invalid command",
    }
    "#);
}

#[test]
fn mutation_allowed_when_enabled() {
    let subgraph = indoc! {r#"
        type User {
            id: ID!
            name: String!
        }

        input UserCreateInput {
            name: String!
        }

        type Query {
            user: User
        }

        type Mutation {
            createUser(input: UserCreateInput!): User!
        }
    "#};

    let resolver = json!({
        "id": "1",
        "name": "Alice"
    });

    let subgraph = DynamicSchema::builder(subgraph)
        .with_resolver("Query", "user", resolver.clone())
        .with_resolver("Mutation", "createUser", resolver)
        .into_subgraph("a");

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph)
            .with_toml_config(MUT_CONFIG)
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let args = json!({
            "__selection": "{ id name }",
            "input": {
                "name": "Bob"
            }
        });

        stream.call_tool("mutation/createUser", args).await.unwrap()
    });

    insta::assert_debug_snapshot!(&response, @r#"
    Object {
        "id": String("1"),
        "name": String("Alice"),
    }
    "#);
}
