use super::with_gateway;
use serde_json::json;

#[test]
fn one_object() {
    let schema = r#"
    type A implements Node {
        id: ID!
        a: String!
    }
    "#;
    let nodes = json!([{"__typename": "A", "id": "a_id", "a": "a_a"}]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway.post("query { nodes { id } }").await;
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

        let response = gateway.post("query { nodes { __typename } }").await;
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

        let response = gateway.post("query { nodes { ... on A { id a } } }").await;
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

        let response = gateway.post("query { nodes { ... on A { __typename id a } } }").await;
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

        let response = gateway.post("query { nodes { id ... on A { a } } }").await;
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

        let response = gateway.post("query { nodes { __typename id ... on A { a } } }").await;
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

        let response = gateway
            .post("query { nodes { __typename id ... on A { __typename id } } }")
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
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
fn two_objects() {
    let schema = r#"
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
        {"__typename": "B", "id": "b_id", "b": "b_b"}
    ]);

    with_gateway(schema, nodes, |gateway| async move {
        let response = gateway.post("query { nodes { id } }").await;
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

        let response = gateway.post("query { nodes { __typename } }").await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A"
              },
              {
                "__typename": "B"
              }
            ]
          }
        }
        "#
        );

        // ==============
        // === only A ===
        // ==============
        let response = gateway.post("query { nodes { ... on A { id a } } }").await;
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
              {}
            ]
          }
        }
        "#
        );

        let response = gateway.post("query { nodes { ... on A { __typename id a } } }").await;
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
              {}
            ]
          }
        }
        "#
        );

        let response = gateway.post("query { nodes { id ... on A { a } } }").await;
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

        let response = gateway.post("query { nodes { __typename id ... on A { a } } }").await;
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
                "id": "b_id"
              }
            ]
          }
        }
        "#
        );

        let response = gateway
            .post("query { nodes { __typename id ... on A { __typename id } } }")
            .await;
        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "nodes": [
              {
                "__typename": "A",
                "id": "a_id"
              },
              {
                "__typename": "B",
                "id": "b_id"
              }
            ]
          }
        }
        "#
        );

        // ===============
        // === A and B ===
        // ===============
        let response = gateway
            .post(
                r#"query { nodes {
                    ... on A { id a }
                    ... on B { id b }
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

        let response = gateway
            .post(
                r#"query { nodes {
                    ... on A { __typename id a }
                    ... on B { __typename id b }
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
                    id 
                    ... on A { a }
                    ... on B { b }
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

        let response = gateway
            .post(
                r#"query { nodes {
                    __typename id
                    ... on A { a }
                    ... on B { b }
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
                    __typename id
                    ... on A { __typename id }
                    ... on B { __typename id }
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
                "id": "a_id"
              },
              {
                "__typename": "B",
                "id": "b_id"
              }
            ]
          }
        }
        "#
        );
    })
}
