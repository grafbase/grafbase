use integration_tests::{federation::TestFederationEngine, runtime};
use parser_sdl::federation::FederatedGraphConfig;
use runtime::{hooks::Hooks, trusted_documents_client};
use runtime_noop::hooks::HooksNoop;
use std::{future::IntoFuture, sync::Arc};

#[test]
fn works_with_empty_config() {
    let federated_graph =
        graphql_federated_graph::FederatedGraph::V3(graphql_federated_graph::FederatedGraphV3::default());

    let cache = runtime_local::InMemoryCache::runtime(runtime::cache::GlobalCacheConfig {
        enabled: true,
        ..Default::default()
    });

    let federated_graph_config = FederatedGraphConfig::default();

    let config = engine_config_builder::build_config(&federated_graph_config, federated_graph);
    let gateway = TestFederationEngine::new(Arc::new(engine_v2::Engine::new(
        Arc::new(config.into_latest().try_into().unwrap()),
        engine_v2::EngineEnv {
            fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
            cache: cache.clone(),
            trusted_documents: trusted_documents_client::Client::new(
                runtime_noop::trusted_documents::NoopTrustedDocuments,
            ),
            kv: runtime_local::InMemoryKvStore::runtime(),
            meter: grafbase_tracing::metrics::meter_from_global_provider(),
            hooks: Hooks::new(HooksNoop),
        },
    )));

    let request = r#"{ __typename }"#;

    let response = runtime().block_on(gateway.execute(request).into_future());

    assert_eq!("Query", response.body["data"]["__typename"]);
}
