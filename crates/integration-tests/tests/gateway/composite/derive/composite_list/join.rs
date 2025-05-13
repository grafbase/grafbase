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
                    product: Product
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
                    category: ID @shareable
                }
                "#,
                )
                .with_resolver("Query", "product", json!({"id": "product_1", "inventoriesKeys": [
                    {"countryId": "FR", "warehouseId": "P9"}, {"countryId": "US", "warehouseId": "NY3"}
                ]}))
                .into_subgraph("products"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        inventories: [Inventory!]!
                    }

                    type Inventory @key(fields: "countryId warehouseId") {
                        countryId: ID!
                        warehouseId: ID!
                        category: ID @shareable
                    }
                    "#,
                )
                .with_entity_resolver("Inventory", |ctx: EntityResolverContext<'_>| -> Option<serde_json::Value> {
                    match ctx.representation["countryId"].as_str().unwrap() {
                        "FR" => Some(json!({"category": "world class"})),
                        _ => None
                    }
                })
                .into_subgraph("comments"),
            )
            .build()
            .await;

        let response = gateway.post("{ product { inventories { countryId category } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "inventories": [
                {
                  "countryId": "FR",
                  "category": "world class"
                },
                {
                  "countryId": "US",
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
                    product: Product
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
                    category: ID @external
                }
                "#,
                )
                .with_resolver("Query", "product", json!({"id": "product_1", "inventoriesKeys": [
                    {"countryId": "FR", "warehouseId": "P9"}, {"countryId": "US", "warehouseId": "NY3"}
                ]}))
                .into_subgraph("products"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        inventories: [Inventory!]!
                    }

                    type Inventory @key(fields: "countryId warehouseId") {
                        countryId: ID!
                        warehouseId: ID!
                        category: ID
                    }
                    "#,
                )
                .with_entity_resolver("Inventory", |ctx: EntityResolverContext<'_>| -> Option<serde_json::Value> {
                    match ctx.representation["countryId"].as_str().unwrap() {
                        "FR" => Some(json!({"category": "world class"})),
                        _ => None
                    }
                })
                .into_subgraph("comments"),
            )
            .build()
            .await;

        let response = gateway.post("{ product { inventories { countryId category } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "inventories": [
                {
                  "countryId": "FR",
                  "category": "world class"
                },
                {
                  "countryId": "US",
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
