//! Tests of caching behaviour

use engine_v2::Engine;
use integration_tests::federation::GraphqlResponse;
use integration_tests::mocks::graphql::StateMutationSchema;
use integration_tests::{federation::EngineV2Ext, mocks::graphql::FakeGithubSchema, runtime, MockGraphQlServer};
use std::time::Duration;

#[test]
fn test_basic_query_caching() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(
                r#"
                    extend schema
                        @cache(rules: [
                            {maxAge: 10, types: ["Query"]}
                        ])
                "#,
            )
            .finish()
            .await;

        let response: GraphqlResponse = engine.execute("query { serverVersion }").await;

        assert_eq!(
            response.metadata.cache_config,
            Some(engine_v2::CacheConfig {
                max_age: Duration::from_secs(10),
                ..Default::default()
            })
        );

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "###);
    });
}

#[test]
fn test_field_caching() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(
                r#"
                    extend schema
                        @cache(rules: [
                            {
                                maxAge: 10,
                                types: [{
                                    name: "Query",
                                    fields: ["favoriteRepository"]
                                }]
                            }
                        ])
                "#,
            )
            .finish()
            .await;

        let response: GraphqlResponse = engine.execute("query { serverVersion }").await;
        assert_eq!(response.metadata.cache_config, None);

        let response: GraphqlResponse = engine.execute("query { favoriteRepository }").await;
        assert_eq!(
            response.metadata.cache_config,
            Some(engine_v2::CacheConfig {
                max_age: Duration::from_secs(10),
                ..Default::default()
            })
        );

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "favoriteRepository": {
              "owner": "rust-lang",
              "name": "rust"
            }
          }
        }
        "###);
    });
}

#[test]
fn test_object_caching() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(
                r#"
                    extend schema
                        @cache(rules: [
                            {
                                maxAge: 10,
                                types: "PullRequest"
                            }
                        ])
                "#,
            )
            .finish()
            .await;

        let response: GraphqlResponse = engine.execute(r#"query { botPullRequests(bots: []) { title } }"#).await;
        assert_eq!(
            response.metadata.cache_config,
            Some(engine_v2::CacheConfig {
                max_age: Duration::from_secs(10),
                ..Default::default()
            })
        );

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "botPullRequests": [
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
    });
}

#[test]
fn test_non_object_caching() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(
                r#"
                    extend schema
                        @cache(rules: [
                            {
                                maxAge: 10,
                                types: "PullRequestOrIssue" # this is an interface
                            }
                        ])
                "#,
            )
            .finish()
            .await;

        let response: GraphqlResponse = engine
            .execute(r#"query { pullRequestsAndIssues(filter: { search: "1" }) { title } }"#)
            .await;
        assert_eq!(response.metadata.cache_config, None);

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "pullRequestsAndIssues": [
              {
                "title": "Creating the thing"
              },
              {
                "title": "Some bot PR"
              },
              {
                "title": "Everythings fine"
              }
            ]
          }
        }
        "###);
    });
}

#[test]
fn test_min_object_field_caching() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(
                r#"
                    extend schema
                        @cache(rules: [
                            {
                                maxAge: 10,
                                types: "PullRequest"
                            },
                            {
                                maxAge: 5,
                                types: [{
                                    name: "PullRequest",
                                    fields: ["title"]
                                }]
                            }
                        ])
                "#,
            )
            .finish()
            .await;

        let response: GraphqlResponse = engine.execute(r#"query { botPullRequests(bots: []) { title } }"#).await;
        assert_eq!(
            response.metadata.cache_config,
            Some(engine_v2::CacheConfig {
                max_age: Duration::from_secs(5),
                ..Default::default()
            })
        );

        let response: GraphqlResponse = engine
            .execute(r#"query { botPullRequests(bots: []) { checks } }"#)
            .await;
        assert_eq!(
            response.metadata.cache_config,
            Some(engine_v2::CacheConfig {
                max_age: Duration::from_secs(10),
                ..Default::default()
            })
        );

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "botPullRequests": [
              {
                "checks": [
                  "Success!"
                ]
              },
              {
                "checks": [
                  "Success!"
                ]
              }
            ]
          }
        }
        "###);
    });
}

#[test]
fn test_no_caching() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("github", &github_mock).await.finish().await;

        let response: GraphqlResponse = engine.execute(r#"query { botPullRequests(bots: []) { title } }"#).await;
        assert_eq!(response.metadata.cache_config, None);

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "botPullRequests": [
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
    });
}

#[test]
fn test_no_caching_on_mutation() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(StateMutationSchema::default()).await;

        let engine = Engine::build().with_schema("github", &github_mock).await.finish().await;

        let response = engine
            .execute(
                r"
                mutation {
                    first: set(val: 1)
                    second: multiply(by: 2)
                    third: multiply(by: 7)
                    fourth: set(val: 3)
                    fifth: multiply(by: 11)
                }
                ",
            )
            .await;

        assert_eq!(response.metadata.cache_config, None);

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "first": 1,
            "second": 2,
            "third": 14,
            "fourth": 3,
            "fifth": 33
          }
        }
        "###);
    });
}
