use graphql_mocks::dynamic::{DynamicSchema, EntityResolverContext};
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn simple_shared_root() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                      @link(
                        url: "https://specs.apollo.dev/federation/v2.3"
                        import: ["@key", "@shareable"]
                      )

                    type Query {
                        node: Node! @shareable
                    }

                    type Node {
                        id: ID! @shareable
                        f1: String!
                    }
                    "#,
                )
                .with_resolver("Query", "node", json!({"id": "1", "f1": "A"}))
                .into_subgraph("a"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                      @link(
                        url: "https://specs.apollo.dev/federation/v2.3"
                        import: ["@key", "@shareable"]
                      )

                    type Query {
                        node: Node! @shareable
                    }

                    type Node {
                        id: ID! @shareable
                        f2: String!
                    }
                    "#,
                )
                .with_resolver("Query", "node", json!({"id": "1", "f2": "B"}))
                .into_subgraph("b"),
            )
            .build()
            .await;

        let response = gateway
            .post(
                r#"
            query {
                node {
                    id
                    f1
                    f2
                }
            }
            "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "id": "1",
              "f1": "A",
              "f2": "B"
            }
          }
        }
        "#);
    });
}

#[test]
fn nested_shared_root() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                      @link(
                        url: "https://specs.apollo.dev/federation/v2.3"
                        import: ["@key", "@shareable"]
                      )

                    type Query {
                        node: Node! @shareable
                    }

                    type Node {
                        id: ID! @shareable
                        node: Node! @shareable
                        f1: String!
                    }
                    "#,
                )
                .with_resolver("Query", "node", json!({"id": "0", "f1": "A1"}))
                .with_resolver("Node", "node", json!({"id": "1", "f1": "A2"}))
                .into_subgraph("a"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                      @link(
                        url: "https://specs.apollo.dev/federation/v2.3"
                        import: ["@key", "@shareable"]
                      )

                    type Query {
                        node: Node! @shareable
                    }

                    type Node {
                        id: ID! @shareable
                        node: Node! @shareable
                        f2: String!
                    }
                    "#,
                )
                .with_resolver("Query", "node", json!({"id": "0", "f2": "B1"}))
                .with_resolver("Node", "node", json!({"id": "1", "f2": "B2"}))
                .into_subgraph("b"),
            )
            .build()
            .await;

        let response = gateway
            .post(
                r#"
            query {
                node {
                    id
                    f1
                    f2
                    node {
                        id
                        f1
                        f2
                    }
                }
            }
            "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "id": "0",
              "f1": "A1",
              "f2": "B1",
              "node": {
                "id": "1",
                "f1": "A2",
                "f2": "B2"
              }
            }
          }
        }
        "#);
    });
}

#[test]
fn shared_root_with_entity() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                      @link(
                        url: "https://specs.apollo.dev/federation/v2.3"
                        import: ["@key", "@shareable"]
                      )

                    type Query {
                        node: Node! @shareable
                    }

                    type Node @key(fields: "id") {
                        id: ID! @shareable
                        n1: Node! @shareable
                        n2: Node! @shareable
                        a: String! @shareable
                        b: String! @shareable
                    }
                    "#,
                )
                .with_resolver("Query", "node", json!({"id": "0", "a": "A0", "b": "B0"}))
                .with_resolver("Node", "n1", json!({"id": "1", "a": "A1", "b": "B1"}))
                .with_resolver("Node", "n2", json!({"id": "2", "a": "A2", "b": "B2"}))
                .with_entity_resolver("Node", |ctx: EntityResolverContext<'_>| {
                    let id = ctx.representation["id"].as_str().unwrap();
                    Some(json!({"id": id, "a": format!("A{id}"), "b": format!("B{id}")}))
                })
                .into_subgraph("a"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                      @link(
                        url: "https://specs.apollo.dev/federation/v2.3"
                        import: ["@key", "@shareable"]
                      )

                    type Query {
                        node: Node! @shareable
                    }

                    type Node @key(fields: "id") {
                        id: ID! @shareable
                        n2: Node! @shareable
                        n3: Node! @shareable
                        b: String! @shareable
                        c: String! @shareable
                    }
                    "#,
                )
                .with_resolver("Query", "node", json!({"id": "0", "b": "B0", "c": "C0"}))
                .with_resolver("Node", "n2", json!({"id": "2", "b": "B2", "c": "C2"}))
                .with_resolver("Node", "n3", json!({"id": "3", "b": "B3", "c": "C3"}))
                .with_entity_resolver("Node", |ctx: EntityResolverContext<'_>| {
                    let id = ctx.representation["id"].as_str().unwrap();
                    Some(json!({"id": id, "b": format!("B{id}"), "c": format!("C{id}")}))
                })
                .into_subgraph("b"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                      @link(
                        url: "https://specs.apollo.dev/federation/v2.3"
                        import: ["@key", "@shareable"]
                      )

                    type Node @key(fields: "id") {
                        id: ID! @shareable
                        n1: Node! @shareable
                        n3: Node! @shareable
                        a: String! @shareable
                        c: String! @shareable
                    }
                    "#,
                )
                .with_resolver("Node", "n1", json!({"id": "1", "a": "A1", "c": "C1"}))
                .with_resolver("Node", "n3", json!({"id": "3", "a": "A3", "c": "C3"}))
                .with_entity_resolver("Node", |ctx: EntityResolverContext<'_>| {
                    let id = ctx.representation["id"].as_str().unwrap();
                    Some(json!({"id": id, "a": format!("A{id}"), "c": format!("C{id}")}))
                })
                .into_subgraph("c"),
            )
            .build()
            .await;

        let response = gateway
            .post(
                r#"
            query {
                node {
                    a b c
                    n1 {
                        a b c
                        n2 {
                            a b c
                            n3 {
                                a b c
                            }
                        }
                    }
                    n2 {
                        a b c
                        n3 {
                            a b c
                            n1 {
                                a b c
                            }
                        }
                    }
                    n3 {
                        a b c
                        n1 {
                            a b c
                            n2 {
                                a b c
                            }
                        }
                    }
                }
            }
            "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "a": "A0",
              "b": "B0",
              "c": "C0",
              "n1": {
                "a": "A1",
                "b": "B1",
                "c": "C1",
                "n2": {
                  "a": "A2",
                  "b": "B2",
                  "c": "C2",
                  "n3": {
                    "a": "A3",
                    "b": "B3",
                    "c": "C3"
                  }
                }
              },
              "n2": {
                "a": "A2",
                "b": "B2",
                "c": "C2",
                "n3": {
                  "a": "A3",
                  "b": "B3",
                  "c": "C3",
                  "n1": {
                    "a": "A1",
                    "b": "B1",
                    "c": "C1"
                  }
                }
              },
              "n3": {
                "a": "A3",
                "b": "B3",
                "c": "C3",
                "n1": {
                  "a": "A1",
                  "b": "B1",
                  "c": "C1",
                  "n2": {
                    "a": "A2",
                    "b": "B2",
                    "c": "C2"
                  }
                }
              }
            }
          }
        }
        "#);
    });
}
