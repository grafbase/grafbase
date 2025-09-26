use graphql_mocks::{EchoSchema, dynamic::DynamicSchema};
use integration_tests::{
    gateway::{AuthenticationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authentication::static_auth::StaticAuth;

#[test]
fn query_and_response_auth_with_headers_modifications() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default())
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema @link(url: "authz-21", import: ["@deniedIds", "@deny"])

                    type Query {
                        users: [User]!
                        secret: String @deny
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
                        {"id": 2, "name": "Bob"},
                        {"id": 3, "name": "Charlie"},
                        {"id": 4, "name": "David"},
                        {"id": 5, "name": "Eve"},
                        {"id": 6, "name": "Frank"},
                        {"id": 7, "name": "Grace"},
                        {"id": 8, "name": "Helen"}
                    ]),
                )
                .into_subgraph("x"),
            )
            .with_extension(AuthenticationExt::new(StaticAuth::bytes("Hello world!")))
            .with_extension("authz-21")
            .build()
            .await;

        let response = engine
            .post(r#"query { secret header(name: "token") users { name } }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "secret": null,
            "header": "Hello world!",
            "users": [
              {
                "name": "Alice"
              },
              null,
              {
                "name": "Charlie"
              },
              null,
              {
                "name": "Eve"
              },
              {
                "name": "Frank"
              },
              {
                "name": "Grace"
              },
              null
            ]
          },
          "errors": [
            {
              "message": "Not authorized, query auth SDK021",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "secret"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            },
            {
              "message": "Not authorized, response auth SDK021",
              "locations": [
                {
                  "line": 1,
                  "column": 38
                }
              ],
              "path": [
                "users",
                1
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            },
            {
              "message": "Not authorized, response auth SDK021",
              "locations": [
                {
                  "line": 1,
                  "column": 38
                }
              ],
              "path": [
                "users",
                3
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            },
            {
              "message": "Not authorized, response auth SDK021",
              "locations": [
                {
                  "line": 1,
                  "column": 38
                }
              ],
              "path": [
                "users",
                7
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}
