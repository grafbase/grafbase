use engine_v2::Engine;
use graphql_mocks::FederatedProductsSchema;
use integration_tests::{federation::EngineV2Ext, runtime};
use rand::Rng;

#[test]
fn entity_caching_via_redis() {
    // Create a random key prefix so we don't clash with other tests
    let key_prefix = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>();

    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_toml_config(format!(
                r#"
                [entity_caching]
                enabled = true
                redis.url = "redis://localhost:6379"
                redis.key_prefix = "test-{key_prefix}-"
                "#,
            ))
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
