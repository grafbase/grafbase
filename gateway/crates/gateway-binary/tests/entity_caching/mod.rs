use std::{future::Future, sync::Arc};

use graphql_mocks::Schema;
use indoc::formatdoc;
use rand::Rng;

use crate::{runtime, Client};

#[test]
fn entity_caching_via_redis() {
    // Create a random key prefix so we don't clash with other tests
    let key_prefix = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>();

    let config = formatdoc!(
        r#"
        [entity_caching]
        enabled = true
        redis.url = "redis://localhost:6379"
        redis.key_prefix = "test-{key_prefix}-"
        "#,
    );

    let subgraph_schema = graphql_mocks::EchoSchema;
    let subgraph_sdl = subgraph_schema.sdl();
    let subgraph_server = runtime().block_on(async { graphql_mocks::MockGraphQlServer::new(subgraph_schema).await });

    with_mock_subgraph(
        &config,
        &subgraph_sdl,
        subgraph_server.url().as_str(),
        |client| async move {
            const QUERY: &str = r#"query { id(input: "hello") }"#;

            let first_response = client.gql::<serde_json::Value>(QUERY).send().await;
            let second_response = client.gql::<serde_json::Value>(QUERY).send().await;

            assert_eq!(first_response, second_response);

            assert_eq!(subgraph_server.drain_received_requests().count(), 1);

            insta::assert_json_snapshot!(first_response, @r###"
            {
              "data": {
                "id": "hello"
              }
            }
            "###);
        },
    );
}

fn with_mock_subgraph<T, F>(config: &str, subgraph_schema: &str, subgraph_url: &str, test: T)
where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    let federated_schema = {
        let mut subgraphs = graphql_composition::Subgraphs::default();
        subgraphs
            .ingest_str(subgraph_schema, "the-subgraph", subgraph_url)
            .unwrap();
        graphql_composition::compose(&subgraphs)
            .into_result()
            .unwrap()
            .into_federated_sdl()
    };

    crate::GatewayBuilder {
        toml_config: config.into(),
        schema: &federated_schema,
        log_level: None,
        client_url_path: None,
        client_headers: None,
    }
    .run(test)
}
