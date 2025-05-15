use graphql_mocks::dynamic::{DynamicSchema, EntityResolverContext};
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn shareable_field() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive", "@shareable"])

                type Query {
                    post: Post
                }

                type Post {
                    id: ID!
                    code: String!
                    commentIds: [ID!]
                    comments: [Comment!] @derive
                }

                type Comment @key(fields: "id") {
                    id: ID!
                    category: ID @shareable
                }
                "#,
                )
                .with_resolver("Query", "post", json!({"commentIds": ["c1", "c2"]}))
                .into_subgraph("posts"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        comments: [Comment!]!
                    }

                    type Comment @key(fields: "id") {
                        id: ID!
                        category: ID @shareable
                    }
                    "#,
                )
                .with_entity_resolver("Comment", |ctx: EntityResolverContext<'_>| -> Option<serde_json::Value> {
                    match ctx.representation["id"].as_str().unwrap() {
                        "c1" => Some(json!({"id": "c1", "category": "cat1"})),
                        _ => None
                    }
                })
                .into_subgraph("comments"),
            )
            .build()
            .await;

        let response = gateway.post("{ post { comments { id category } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "comments": [
                {
                  "id": "c1",
                  "category": "cat1"
                },
                {
                  "id": "c2",
                  "category": null
                }
              ]
            }
          }
        }
        "#
        );
    })
}

#[test]
fn external_field() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive", "@external"])

                type Query {
                    post: Post
                }

                type Post {
                    id: ID!
                    code: String!
                    commentIds: [ID!]
                    comments: [Comment!] @derive
                }

                type Comment @key(fields: "id") {
                    id: ID!
                    category: ID @external
                }
                "#,
                )
                .with_resolver("Query", "post", json!({"commentIds": ["c1", "c2"]}))
                .into_subgraph("posts"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        comments: [Comment!]!
                    }

                    type Comment @key(fields: "id") {
                        id: ID!
                        category: ID
                    }
                    "#,
                )
                .with_entity_resolver("Comment", |ctx: EntityResolverContext<'_>| -> Option<serde_json::Value> {
                    match ctx.representation["id"].as_str().unwrap() {
                        "c1" => Some(json!({"id": "c1", "category": "cat1"})),
                        _ => None
                    }
                })
                .into_subgraph("comments"),
            )
            .build()
            .await;

        let response = gateway.post("{ post { comments { id category } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "comments": [
                {
                  "id": "c1",
                  "category": "cat1"
                },
                {
                  "id": "c2",
                  "category": null
                }
              ]
            }
          }
        }
        "#
        );
    })
}
