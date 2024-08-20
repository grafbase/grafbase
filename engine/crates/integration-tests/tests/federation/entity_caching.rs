use std::time::Duration;

use engine_v2::Engine;
use graphql_mocks::{ErrorSchema, FederatedInventorySchema, FederatedProductsSchema, FederatedReviewsSchema};
use integration_tests::{federation::EngineV2Ext, runtime};
use serde_json::json;

mod subgraph_cache_control;

#[test]
fn root_level_entity_caching() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = r"query { topProducts { upc name price } }";

        let first_response = engine.post(QUERY).await.into_data();
        let second_response = engine.post(QUERY).await.into_data();

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
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        let first_response = engine.post("query { topProducts { upc } }").await.into_data();
        let second_response = engine.post("query { topProducts { name } }").await.into_data();

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
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true

                [subgraphs.products.entity_caching]
                ttl = "1s"
                "#,
            )
            .build()
            .await;

        const QUERY: &str = r"query { topProducts { upc name price } }";

        let first_response = engine.post(QUERY).await.into_data();

        tokio::time::sleep(Duration::from_millis(1100)).await;

        let second_response = engine.post(QUERY).await.into_data();

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
fn cache_skipped_if_subgraph_errors() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(ErrorSchema::default())
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = "query { brokenField(error: \"blah\") }";

        let first_response = engine.post(QUERY).await;
        let second_response = engine.post(QUERY).await;

        assert!(!first_response.errors().is_empty());

        assert!(first_response.into_value() == second_response.into_value());

        assert_eq!(engine.drain_graphql_requests_sent_to::<ErrorSchema>().len(), 2);
    });
}

#[test]
fn entity_request_caching() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        let response = engine
            .post("{ topProducts { upc reviews { id body } } }")
            .await
            .into_data();
        let first_product = &response["topProducts"][0];
        let product_upc = &first_product["upc"];

        let second_response = engine
            .post("query ($upc: String!) { product(upc: $upc) { reviews { id body } } }")
            .variables(json!({ "upc": product_upc }))
            .await
            .into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            1
        );

        assert_eq!(first_product["reviews"], second_response["product"]["reviews"]);
    })
}

#[test]
fn entity_request_cache_partial_hit() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        engine
            .post("query ($upc: String!) { product(upc: $upc) { reviews { id body } } }")
            .variables(json!({ "upc": "top-1" }))
            .await
            .into_data();

        engine
            .post("{ topProducts { upc reviews { id body } } }")
            .await
            .into_data();

        // The first request here should be for top-1, and the second one should _not_ have top-1 because
        // it should have been loaded from the cache
        insta::assert_json_snapshot!(engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>(), @r###"
        [
          {
            "query": "query($var0: [_Any!]!) {\n  _entities(representations: $var0) {\n    ... on Product {\n      reviews {\n        id\n        body\n      }\n    }\n  }\n}",
            "operationName": null,
            "variables": {
              "var0": [
                {
                  "__typename": "Product",
                  "upc": "top-1"
                }
              ]
            },
            "extensions": {}
          },
          {
            "query": "query($var0: [_Any!]!) {\n  _entities(representations: $var0) {\n    ... on Product {\n      reviews {\n        id\n        body\n      }\n    }\n  }\n}",
            "operationName": null,
            "variables": {
              "var0": [
                {
                  "__typename": "Product",
                  "upc": "top-2"
                },
                {
                  "__typename": "Product",
                  "upc": "top-3"
                },
                {
                  "__typename": "Product",
                  "upc": "top-4"
                },
                {
                  "__typename": "Product",
                  "upc": "top-5"
                }
              ]
            },
            "extensions": {}
          }
        ]
        "###);
    })
}

#[test]
fn test_headers_impact_root_field_caching() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true

                [[headers]]
                rule = "forward"
                name = "authentication"
                "#,
            )
            .build()
            .await;

        const QUERY: &str = r"query { topProducts { upc name price } }";

        engine.post(QUERY).await.into_data();
        engine
            .post(QUERY)
            .header("Authentication", "Bearer 1")
            .await
            .into_data();
        engine
            .post(QUERY)
            .header("Authentication", "Bearer 2")
            .await
            .into_data();
        engine
            .post(QUERY)
            .header("Authentication", "Bearer 2")
            .await
            .into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedProductsSchema>().len(),
            3
        );
    });
}

#[test]
fn test_headers_impact_entity_field_caching() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true

                [[headers]]
                rule = "forward"
                name = "authentication"
                "#,
            )
            .build()
            .await;

        engine
            .post("{ topProducts { upc reviews { id body } } }")
            .await
            .into_data();
        engine
            .post("{ topProducts { upc reviews { id body } } }")
            .header("Authentication", "Bearer 1")
            .await
            .into_data();
        engine
            .post("{ topProducts { upc reviews { id body } } }")
            .header("Authentication", "Bearer 2")
            .await
            .into_data();
        engine
            .post("{ topProducts { upc reviews { id body } } }")
            .header("Authentication", "Bearer 2")
            .await
            .into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            3
        );
    })
}
