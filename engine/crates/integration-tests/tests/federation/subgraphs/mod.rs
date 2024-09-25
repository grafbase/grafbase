mod interface_object;
mod not_reachable;
mod overrride;
mod provides;
mod requires;
mod sibling_dependencies;
mod simple_key;

use engine_v2::Engine;
use graphql_mocks::{
    FederatedAccountsSchema, FederatedInventorySchema, FederatedProductsSchema, FederatedReviewsSchema,
    FederatedShippingSchema,
};
use integration_tests::{
    federation::{EngineV2Ext, GraphqlResponse},
    runtime,
};
use serde_json::json;

async fn execute(request: &str) -> GraphqlResponse {
    let engine = Engine::builder()
        .with_subgraph(FederatedAccountsSchema)
        .with_subgraph(FederatedProductsSchema)
        .with_subgraph(FederatedReviewsSchema)
        .with_subgraph(FederatedInventorySchema)
        .with_subgraph(FederatedShippingSchema)
        .build()
        .await;
    engine.post(request).await
}

async fn execute_with_variables(request: &str, variables: serde_json::Value) -> GraphqlResponse {
    let engine = Engine::builder()
        .with_subgraph(FederatedAccountsSchema)
        .with_subgraph(FederatedProductsSchema)
        .with_subgraph(FederatedReviewsSchema)
        .with_subgraph(FederatedInventorySchema)
        .build()
        .await;
    engine.post(request).variables(variables).await
}

#[test]
fn root_fields_from_different_subgraphs() {
    let response = runtime().block_on(execute(
        r"
        query {
            me {
                id
                username
            }
            topProducts {
                name
                price
            }
        }
        ",
    ));

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "me": {
          "id": "1234",
          "username": "Me"
        },
        "topProducts": [
          {
            "name": "Trilby",
            "price": 11
          },
          {
            "name": "Fedora",
            "price": 22
          },
          {
            "name": "Boater",
            "price": 33
          },
          {
            "name": "Jeans",
            "price": 44
          },
          {
            "name": "Pink Jeans",
            "price": 55
          }
        ]
      }
    }
    "###);
}

#[test]
fn root_fragment_on_different_subgraphs() {
    let response = runtime().block_on(execute(
        r"
        query {
            ...Test
        }

        fragment Test on Query {
            me {
                id
                username
            }
            topProducts {
                name
                price
            }
        }
        ",
    ));

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "me": {
          "id": "1234",
          "username": "Me"
        },
        "topProducts": [
          {
            "name": "Trilby",
            "price": 11
          },
          {
            "name": "Fedora",
            "price": 22
          },
          {
            "name": "Boater",
            "price": 33
          },
          {
            "name": "Jeans",
            "price": 44
          },
          {
            "name": "Pink Jeans",
            "price": 55
          }
        ]
      }
    }
    "###);
}

#[test]
fn skip_test() {
    let response = runtime().block_on(execute_with_variables(
        r#"
            query Test($skipping: Boolean = true) {
                me {
                    ... on User @skip(if: $skipping) @include(if: true) {
                        id @skip(if: true)
                        id @skip(if: false)
                        username
                    }
                }
                topProducts {
                    ...TopProductFields @skip(if: $skipping) @include(if: true)
                }
            }

            fragment TopProductFields on Product {
                name
                price
            }
        "#
        .trim(),
        json!({}),
    ));

    dbg!(response);
}
