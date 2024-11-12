//! Tests of Basic GraphQL things going through our federation setup.
//!
//! This file shouldn't have much federation specific stuff in it, mostly just checking
//! that our engine supports all the things a normal GraphQL server should.

// mod caching;
mod empty_config;
mod enums;
mod errors;
mod fragments;
mod headers;
mod mutation;
mod operation_limits;
mod operations;
mod scalars;
mod skip_include;
mod streaming;
mod variables;

use engine_v2::Engine;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn single_field_from_single_server() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine.post("query { serverVersion }").await
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
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine.post("query { __typename }").await
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
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine
            .post(
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
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine.post("query { allBotPullRequests { title } }").await
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
