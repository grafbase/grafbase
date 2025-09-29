use integration_tests::{gateway::Gateway, runtime};

use super::super::{Resolve, gql_product};

#[test]
fn nested_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_product())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@require", "@key", "@external"])

                type Product @key(fields: "id") {
                    id: ID!
                    details: ProductDetails @external
                    categories: [String!]! @external
                    dummy(code: String @require(field: "details.code")): JSON @resolve
                }

                type ProductDetails @external {
                    code: String!
                }

                scalar JSON
                "#,
            )
            .with_extension(Resolve::with(Ok))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "dummy": {
                  "code": "I1"
                }
              },
              {
                "id": "2",
                "dummy": {
                  "code": "I2"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn list() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_product())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@require", "@key", "@external"])

                type Product @key(fields: "id") {
                    id: ID!
                    details: ProductDetails @external
                    categories: [String!]! @external
                    dummy(categories: [String] @require(field: "categories")): JSON @resolve
                }

                type ProductDetails @external {
                    code: String!
                }

                scalar JSON
                "#,
            )
            .with_extension(Resolve::with(Ok))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "dummy": {
                  "categories": [
                    "C1",
                    "C11"
                  ]
                }
              },
              {
                "id": "2",
                "dummy": {
                  "categories": [
                    "C2",
                    "C22"
                  ]
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn input_object() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_product())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@require", "@key", "@external"])

                type Product @key(fields: "id") {
                    id: ID!
                    details: ProductDetails @external
                    categories: [String!]! @external
                    dummy(input: ProductInput! @require(field: "{ productCode: details.code productCategories: categories }")): JSON @resolve
                }

                type ProductDetails @external {
                    code: String!
                }

                input ProductInput {
                    productCode: String!
                    productCategories: [String!]!
                }

                scalar JSON
                "#,
            )
            .with_extension(Resolve::with(Ok))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "dummy": {
                  "input": {
                    "productCode": "I1",
                    "productCategories": [
                      "C1",
                      "C11"
                    ]
                  }
                }
              },
              {
                "id": "2",
                "dummy": {
                  "input": {
                    "productCode": "I2",
                    "productCategories": [
                      "C2",
                      "C22"
                    ]
                  }
                }
              }
            ]
          }
        }
        "#);
    })
}
