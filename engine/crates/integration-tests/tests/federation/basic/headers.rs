//! Tests of header forwarding behaviour

use engine_v2::Engine;
use graphql_mocks::EchoSchema;
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn test_default_headers() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_sdl_config(
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_sdl_config(
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_sdl_config(
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                pattern = ".*"

                [[headers]]
                rule = "insert"
                name = "Content-Type"
                value = "application/trust-me"

                [[headers]]
                rule = "rename_duplicate"
                name = "User-Agent"
                rename = "TE"
                "###,
            )
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                pattern = "^x-*"
                "###,
            )
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
fn test_regex_header_forwarding_should_not_duplicate() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                pattern = "^x-*"

                [[headers]]
                rule = "forward"
                name = "x-source"
                rename = "y-source"
                "###,
            )
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
            "name": "y-source",
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                name = "x-source"
                rename = "y-source"
                "###,
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
            "name": "y-source",
            "value": "boom"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_header_forwarding_with_default() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                name = "x-source"
                default = "meow"
                "###,
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                name = "x-source"
                default = "meow"
                "###,
            )
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                pattern = "^x-*"

                [[headers]]
                rule = "remove"
                name = "x-kekw"
                "###,
            )
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                pattern = "^x-*"

                [[headers]]
                rule = "remove"
                pattern = "^x-sou*"
                "###,
            )
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "rename_duplicate"
                name = "foo"
                rename = "bar"
                "###,
            )
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "rename_duplicate"
                name = "foo"
                default = "kekw"
                rename = "bar"
                "###,
            )
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
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "rename_duplicate"
                name = "foo"
                default = "kekw"
                rename = "bar"
                "###,
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

#[test]
fn regex_header_regex_forwarding_should_forward_duplicates_too() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                pattern = "^.*$"
                "###,
            )
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .header("x-source", "zoom")
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
          },
          {
            "name": "x-source",
            "value": "zoom"
          }
        ]
      }
    }
    "###);
}

#[test]
fn regex_header_forwarding_should_forward_duplicates() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                name = "x-source"
                "###,
            )
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .header("x-source", "zoom")
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
          },
          {
            "name": "x-source",
            "value": "zoom"
          }
        ]
      }
    }
    "###);
}

#[test]
fn regex_header_forwarding_should_forward_duplicates_with_rename() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "forward"
                name = "x-source"
                rename = "y-source"
                "###,
            )
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .header("x-source", "zoom")
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
            "name": "y-source",
            "value": "boom"
          },
          {
            "name": "y-source",
            "value": "zoom"
          }
        ]
      }
    }
    "###);
}

#[test]
fn header_remove_should_remove_duplicates() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "remove"
                name = "x-source"
                "###,
            )
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .header("x-source", "zoom")
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
          }
        ]
      }
    }
    "###);
}

#[test]
fn header_regex_remove_should_remove_duplicates() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema)
            .with_toml_config(
                r###"
                [[headers]]
                rule = "remove"
                pattern = "^x-source$"
                "###,
            )
            .build()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .header("x-source", "zoom")
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
          }
        ]
      }
    }
    "###);
}
