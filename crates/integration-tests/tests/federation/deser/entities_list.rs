use serde_json::json;

use integration_tests::{
    federation::{DeterministicEngine, GraphqlResponse},
    runtime,
};

const REQUIRED_OBJECT_SCHEMA: &str = r#"
type Query {
    users: [User]! @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    organization: Organaization! @join__field(graph: A)
    valid: String @join__field(graph: A)
}
type Organaization @join__type(graph: A, key: "id") @join__type(graph: B, key: "id") {
    id: ID!
    name: String! @join__field(graph: B)
    plan: String @join__field(graph: B)
}
"#;

const NULLABLE_OBJECT_SCHEMA: &str = r#"
type Query {
    users: [User]! @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    organization: Organaization @join__field(graph: A)
    valid: String @join__field(graph: A)
}
type Organaization @join__type(graph: A, key: "id") @join__type(graph: B, key: "id") {
    id: ID!
    name: String! @join__field(graph: B)
    plan: String @join__field(graph: B)
}
"#;

const QUERY: &str = r#"
{
    users {
        organization {
            name
            plan
        }
        valid
    }
    dummy
}"#;

fn run(schema: &str, query: &str, subgraph_responses: &[serde_json::Value]) -> GraphqlResponse {
    let schema = [
        r#"
        directive @core(feature: String!) repeatable on SCHEMA

        directive @join__owner(graph: join__Graph!) on OBJECT

        directive @join__type(graph: join__Graph!, key: String!, resolvable: Boolean = true) repeatable on OBJECT | INTERFACE

        directive @join__field(graph: join__Graph, requires: String, provides: String) on FIELD_DEFINITION

        directive @join__graph(name: String!, url: String!) on ENUM_VALUE

        enum join__Graph {
          A @join__graph(name: "accounts", url: "http://127.0.0.1:46697")
          B @join__graph(name: "products", url: "http://127.0.0.1:46698")
        }
        "#,
        schema,
    ]
    .join("\n");
    runtime().block_on(async {
        DeterministicEngine::new(&schema, query, subgraph_responses)
            .await
            .execute()
            .await
    })
}

#[test]
fn expected_required_object_got_entities() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbaes", "plan": "super-enterprise"}
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbaes",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          }
        ],
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_object_got_entities() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbaes", "plan": "super-enterprise"}
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbaes",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          }
        ],
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_object_got_too_many_entities() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbase", "plan": "super-enterprise"},
                {"name": "Unknown", "plan": "enterprise"}
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbase",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          }
        ],
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_object_got_too_many_entities() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbase", "plan": "super-enterprise"},
                {"name": "Unknown", "plan": "enterprise"}
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbase",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          }
        ],
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_object_got_too_few_entities() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"},
                {"organization": {"id": "2"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbase", "plan": "super-enterprise"},
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbase",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          },
          null
        ],
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "path": [
            "users",
            1,
            "organization",
            "name"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_nullable_object_got_too_few_entities() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"},
                {"organization": {"id": "2"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbase", "plan": "super-enterprise"},
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbase",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          },
          {
            "organization": null,
            "valid": "yes"
          }
        ],
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "path": [
            "users",
            1,
            "organization",
            "name"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_required_object_got_with_missing_nullable_field() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"},
                {"organization": {"id": "2"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbase", "plan": "super-enterprise"},
                {"name": "Other"},
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbase",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          },
          {
            "organization": {
              "name": "Other",
              "plan": null
            },
            "valid": "yes"
          }
        ],
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_object_got_with_missing_nullable_field() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"},
                {"organization": {"id": "2"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbase", "plan": "super-enterprise"},
                {"name": "Other"},
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbase",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          },
          {
            "organization": {
              "name": "Other",
              "plan": null
            },
            "valid": "yes"
          }
        ],
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_object_got_with_missing_required_field() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"},
                {"organization": {"id": "2"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbase", "plan": "super-enterprise"},
                {"plan": "enterprise"},
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbase",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          },
          null
        ],
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 5,
              "column": 13
            }
          ],
          "path": [
            "users",
            1,
            "organization",
            "name"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_nullable_object_got_with_missing_required_field() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        &[
            json!({"data": {"users": [
                {"organization": {"id": "1"}, "valid": "yes"},
                {"organization": {"id": "2"}, "valid": "yes"}
            ], "dummy": "yes"}}),
            json!({"data": {"_entities": [
                {"name": "Grafbase", "plan": "super-enterprise"},
                {"plan": "enterprise"},
            ]}}),
        ],
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "organization": {
              "name": "Grafbase",
              "plan": "super-enterprise"
            },
            "valid": "yes"
          },
          {
            "organization": null,
            "valid": "yes"
          }
        ],
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 5,
              "column": 13
            }
          ],
          "path": [
            "users",
            1,
            "organization",
            "name"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}
