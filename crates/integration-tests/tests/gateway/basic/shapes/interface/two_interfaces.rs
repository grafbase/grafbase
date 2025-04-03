use serde_json::json;

use super::with_gateway;

#[test]
fn disjoint() {
    let schema = r#"
    interface I1 {
        f1: ID!
    }

    interface I2 {
        f2: ID!
    }

    type A implements Node & I1 {
        id: ID!
        f1: ID!
        a: String!
    }

    type B implements Node & I2 {
        id: ID!
        f2: ID!
        b: String!
    }

    type C implements Node {
        id: ID!
        c: String!
    }
    "#;
    let nodes = json!([
        {"__typename": "A", "id": "a_id", "f1": "a_f1", "a": "a_a"},
        {"__typename": "B", "id": "b_id", "f2": "b_f2", "b": "b_b"},
        {"__typename": "C", "id": "c_id", "c": "c_c"},
    ]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 { f1 }
                    ... on I2 { f2 }
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
                "f1": "a_f1"
              },
              {
                "f2": "b_f2"
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
                    id
                    ... on I1 { f1 }
                    ... on I2 { f2 }
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
                "f1": "a_f1"
              },
              {
                "id": "b_id",
                "f2": "b_f2"
              },
              {
                "id": "c_id"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 { __typename f1 }
                    ... on I2 { __typename f2 }
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
                "f1": "a_f1"
              },
              {
                "__typename": "B",
                "f2": "b_f2"
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
                    id
                    __typename
                    ... on I1 { f1 __typename }
                    ... on I2 { f2 __typename }
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
                "__typename": "A",
                "f1": "a_f1"
              },
              {
                "id": "b_id",
                "__typename": "B",
                "f2": "b_f2"
              },
              {
                "id": "c_id",
                "__typename": "C"
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
                    ... on I1 { ... on A { a } }
                    ... on C { c }
                    ...F2
                } }
                fragment F2 on I2 { ... on B { b } }
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
              },
              {
                "c": "c_c"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 { ... on A { id f1 a } }
                    ... on C { id c }
                    ...F2
                } }
                fragment F2 on I2 { ... on B { id f2 b } }
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
                "f1": "a_f1",
                "a": "a_a"
              },
              {
                "id": "b_id",
                "f2": "b_f2",
                "b": "b_b"
              },
              {
                "id": "c_id",
                "c": "c_c"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 { ... on A { __typename id f1 a } }
                    ... on C { __typename id c }
                    ...F2
                } }
                fragment F2 on I2 { ... on B { __typename id f2 b } }
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
                "f1": "a_f1",
                "a": "a_a"
              },
              {
                "__typename": "B",
                "id": "b_id",
                "f2": "b_f2",
                "b": "b_b"
              },
              {
                "__typename": "C",
                "id": "c_id",
                "c": "c_c"
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
                    id
                    ... on I1 { ... on A { a } }
                    ... on C { c }
                    ...F2
                } }
                fragment F2 on I2 { ... on B { b } }
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
              },
              {
                "id": "c_id",
                "c": "c_c"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    ... on I1 { ... on A { a } }
                    ... on C { c }
                    ...F2

                    ... on I1 { f1 }
                    ... on I2 { f2 }
                } }
                fragment F2 on I2 { ... on B { b } }
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
                "a": "a_a",
                "f1": "a_f1"
              },
              {
                "id": "b_id",
                "b": "b_b",
                "f2": "b_f2"
              },
              {
                "id": "c_id",
                "c": "c_c"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 { ... on A { a } }
                    ... on C { __typename c }
                    ...F2

                    ... on I1 { __typename f1 }
                    ... on I2 { f2 }
                } }
                fragment F2 on I2 { __typename ... on B { b } }
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
                "f1": "a_f1"
              },
              {
                "__typename": "B",
                "b": "b_b",
                "f2": "b_f2"
              },
              {
                "__typename": "C",
                "c": "c_c"
              }
            ]
          }
        }
        "#
        );
    });
}

