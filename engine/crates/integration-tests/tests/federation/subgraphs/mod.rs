use engine_v2::Engine;
use graphql_mocks::{
    FakeFederationAccountsSchema, FakeFederationProductsSchema, FakeFederationReviewsSchema, MockGraphQlServer,
};
use integration_tests::{
    federation::{EngineV2Ext, GraphqlResponse},
    runtime,
};

mod sibling_dependencies;
mod simple_key;

async fn execute(request: &str) -> GraphqlResponse {
    let accounts = MockGraphQlServer::new(FakeFederationAccountsSchema).await;
    let products = MockGraphQlServer::new(FakeFederationProductsSchema).await;
    let reviews = MockGraphQlServer::new(FakeFederationReviewsSchema).await;

    let engine = Engine::builder()
        .with_schema("accounts", &accounts)
        .await
        .with_schema("products", &products)
        .await
        .with_schema("reviews", &reviews)
        .await
        .finish()
        .await;
    engine.execute(request).await
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
