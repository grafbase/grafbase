use serde_json::json;

use super::with_gateway;

#[test]
fn superset() {
    let schema = r#"
    interface Other {
        other: ID!
    }

    type A implements Node & Other {
        id: ID!
        other: ID!
        a: String!
    }
    type B implements Other {
        other: ID!
        b: String!
    }
    "#;
    let nodes = json!([
        {"__typename": "A", "id": "a_id", "other": "a_other", "a": "a_a"},
    ]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other { other }
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
                "other": "a_other"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other { __typename other }
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
                "other": "a_other"
              }
            ]
          }
        }
        "#
        );

        // ======================
        // Double type conditions
        // ======================
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Node { ... on Other { other } }
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
                "other": "a_other"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other { ... on Node { id } }
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
                "id": "a_id"
              }
            ]
          }
        }
        "#
        );

        // =============
        // interface mix
        // =============
        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    ... on Other { other }
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
                "other": "a_other"
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
                    ... on A { other a }
                } }
                fragment FO on Other {
                  ... on B { other b }
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
                "other": "a_other",
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
                    ... on A { id other a }
                } }
                fragment FO on Other {
                  ... on B { other b }
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
                "other": "a_other",
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
                    ... on A { __typename id other a }
                } }
                fragment FO on Other {
                  ... on B { __typename other b }
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
                "other": "a_other",
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
                    ... on A { a }
                } }
                fragment FO on Other {
                  other
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
                "other": "a_other",
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
                    id
                    ... on A { id other a }
                } }
                fragment FO on Other {
                  other
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
                "other": "a_other",
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
                    __typename
                    id
                    ... on A { a }
                } }
                fragment FO on Other {
                  other
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
                "other": "a_other",
                "__typename": "A",
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
                    __typename
                    id
                    ... on A { __typename id other a }
                } }
                fragment FO on Other {
                  __typename
                  other
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
                "other": "a_other",
                "id": "a_id",
                "a": "a_a"
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
    interface Other {
        other: ID!
    }

    type A implements Node & Other {
        id: ID!
        other: ID!
        a: String!
    }
    type B implements Node {
        id: ID!
        b: String!
    }
    "#;
    let nodes = json!([
        {"__typename": "A", "id": "a_id", "other": "a_other", "a": "a_a"},
        {"__typename": "B", "id": "b_id", "b": "b_b"},
    ]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other { other }
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
                "other": "a_other"
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
                    ... on Other { __typename other }
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
                "other": "a_other"
              },
              {}
            ]
          }
        }
        "#
        );

        // ======================
        // Double type conditions
        // ======================
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Node { ... on Other { other } }
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
                "other": "a_other"
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
                    ... on Other { ... on Node { id } }
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
                "id": "a_id"
              },
              {}
            ]
          }
        }
        "#
        );

        // =============
        // interface mix
        // =============
        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    ... on Other { other }
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
                "other": "a_other"
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
                    ... on B { b }
                } }
                fragment FO on Other {
                  ... on A { other a }
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
                "other": "a_other",
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
                  ... on A { id other a }
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
                "other": "a_other",
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
                  ... on A { __typename id other a }
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
                "other": "a_other",
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
                  other
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
                "other": "a_other",
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
                    id
                    ... on A { id other a }
                    ... on B { id b }
                } }
                fragment FO on Other {
                  other
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
                "other": "a_other",
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
                  other
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
                "other": "a_other",
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
                  other
                  ... on A { __typename id other a }
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
                "other": "a_other",
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
    interface Other {
        other: ID!
    }

    type A implements Node & Other {
        id: ID!
        other: ID!
        a: String!
    }
    type B implements Node & Other {
        id: ID!
        other: ID!
        b: String!
    }
    "#;
    let nodes = json!([
        {"__typename": "A", "id": "a_id", "other": "a_other", "a": "a_a"},
        {"__typename": "B", "id": "b_id", "other": "b_other", "b": "b_b"},
    ]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other { other }
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
                "other": "a_other"
              },
              {
                "other": "b_other"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other { __typename other }
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
                "other": "a_other"
              },
              {
                "__typename": "B",
                "other": "b_other"
              }
            ]
          }
        }
        "#
        );

        // ======================
        // Double type conditions
        // ======================
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Node { ... on Other { other } }
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
                "other": "a_other"
              },
              {
                "other": "b_other"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on Other { ... on Node { id } }
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
                "id": "a_id"
              },
              {
                "id": "b_id"
              }
            ]
          }
        }
        "#
        );

        // =============
        // interface mix
        // =============
        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    ... on Other { other }
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
                "other": "a_other"
              },
              {
                "id": "b_id",
                "other": "b_other"
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
                    ... on B { other b }
                } }
                fragment FO on Other {
                  ... on A { other a }
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
                "other": "a_other",
                "a": "a_a"
              },
              {
                "other": "b_other",
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
                    ... on B { id other b }
                } }
                fragment FO on Other {
                  ... on A { id other a }
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
                "other": "a_other",
                "a": "a_a"
              },
              {
                "id": "b_id",
                "other": "b_other",
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
                    ... on B { __typename id other b }
                } }
                fragment FO on Other {
                  ... on A { __typename id other a }
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
                "other": "a_other",
                "a": "a_a"
              },
              {
                "__typename": "B",
                "id": "b_id",
                "other": "b_other",
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
                  other
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
                "other": "a_other",
                "id": "a_id",
                "a": "a_a"
              },
              {
                "other": "b_other",
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
                    ... on A { id other a }
                    ... on B { id other b }
                } }
                fragment FO on Other {
                  other
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
                "other": "a_other",
                "id": "a_id",
                "a": "a_a"
              },
              {
                "other": "b_other",
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
                  other
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
                "other": "a_other",
                "__typename": "A",
                "id": "a_id",
                "a": "a_a"
              },
              {
                "other": "b_other",
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
                    ... on B { __typename id other b }
                } }
                fragment FO on Other {
                  __typename
                  other
                  ... on A { __typename id other a }
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
                "other": "a_other",
                "id": "a_id",
                "a": "a_a"
              },
              {
                "__typename": "B",
                "other": "b_other",
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
