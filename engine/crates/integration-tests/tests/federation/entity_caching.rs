use std::time::Duration;

use engine_v2::Engine;
use graphql_mocks::{ErrorSchema, FederatedProductsSchema};
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn root_level_entity_caching() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_entity_caching()
            .build()
            .await;

        const QUERY: &str = r"query { topProducts { upc name price } }";

        let first_response = engine.execute(QUERY).await.into_data();
        let second_response = engine.execute(QUERY).await.into_data();

        assert_eq!(first_response, second_response);

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedProductsSchema>().len(),
            1
        );

        first_response
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "topProducts": [
        {
          "upc": "top-1",
          "name": "Trilby",
          "price": 11
        },
        {
          "upc": "top-2",
          "name": "Fedora",
          "price": 22
        },
        {
          "upc": "top-3",
          "name": "Boater",
          "price": 33
        },
        {
          "upc": "top-4",
          "name": "Jeans",
          "price": 44
        },
        {
          "upc": "top-5",
          "name": "Pink Jeans",
          "price": 55
        }
      ]
    }
    "###);
}

#[test]
fn different_queries_dont_share_cache() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_entity_caching()
            .build()
            .await;

        let first_response = engine.execute("query { topProducts { upc } }").await.into_data();
        let second_response = engine.execute("query { topProducts { name } }").await.into_data();

        assert!(first_response != second_response);

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedProductsSchema>().len(),
            2
        );
    });
}

#[test]
fn test_cache_expiry() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_sdl_config(r#"extend schema @subgraph(name: "products", entityCacheTtl: "1s")"#)
            .with_entity_caching()
            .build()
            .await;

        const QUERY: &str = r"query { topProducts { upc name price } }";

        let first_response = engine.execute(QUERY).await.into_data();

        tokio::time::sleep(Duration::from_millis(1100)).await;

        let second_response = engine.execute(QUERY).await.into_data();

        assert_eq!(first_response, second_response);

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedProductsSchema>().len(),
            2
        );

        first_response
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "topProducts": [
        {
          "upc": "top-1",
          "name": "Trilby",
          "price": 11
        },
        {
          "upc": "top-2",
          "name": "Fedora",
          "price": 22
        },
        {
          "upc": "top-3",
          "name": "Boater",
          "price": 33
        },
        {
          "upc": "top-4",
          "name": "Jeans",
          "price": 44
        },
        {
          "upc": "top-5",
          "name": "Pink Jeans",
          "price": 55
        }
      ]
    }
    "###);
}

#[test]
fn cache_skipped_if_downstream_error() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(ErrorSchema::default())
            .with_entity_caching()
            .build()
            .await;

        const QUERY: &str = "query { brokenField(error: \"blah\") }";

        let first_response = engine.execute(QUERY).await;
        let second_response = engine.execute(QUERY).await;

        assert!(!first_response.errors().is_empty());

        assert!(first_response.into_value() == second_response.into_value());

        assert_eq!(engine.drain_graphql_requests_sent_to::<ErrorSchema>().len(), 2);
    });
}
