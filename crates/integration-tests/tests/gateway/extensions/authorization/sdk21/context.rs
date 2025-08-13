use graphql_mocks::{EchoSchema, Schema as _, dynamic::DynamicSchema};
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn query_context() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authz-21-1.0.0", import: ["@grant"])

                type Query {
                    header(name: String): String @grant
                }
                "#,
            ))
            .with_extension("auth-21")
            .with_extension("hooks-21")
            .with_extension("authz-21")
            .build()
            .await;

        let response = engine
            .post(
                r#"query {
                    hooksContext: header(name: "hooks-context")
                    token: header(name: "token")
                }"#,
            )
            .header("Authorization", "bearer")
            .header("hooks-context", "I'm hooked!")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "hooksContext": "I'm hooked!",
            "token": "sdk21:bearer:default"
          }
        }
        "#);
    });
}

#[test]
fn response_context() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.authz-21.config]
                context = "authz21 context"
                response_error_with_context = true
                "#,
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema @link(url: "authz-21-1.0.0", import: ["@deniedIds"])

                    type Query {
                        users: [User]!
                    }

                    type User @deniedIds(ids: [2, 4, 8]) {
                        id: Int!
                        name: String!
                    }
                    "#,
                )
                .with_resolver(
                    "Query",
                    "users",
                    serde_json::json!([
                        {"id": 1, "name": "Alice"},
                    ]),
                )
                .into_subgraph("x"),
            )
            .with_extension("auth-21")
            .with_extension("hooks-21")
            .with_extension("authz-21")
            .build()
            .await;

        let response = engine
            .post(r#"query { users { name } }"#)
            .header("Authorization", "bearer")
            .header("hooks-context", "I'm hooked!")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "users": [
              null
            ]
          },
          "errors": [
            {
              "message": "Failure",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "users",
                0
              ],
              "extensions": {
                "token": "sdk21:bearer:default",
                "authorization_context": [
                  "authz21 context"
                ],
                "hooks_context": "I'm hooked!",
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}
