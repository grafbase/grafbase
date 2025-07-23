mod interface_object;
mod not_reachable;
mod overrride;
mod provides;
mod requires;
mod shared_root;
mod sibling_dependencies;
mod simple_key;

use graphql_mocks::{
    FederatedAccountsSchema, FederatedInventorySchema, FederatedProductsSchema, FederatedReviewsSchema,
    FederatedShippingSchema,
};
use integration_tests::{
    gateway::{Gateway, GraphqlResponse},
    runtime,
};

async fn execute(request: &str) -> GraphqlResponse {
    let engine = Gateway::builder()
        .with_subgraph(FederatedAccountsSchema::default())
        .with_subgraph(FederatedProductsSchema::default())
        .with_subgraph(FederatedReviewsSchema::default())
        .with_subgraph(FederatedInventorySchema::default())
        .with_subgraph(FederatedShippingSchema::default())
        .build()
        .await;
    engine.post(request).await
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
