use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use crate::gateway::extensions::resolver::ResolverExt;

#[test]
fn lookup_in_same_subgraph() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                extend schema
                    @link(url: "resolver-1.0.0", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key", "lookup"])

                type Query {
                    post: Post! @resolve
                    userLookup(id: ID!): User! @lookup @resolve
                }

                type Post {
                    id: ID!
                    authorId: ID!
                    author: User! @derive
                }

                type User @key(fields: "id") {
                    id: ID!
                    name: String!
                }
            "#,
            )
            .with_extension(ResolverExt::callback(|name, _arguments| match name.as_str() {
                "Query.post" => json!({"id": "post_1", "authorId": "1"}),
                "Query.userLookup" => {
                    json!({"id": "1", "name": "John Doe"})
                }
                _ => unreachable!(),
            }))
            .build()
            .await;

        let response = engine.post("query { post { id author { id name } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": {
                "id": "1",
                "name": "John Doe"
              }
            }
          }
        }
        "#);
    })
}

#[test]
fn missing_lookup_resolver() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                extend schema
                    @link(url: "resolver-1.0.0", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key", "lookup"])

                type Query {
                    post: Post! @resolve
                    userLookup(id: ID!): User! @lookup
                }

                type Post {
                    id: ID!
                    authorId: ID!
                    author: User! @derive
                }

                type User @key(fields: "id") {
                    id: ID!
                    name: String!
                }
            "#,
            )
            .with_extension(ResolverExt::echo_data())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.userLookup, for directive @lookup The @lookup field Query.userLookup does not have a any resolvers for the key: id
        34 |   post: Post! @extension__directive(graph: X, extension: RESOLVER, name: "resolve", arguments: {}) @join__field(graph: X)
        35 |   userLookup(id: ID!): User! @composite__lookup(graph: X) @join__field(graph: X)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        36 | }
        "#)
    })
}
