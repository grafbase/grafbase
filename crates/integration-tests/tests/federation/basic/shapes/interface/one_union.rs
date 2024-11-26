use serde_json::json;

use super::with_gateway;

#[test]
fn superset() {
    let schema = r#"
    union Other = A | B

    type A implements Node {
        id: ID!
        a: String!
    }
    type B {
        b: String!
    }
    "#;
    let nodes = json!([{"__typename": "A", "id": "a_id", "a": "a_a"}]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other {
                        ... on A { a }
                        ... on B { b }
                    }
                } }"#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other { __typename }
                } }"#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A"
              }
            ]
          }
        }
        "#
        );

        // ===
        // Mix
        // ===
        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    ... on Other {
                        ... on A { a }
                        ... on B { b }
                    }
                } }"#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "id": "a_id",
                "a": "a_a"
              }
            ]
          }
        }
        "#
        );

        // =============================
        // Specifying fields on objects.
        // =============================
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    ... on A { a }
                } }
                fragment FO on Other {
                  ... on B { b }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    ... on A { id a }
                } }
                fragment FO on Other {
                  ... on B { b }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "id": "a_id",
                "a": "a_a"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    ... on A { __typename id a }
                } }
                fragment FO on Other {
                  ... on B { __typename b }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A",
                "id": "a_id",
                "a": "a_a"
              }
            ]
          }
        }
        "#
        );

        // ==================================
        // Mixing interface and object fields
        // ==================================
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    id
                } }
                fragment FO on Other {
                    ... on A { a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a",
                "id": "a_id"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    __typename
                    id
                } }
                fragment FO on Other {
                    ... on A { a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a",
                "__typename": "A",
                "id": "a_id"
              }
            ]
          }
        }
        "#
        );
    })
}

#[test]
fn subset() {
    let schema = r#"
    union Other = A

    type A implements Node {
        id: ID!
        a: String!
    }
    type B implements Node {
        id: ID!
        b: String!
    }
    "#;
    let nodes = json!([
        {"__typename": "A", "id": "a_id", "a": "a_a"},
        {"__typename": "B", "id": "b_id", "b": "b_b"},
    ]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other {
                        ... on A { a }
                    }
                } }"#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a"
              },
              {}
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other {
                        __typename
                        ... on A { a }
                    }
                } }"#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A",
                "a": "a_a"
              },
              {}
            ]
          }
        }
        "#
        );

        // ===
        // Mix
        // ===
        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    ... on Other {
                        ... on A { a }
                    }
                } }"#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "id": "a_id",
                "a": "a_a"
              },
              {
                "id": "b_id"
              }
            ]
          }
        }
        "#
        );

        // =============================
        // Specifying fields on objects.
        // =============================
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    ... on B { b }
                } }
                fragment FO on Other {
                  ... on A { a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a"
              },
              {
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    ... on B { id b }
                } }
                fragment FO on Other {
                  ... on A { id a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "id": "a_id",
                "a": "a_a"
              },
              {
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    ... on A { __typename id a }
                    ... on B { __typename id b }
                } }
                fragment FO on Other {
                  ... on A { __typename id a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A",
                "id": "a_id",
                "a": "a_a"
              },
              {
                "__typename": "B",
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );

        // ==================================
        // Mixing interface and object fields
        // ==================================
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    id
                    ... on A { a }
                    ... on B { b }
                } }
                fragment FO on Other {
                    ... on A { a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a",
                "id": "a_id"
              },
              {
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    id
                    ... on A { id a }
                    ... on B { id b }
                } }
                fragment FO on Other {
                    ... on A { id a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "id": "a_id",
                "a": "a_a"
              },
              {
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    __typename
                    id
                    ... on A { a }
                    ... on B { b }
                } }
                fragment FO on Other {
                    ... on A { a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a",
                "__typename": "A",
                "id": "a_id"
              },
              {
                "__typename": "B",
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    __typename
                    id
                    ... on B { __typename id b }
                } }
                fragment FO on Other {
                  __typename
                  ... on A { __typename id a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A",
                "id": "a_id",
                "a": "a_a"
              },
              {
                "__typename": "B",
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );
    })
}

#[test]
fn equivalent() {
    let schema = r#"
    union Other = A | B

    type A implements Node {
        id: ID!
        a: String!
    }
    type B implements Node {
        id: ID!
        b: String!
    }
    "#;
    let nodes = json!([
        {"__typename": "A", "id": "a_id", "a": "a_a"},
        {"__typename": "B", "id": "b_id", "b": "b_b"},
    ]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other {
                        ... on A { a }
                        ... on B { b }
                    }
                } }"#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a"
              },
              {
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other {
                        __typename
                        ... on A { a }
                        ... on B { b }
                    }
                } }"#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A",
                "a": "a_a"
              },
              {
                "__typename": "B",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );

        // ===
        // Mix
        // ===
        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    ... on Other {
                        ... on A { a }
                        ... on B { b }
                    }
                } }"#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "id": "a_id",
                "a": "a_a"
              },
              {
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );

        // =============================
        // Specifying fields on objects.
        // =============================
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    ... on B { b }
                } }
                fragment FO on Other {
                    ... on A { a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a"
              },
              {
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    ... on B { id b }
                } }
                fragment FO on Other {
                  ... on A { id a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "id": "a_id",
                "a": "a_a"
              },
              {
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    ... on B { __typename id b }
                } }
                fragment FO on Other {
                  ... on A { __typename id a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A",
                "id": "a_id",
                "a": "a_a"
              },
              {
                "__typename": "B",
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );

        // ==================================
        // Mixing interface and object fields
        // ==================================
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    id
                } }
                fragment FO on Other {
                    ... on A { a }
                    ... on B { b }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a",
                "id": "a_id"
              },
              {
                "b": "b_b",
                "id": "b_id"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    id
                    ... on A { a }
                    ... on B { b }
                } }
                fragment FO on Other {
                    ... on A { a id }
                    ... on B { b id }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "a": "a_a",
                "id": "a_id"
              },
              {
                "b": "b_b",
                "id": "b_id"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ...FO
                    __typename
                    id
                    ... on B { __typename id b }
                } }
                fragment FO on Other {
                  __typename
                  ... on A { __typename id a }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A",
                "id": "a_id",
                "a": "a_a"
              },
              {
                "__typename": "B",
                "id": "b_id",
                "b": "b_b"
              }
            ]
          }
        }
        "#
        );
    })
}
