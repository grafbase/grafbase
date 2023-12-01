//! Tests of Basic GraphQL things going through our federation setup.
//!
//! This file shouldn't have much federation specific stuff in it, mostly just checking
//! that our engine supports all the things a normal GraphQL server should.

mod fragments;
mod variables;

use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext, mocks::graphql::FakeGithubSchema, runtime, MockGraphQlServer};

#[test]
fn single_field_from_single_server() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

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

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

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
fn response_with_lists() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

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
