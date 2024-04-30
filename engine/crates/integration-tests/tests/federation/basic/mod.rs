//! Tests of Basic GraphQL things going through our federation setup.
//!
//! This file shouldn't have much federation specific stuff in it, mostly just checking
//! that our engine supports all the things a normal GraphQL server should.

// mod caching;
mod errors;
mod fragments;
mod headers;
mod mutation;
mod operation_limits;
mod scalars;
mod streaming;
mod variables;

use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{
    federation::{GatewayV2Ext, TestFederationGateway},
    runtime,
};
use parser_sdl::federation::FederatedGraphConfig;
use runtime::trusted_documents_client;
use std::sync::Arc;

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
    let gateway = TestFederationGateway {
        gateway: Arc::new(engine_v2::Engine::new(
            engine_v2::Schema::try_from(config.into_latest()).unwrap(),
            engine_v2::EngineEnv {
                fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
                cache: cache.clone(),
                trusted_documents: trusted_documents_client::Client::new(
                    runtime_noop::trusted_documents::NoopTrustedDocuments,
                ),
                kv: runtime_local::InMemoryKvStore::runtime(),
                meter: grafbase_tracing::metrics::meter_from_global_provider(),
            },
        )),
    };

    let request: engine::Request = serde_json::from_value(serde_json::json!({"query": "{ __typename }"})).unwrap();

    runtime().block_on(
        gateway
            .gateway
            .execute(Default::default(), engine::BatchRequest::Single(request)),
    );
}

#[test]
fn single_field_from_single_server() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine.execute("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "serverVersion": "1"
      }
    }
    "###);
}

#[test]
fn top_level_typename() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine.execute("query { __typename }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__typename": "Query"
      }
    }
    "###);
}

#[test]
fn only_typename() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine
            .execute(
                r#"query { 
                    pullRequestsAndIssues(filter: { search: "1" }) { __typename } 
                    allBotPullRequests { __typename } 
                }"#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "pullRequestsAndIssues": [
          {
            "__typename": "PullRequest"
          },
          {
            "__typename": "PullRequest"
          },
          {
            "__typename": "Issue"
          }
        ],
        "allBotPullRequests": [
          {
            "__typename": "PullRequest"
          },
          {
            "__typename": "PullRequest"
          }
        ]
      }
    }
    "###);
}

#[test]
fn response_with_lists() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine.execute("query { allBotPullRequests { title } }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "allBotPullRequests": [
          {
            "title": "Creating the thing"
          },
          {
            "title": "Some bot PR"
          }
        ]
      }
    }
    "###);
}
