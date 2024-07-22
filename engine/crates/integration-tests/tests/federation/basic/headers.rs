//! Tests of header forwarding behaviour

use engine_v2::Engine;
use graphql_mocks::{EchoSchema, MockGraphQlServer};
use integration_tests::{federation::EngineV2Ext, runtime};
use parser_sdl::federation::header::{
    NameOrPattern, SubgraphHeaderForward, SubgraphHeaderInsert, SubgraphHeaderRemove, SubgraphHeaderRule,
    SubgraphRenameDuplicate,
};
use regex::Regex;

#[test]
fn test_default_headers() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
            .with_supergraph_config(
                r#"
                    extend schema
                        @allSubgraphs(headers: [
                            {name: "x-foo", value: "BAR"}
                        ])
                "#,
            )
            .build()
            .await;

        engine.execute("query { headers { name value }}").await
    });

    insta::assert_json_snapshot!(response, { "data.headers."}, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-foo",
            "value": "BAR"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_default_headers_forwarding() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-foo",
            "value": "BAR"
          },
          {
            "name": "x-forwarded",
            "value": "boom"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_subgraph_specific_header_forwarding() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
            .with_supergraph_config(
                r#"
                    extend schema
                        @subgraph(name: "other", headers: [
                            {name: "boop", value: "bleep"}
                        ])
                        @subgraph(name: "echo", headers: [
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-foo",
            "value": "BAR"
          },
          {
            "name": "x-forwarded",
            "value": "boom"
          }
        ]
      }
    }
    "###);
}

#[test]
fn should_not_propagate_blacklisted_headers() {
    runtime().block_on(async move {
        let echo_mock = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_mock)
            .with_header_rule(SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                name: NameOrPattern::Pattern(Regex::new(".*").unwrap()),
                default: None,
                rename: None,
            }))
            .with_header_rule(SubgraphHeaderRule::Insert(SubgraphHeaderInsert {
                name: "Content-Type".into(),
                value: "application/trust-me".into(),
            }))
            .with_header_rule(SubgraphHeaderRule::RenameDuplicate(SubgraphRenameDuplicate {
                name: "User-Agent".into(),
                default: None,
                rename: "TE".into(),
            }))
            .build()
            .await;

        let response = engine
            .execute("query { headers { name value }}")
            .header("User-Agent", "Rusty")
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("Accept-Charset", "utf-8")
            .header("Accept-Ranges", "bytes")
            .header("Content-Length", "728")
            .header("Content-Type", "application/jpeg")
            .header("Connection", "keep-alive")
            .header("Keep-Alive", "10")
            .header("Proxy-Authenticate", "Basic")
            .header("Proxy-Authorization", "Basic")
            .header("TE", "gzip")
            .header("Trailer", "gzip")
            .header("Transfer-Encoding", "gzip")
            .header("Upgrade", "foo/2")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "headers": [
              {
                "name": "accept",
                "value": "application/json"
              },
              {
                "name": "content-length",
                "value": "78"
              },
              {
                "name": "content-type",
                "value": "application/json"
              },
              {
                "name": "user-agent",
                "value": "Rusty"
              }
            ]
          }
        }
        "###);
    })
}
#[test]
fn test_regex_header_forwarding() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-some",
            "value": "meow"
          },
          {
            "name": "x-source",
            "value": "boom"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_header_forwarding_with_rename() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_subgraph("github", &github_mock)
            .with_header_rule(SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                name: NameOrPattern::Name(String::from("x-source")),
                rename: Some(String::from("y-source")),
                default: None,
            }))
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
            name: "y-source",
            value: "boom",
        },
    ]
    "###);
}

#[test]
fn test_header_forwarding_with_default() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
            .with_header_rule(SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                name: NameOrPattern::Name(String::from("x-source")),
                rename: None,
                default: Some(String::from("meow")),
            }))
            .build()
            .await;

        engine.execute("query { headers { name value }}").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-source",
            "value": "meow"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_header_forwarding_with_default_and_existing_header() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-source",
            "value": "kekw"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_regex_header_forwarding_then_delete() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-source",
            "value": "boom"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_regex_header_forwarding_then_delete_with_regex() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-kekw",
            "value": "meow"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_rename_duplicate_no_default() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
            .with_header_rule(SubgraphHeaderRule::RenameDuplicate(SubgraphRenameDuplicate {
                name: String::from("foo"),
                default: None,
                rename: String::from("bar"),
            }))
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("foo", "lol")
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "bar",
            "value": "lol"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "foo",
            "value": "lol"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_rename_duplicate_default() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
            .with_header_rule(SubgraphHeaderRule::RenameDuplicate(SubgraphRenameDuplicate {
                name: String::from("foo"),
                default: Some(String::from("kekw")),
                rename: String::from("bar"),
            }))
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("foo", "lol")
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "bar",
            "value": "lol"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "foo",
            "value": "lol"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_rename_duplicate_default_with_missing_value() {
    let response = runtime().block_on(async move {
        let echo_subgraph = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_subgraph("echo", &echo_subgraph)
            .with_header_rule(SubgraphHeaderRule::RenameDuplicate(SubgraphRenameDuplicate {
                name: String::from("foo"),
                default: Some(String::from("kekw")),
                rename: String::from("bar"),
            }))
            .build()
            .await;

        engine.execute("query { headers { name value }}").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "accept",
            "value": "application/json"
          },
          {
            "name": "bar",
            "value": "kekw"
          },
          {
            "name": "content-length",
            "value": "78"
          },
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "foo",
            "value": "kekw"
          }
        ]
      }
    }
    "###);
}
