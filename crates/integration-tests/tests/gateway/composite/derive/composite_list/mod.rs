mod authorization;
mod is;
mod join;
mod skip_include;

use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

fn gql_subgraph() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
            extend schema
                @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key"])

            type Query {
                product: Product!
            }

            type Product {
                id: ID!
                inventoriesKeys: [InventoryKeys!]!
                inventories: [Inventory!]! @derive
            }

            type InventoryKeys {
                countryId: ID!
                warehouseId: ID!
            }

            type Inventory @key(fields: "countryId warehouseId") {
                countryId: ID!
                warehouseId: ID!
            }
            "#,
    )
    .with_resolver("Query", "product", json!({"id": "product_1", "inventoriesKeys": [{"countryId": "FR", "warehouseId": "P9"}, {"countryId": "US", "warehouseId": "NY3"}]}))
    .into_subgraph("x")
}

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine
            .post("query { product { id inventories { countryId warehouseId } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1",
              "inventories": [
                {
                  "countryId": "FR",
                  "warehouseId": "P9"
                },
                {
                  "countryId": "US",
                  "warehouseId": "NY3"
                }
              ]
            }
          }
        }
        "#);
    })
}

#[test]
fn both_derive_and_original_field() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine
            .post(
                "query { product { id inventoriesKeys { countryId warehouseId } inventories { countryId warehouseId } } }",
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1",
              "inventoriesKeys": [
                {
                  "countryId": "FR",
                  "warehouseId": "P9"
                },
                {
                  "countryId": "US",
                  "warehouseId": "NY3"
                }
              ],
              "inventories": [
                {
                  "countryId": "FR",
                  "warehouseId": "P9"
                },
                {
                  "countryId": "US",
                  "warehouseId": "NY3"
                }
              ]
            }
          }
        }
        "#);
    })
}

#[test]
fn typename() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine
            .post("query { product { id inventories { __typename countryId warehouseId } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1",
              "inventories": [
                {
                  "__typename": "Inventory",
                  "countryId": "FR",
                  "warehouseId": "P9"
                },
                {
                  "__typename": "Inventory",
                  "countryId": "US",
                  "warehouseId": "NY3"
                }
              ]
            }
          }
        }
        "#);
    })
}

#[test]
fn aliases() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine
            .post("query { product { id i: inventories { t: __typename c: countryId w: warehouseId } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1",
              "i": [
                {
                  "t": "Inventory",
                  "c": "FR",
                  "w": "P9"
                },
                {
                  "t": "Inventory",
                  "c": "US",
                  "w": "NY3"
                }
              ]
            }
          }
        }
        "#);
    })
}

#[test]
fn snake_case() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(DynamicSchema::builder(
                r#"
                    extend schema
                        @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key"])

                    type Query {
                        product: Product!
                    }

                    type Product {
                        id: ID!
                        inventories_keys: [InventoryKeys!]!
                        inventories: [Inventory!]! @derive
                    }

                    type InventoryKeys {
                        country_id: ID!
                        warehouse_id: ID!
                    }

                    type Inventory @key(fields: "countryId warehouseId") {
                        countryId: ID!
                        warehouseId: ID!
                    }
                    "#,
            )
            .with_resolver("Query", "product", json!({"id": "product_1", "inventories_keys": [{"country_id": "FR", "warehouse_id": "P9"}, {"country_id": "US", "warehouse_id": "NY3"}]}))
            .into_subgraph("x")
            )
            .build()
            .await;

        let response = engine.post("query { product { id inventories { countryId warehouseId } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1",
              "inventories": [
                {
                  "countryId": "FR",
                  "warehouseId": "P9"
                },
                {
                  "countryId": "US",
                  "warehouseId": "NY3"
                }
              ]
            }
          }
        }
        "#);
    })
}
