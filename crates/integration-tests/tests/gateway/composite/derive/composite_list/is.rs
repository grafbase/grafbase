use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn explicit_is() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        extend schema
                            @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key", "@is"])

                        type Query {
                            product: Product!
                        }

                        type Product {
                            id: ID!
                            invKeys: [InventoryKeys!]!
                            inventories: [Inventory!]! @derive @is(field: "invKeys[{ countryId: cId warehouseId: wId }]")
                        }

                        type InventoryKeys {
                            cId: ID!
                            wId: ID!
                        }

                        type Inventory @key(fields: "countryId warehouseId") {
                            countryId: ID!
                            warehouseId: ID!
                        }
                        "#,
                )
                .with_resolver("Query", "product", json!({"id": "product_1", "invKeys": [{"cId": "FR", "wId": "P9"}, {"cId": "US", "wId": "NY3"}]}))
                .into_subgraph("x"),
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
