use engine_v2::Engine;
use integration_tests::{
    federation::EngineV2Ext,
    mocks::graphql::{FakeFederationAccountsSchema, FakeFederationProductsSchema},
    runtime, MockGraphQlServer,
};

mod simple_key;

#[test]
fn root_fields_from_different_subgraphs() {
    let response = runtime().block_on(async move {
        let accounts = MockGraphQlServer::new(FakeFederationAccountsSchema).await;
        let products = MockGraphQlServer::new(FakeFederationProductsSchema).await;

        let engine = Engine::build()
            .with_schema("accounts", &accounts)
            .await
            .with_schema("products", &products)
            .await
            .finish()
            .await;

        engine
            .execute(
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
            )
            .await
    });

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
    let response = runtime().block_on(async move {
        let accounts = MockGraphQlServer::new(FakeFederationAccountsSchema).await;
        let products = MockGraphQlServer::new(FakeFederationProductsSchema).await;

        let engine = Engine::build()
            .with_schema("accounts", &accounts)
            .await
            .with_schema("products", &products)
            .await
            .finish()
            .await;

        engine
            .execute(
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
            )
            .await
    });

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
