//! Tests of header forwarding behaviour

use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{federation::EngineV2Ext, runtime};
use parser_sdl::federation::header::{NameOrPattern, SubgraphHeaderForward, SubgraphHeaderRemove, SubgraphHeaderRule};
use regex::Regex;

#[derive(serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Header {
    name: String,
    value: String,
}

#[derive(serde::Deserialize)]
struct Response {
    headers: Vec<Header>,
}

#[test]
fn test_default_headers() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_subgraph("github", &github_mock)
            .with_supergraph_config(
                r#"
                    extend schema
                        @allSubgraphs(headers: [
                            {name: "x-foo", value: "BAR"}
                            {name: "x-forwarded", forward: "x-source"}
                        ])
                "#,
            )
            .build()
            .await;

        engine.execute("query { headers { name value }}").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-foo",
            "value": "BAR"
          },
          {
            "name": "accept",
            "value": "*/*"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_default_headers_forwarding() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_subgraph("github", &github_mock)
            .with_supergraph_config(
                r#"
                    extend schema
                        @allSubgraphs(headers: [
                            {name: "x-foo", value: "BAR"}
                            {name: "x-forwarded", forward: "x-source"}
                        ])
                "#,
            )
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .await
    });

    let mut response: Response = serde_json::from_value(response.into_data()).unwrap();
    response.headers.sort();

    insta::assert_debug_snapshot!(response.headers, @r###"
    [
        Header {
            name: "accept",
            value: "*/*",
        },
        Header {
            name: "content-type",
            value: "application/json",
        },
        Header {
            name: "x-foo",
            value: "BAR",
        },
        Header {
            name: "x-forwarded",
            value: "boom",
        },
    ]
    "###);
}

#[test]
fn test_subgraph_specific_header_forwarding() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_subgraph("github", &github_mock)
            .with_supergraph_config(
                r#"
                    extend schema
                        @subgraph(name: "other", headers: [
                            {name: "boop", value: "bleep"}
                        ])
                        @subgraph(name: "github", headers: [
                            {name: "x-foo", value: "BAR"}
                            {name: "x-forwarded", forward: "x-source"}
                        ])
                "#,
            )
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .await
    });

    let mut response: Response = serde_json::from_value(response.into_data()).unwrap();
    response.headers.sort();

    insta::assert_debug_snapshot!(response.headers, @r###"
    [
        Header {
            name: "accept",
            value: "*/*",
        },
        Header {
            name: "content-type",
            value: "application/json",
        },
        Header {
            name: "x-foo",
            value: "BAR",
        },
        Header {
            name: "x-forwarded",
            value: "boom",
        },
    ]
    "###);
}

#[test]
fn test_regex_header_forwarding() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_subgraph("github", &github_mock)
            .with_header_rule(SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                name: NameOrPattern::Pattern(Regex::new("^x-*").unwrap()),
                default: None,
                rename: None,
            }))
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .header("asdf", "lol")
            .header("x-some", "meow")
            .await
    });

    let mut response: Response = serde_json::from_value(response.into_data()).unwrap();
    response.headers.sort();

    insta::assert_debug_snapshot!(response.headers, @r###"
    [
        Header {
            name: "accept",
            value: "*/*",
        },
        Header {
            name: "content-type",
            value: "application/json",
        },
        Header {
            name: "x-some",
            value: "meow",
        },
        Header {
            name: "x-source",
            value: "boom",
        },
    ]
    "###);
}

#[test]
fn test_header_forwarding_with_default() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_subgraph("github", &github_mock)
            .with_header_rule(SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                name: NameOrPattern::Name(String::from("x-source")),
                rename: None,
                default: Some(String::from("meow")),
            }))
            .build()
            .await;

        engine.execute("query { headers { name value }}").await
    });

    let mut response: Response = serde_json::from_value(response.into_data()).unwrap();
    response.headers.sort();

    insta::assert_debug_snapshot!(response.headers, @r###"
    [
        Header {
            name: "accept",
            value: "*/*",
        },
        Header {
            name: "content-type",
            value: "application/json",
        },
        Header {
            name: "x-source",
            value: "meow",
        },
    ]
    "###);
}

#[test]
fn test_header_forwarding_with_default_and_existing_header() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_subgraph("github", &github_mock)
            .with_header_rule(SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                name: NameOrPattern::Name(String::from("x-source")),
                rename: None,
                default: Some(String::from("meow")),
            }))
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "kekw")
            .await
    });

    let mut response: Response = serde_json::from_value(response.into_data()).unwrap();
    response.headers.sort();

    insta::assert_debug_snapshot!(response.headers, @r###"
    [
        Header {
            name: "accept",
            value: "*/*",
        },
        Header {
            name: "content-type",
            value: "application/json",
        },
        Header {
            name: "x-source",
            value: "kekw",
        },
    ]
    "###);
}

#[test]
fn test_regex_header_forwarding_then_delete() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_subgraph("github", &github_mock)
            .with_header_rule(SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                name: NameOrPattern::Pattern(Regex::new("^x-*").unwrap()),
                default: None,
                rename: None,
            }))
            .with_header_rule(SubgraphHeaderRule::Remove(SubgraphHeaderRemove {
                name: NameOrPattern::Name(String::from("x-kekw")),
            }))
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .header("asdf", "lol")
            .header("x-kekw", "meow")
            .await
    });

    let mut response: Response = serde_json::from_value(response.into_data()).unwrap();
    response.headers.sort();

    insta::assert_debug_snapshot!(response.headers, @r###"
    [
        Header {
            name: "accept",
            value: "*/*",
        },
        Header {
            name: "content-type",
            value: "application/json",
        },
        Header {
            name: "x-source",
            value: "boom",
        },
    ]
    "###);
}

#[test]
fn test_regex_header_forwarding_then_delete_with_regex() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_subgraph("github", &github_mock)
            .with_header_rule(SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                name: NameOrPattern::Pattern(Regex::new("^x-*").unwrap()),
                default: None,
                rename: None,
            }))
            .with_header_rule(SubgraphHeaderRule::Remove(SubgraphHeaderRemove {
                name: NameOrPattern::Pattern(Regex::new("^x-sou*").unwrap()),
            }))
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .header("x-soup", "kaboom")
            .header("asdf", "lol")
            .header("x-kekw", "meow")
            .await
    });

    let mut response: Response = serde_json::from_value(response.into_data()).unwrap();
    response.headers.sort();

    insta::assert_debug_snapshot!(response.headers, @r###"
    [
        Header {
            name: "accept",
            value: "*/*",
        },
        Header {
            name: "content-type",
            value: "application/json",
        },
        Header {
            name: "x-kekw",
            value: "meow",
        },
    ]
    "###);
}
