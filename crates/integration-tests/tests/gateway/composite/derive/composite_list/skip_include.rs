use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use super::gql_subgraph;

#[test]
fn include_derived_field() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let query = r#"
        query($include: Boolean!) {
            product {
                id
                inventories {
                    countryId @include(if: $include)
                    warehouseId @skip(if: $include)
                }
            }
        }"#;

        let response = engine
            .post(query)
            .variables(json!({
                "include": true
            }))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1",
              "inventories": [
                {
                  "countryId": "FR"
                },
                {
                  "countryId": "US"
                }
              ]
            }
          }
        }
        "#);

        let response = engine
            .post(query)
            .variables(json!({
                "include": false
            }))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1",
              "inventories": [
                {
                  "warehouseId": "P9"
                },
                {
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
fn include_derived_entity() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let query = r#"
        query($include: Boolean!) {
            product {
                id
                inventories @include(if: $include) {
                    countryId
                    warehouseId
                }
            }
        }"#;

        let response = engine
            .post(query)
            .variables(json!({
                "include": true
            }))
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

        let response = engine
            .post(query)
            .variables(json!({
                "include": false
            }))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1"
            }
          }
        }
        "#);
    })
}

#[test]
fn include_original_batch_field() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let query = r#"
        query($include: Boolean!) {
            product {
                id
                inventoriesKeys @include(if: $include) {
                    countryId 
                    warehouseId
                }
                inventories {
                    countryId
                    warehouseId
                }
            }
        }"#;

        let response = engine
            .post(query)
            .variables(json!({
                "include": true
            }))
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

        let response = engine
            .post(query)
            .variables(json!({
                "include": false
            }))
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
fn include_original_field() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let query = r#"
        query($include: Boolean!) {
            product {
                id
                inventoriesKeys {
                    countryId @include(if: $include)
                    warehouseId @skip(if: $include)
                }
                inventories {
                    countryId
                    warehouseId
                }
            }
        }"#;

        let response = engine
            .post(query)
            .variables(json!({
                "include": true
            }))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1",
              "inventoriesKeys": [
                {
                  "countryId": "FR"
                },
                {
                  "countryId": "US"
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

        let response = engine
            .post(query)
            .variables(json!({
                "include": false
            }))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "id": "product_1",
              "inventoriesKeys": [
                {
                  "warehouseId": "P9"
                },
                {
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