#[test]
fn non_disjoint() {
    let schema = r#"
    interface I1 {
        f1: ID!
    }

    interface I2 {
        f2: ID!
    }

    type A implements Node & I1 {
        id: ID!
        f1: ID!
        a: String!
    }

    type B implements Node & I2 {
        id: ID!
        f2: ID!
        b: String!
    }

    type C implements Node {
        id: ID!
        c: String!
    }

    type D implements Node & I1 & I2 {
        id: ID!
        f1: ID!
        f2: ID!
        d: String!
    }
    "#;
    let nodes = json!([
        {"__typename": "A", "id": "a_id", "f1": "a_f1", "a": "a_a"},
        {"__typename": "B", "id": "b_id", "f2": "b_f2", "b": "b_b"},
        {"__typename": "C", "id": "c_id", "c": "c_c"},
        {"__typename": "D", "id": "d_id", "f1": "d_f1", "f2": "d_f2", "d": "d_d"}
    ]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 { f1 }
                    ... on I2 { f2 }
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
                "f1": "a_f1"
              },
              {
                "f2": "b_f2"
              },
              {},
              {
                "f1": "d_f1",
                "f2": "d_f2"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    ... on I1 { f1 }
                    ... on I2 { f2 }
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
                "f1": "a_f1"
              },
              {
                "id": "b_id",
                "f2": "b_f2"
              },
              {
                "id": "c_id"
              },
              {
                "id": "d_id",
                "f1": "d_f1",
                "f2": "d_f2"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 { __typename f1 }
                    ... on I2 { __typename f2 }
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
                "f1": "a_f1"
              },
              {
                "__typename": "B",
                "f2": "b_f2"
              },
              {},
              {
                "__typename": "D",
                "f1": "d_f1",
                "f2": "d_f2"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    __typename
                    ... on I1 { f1 __typename }
                    ... on I2 { f2 __typename }
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
                "__typename": "A",
                "f1": "a_f1"
              },
              {
                "id": "b_id",
                "__typename": "B",
                "f2": "b_f2"
              },
              {
                "id": "c_id",
                "__typename": "C"
              },
              {
                "id": "d_id",
                "__typename": "D",
                "f1": "d_f1",
                "f2": "d_f2"
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
                    ... on I1 { ... on A { a } ... on D { d } }
                    ... on C { c }
                    ...F2
                } }
                fragment F2 on I2 { ... on B { b } }
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
              },
              {
                "c": "c_c"
              },
              {
                "d": "d_d"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 {
                        ... on A { id f1 a }
                        ... on D { id f2 d }
                    }
                    ... on C { id c }
                    ...F2
                } }
                fragment F2 on I2 {
                    ... on B { id f2 b }
                    ... on D { f1 }
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
                "f1": "a_f1",
                "a": "a_a"
              },
              {
                "id": "b_id",
                "f2": "b_f2",
                "b": "b_b"
              },
              {
                "id": "c_id",
                "c": "c_c"
              },
              {
                "id": "d_id",
                "f2": "d_f2",
                "d": "d_d",
                "f1": "d_f1"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 {
                        ... on A { __typename id f1 a }
                        ... on D { id d }
                    }
                    ... on C { __typename id c }
                    ... on D { __typename f2 }
                    ...F2
                } }
                fragment F2 on I2 {
                    ... on B { __typename id f2 b }
                    ... on D { f1 }
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
                "f1": "a_f1",
                "a": "a_a"
              },
              {
                "__typename": "B",
                "id": "b_id",
                "f2": "b_f2",
                "b": "b_b"
              },
              {
                "__typename": "C",
                "id": "c_id",
                "c": "c_c"
              },
              {
                "id": "d_id",
                "d": "d_d",
                "__typename": "D",
                "f2": "d_f2",
                "f1": "d_f1"
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
                    id
                    ... on I1 {
                        ... on A { a }
                        ... on D { d }
                    }
                    ... on C { c }
                    ...F2
                } }
                fragment F2 on I2 { ... on B { b } }
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
              },
              {
                "id": "c_id",
                "c": "c_c"
              },
              {
                "id": "d_id",
                "d": "d_d"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post(
                r#"query { nodes {
                    id
                    ... on I1 {
                        ... on A { a }
                        ... on D { d }
                    }
                    ... on C { c }
                    ...F2

                    ... on I1 { f1 }
                    ... on I2 { f2 }
                } }
                fragment F2 on I2 { ... on B { b } }
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
                "a": "a_a",
                "f1": "a_f1"
              },
              {
                "id": "b_id",
                "b": "b_b",
                "f2": "b_f2"
              },
              {
                "id": "c_id",
                "c": "c_c"
              },
              {
                "id": "d_id",
                "d": "d_d",
                "f1": "d_f1",
                "f2": "d_f2"
              }
            ]
          }
        }
        "#
        );
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on I1 {
                        ... on A { a }
                        ... on D { d }
                    }
                    ... on C { __typename c }
                    ...F2

                    ... on I1 { __typename f1 }
                    ... on I2 { f2 }
                } }
                fragment F2 on I2 { __typename ... on B { b } }
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
                "f1": "a_f1"
              },
              {
                "__typename": "B",
                "b": "b_b",
                "f2": "b_f2"
              },
              {
                "__typename": "C",
                "c": "c_c"
              },
              {
                "d": "d_d",
                "__typename": "D",
                "f1": "d_f1",
                "f2": "d_f2"
              }
            ]
          }
        }
        "#
        );
    });
}
