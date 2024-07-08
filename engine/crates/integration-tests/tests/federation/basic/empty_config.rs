use integration_tests::{
    federation::{TestFederationEngine, TestRuntime},
    runtime,
};
use parser_sdl::federation::FederatedGraphConfig;
use std::{future::IntoFuture, sync::Arc};

#[test]
fn works_with_empty_config() {
    let federated_graph =
        graphql_federated_graph::FederatedGraph::V3(graphql_federated_graph::FederatedGraphV3::default());

    let federated_graph_config = FederatedGraphConfig::default();

    let config = engine_config_builder::build_config(&federated_graph_config, federated_graph);
    let gateway = TestFederationEngine::new(Arc::new(runtime().block_on(engine_v2::Engine::new(
        Arc::new(config.into_latest().try_into().unwrap()),
        None,
        TestRuntime::default(),
    ))));

    let request = r#"{ __typename }"#;

    let response = runtime().block_on(gateway.execute(request).into_future());

    assert_eq!("Query", response.body["data"]["__typename"]);
}
