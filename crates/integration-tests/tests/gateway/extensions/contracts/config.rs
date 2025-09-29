use integration_tests::{gateway::Gateway, runtime};

#[test]
fn no_contract() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl(
                "user",
                r#"
                extend schema @link(url: "contracts-19", import: ["@tag"])

                type Query {
                    public: ID! @tag(name: "public")
                    private: ID! @tag(name: "internal")
                }
            "#,
            )
            .with_extension("contracts-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let response = gateway.introspect().await;
        insta::assert_snapshot!(response, @r#"
        type Query {
          private: ID!
          public: ID!
        }
        "#);
    });
}

#[test]
fn static_contract() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl(
                "user",
                r#"
                extend schema @link(url: "contracts-19", import: ["@tag"])

                type Query {
                    public: ID! @tag(name: "public")
                    private: ID! @tag(name: "internal")
                }
            "#,
            )
            .with_extension("contracts-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                contracts.default_key = "{\"excludedTags\": [\"internal\"]}"
                "#,
            )
            .build()
            .await;

        let response = gateway.introspect().await;
        insta::assert_snapshot!(response, @r#"
        type Query {
          public: ID!
        }
        "#);
    });
}

#[test]
fn no_cache_size() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl(
                "user",
                r#"
                extend schema @link(url: "contracts-19", import: ["@tag"])

                type Query {
                    public: ID! @tag(name: "public")
                    private: ID! @tag(name: "internal")
                }
            "#,
            )
            .with_extension("contracts-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true

                [graph.contracts]
                default_key = "{\"excludedTags\": [\"internal\"]}"
                cache.max_size = 0
                "#,
            )
            .build()
            .await;

        let response = gateway.introspect().await;
        insta::assert_snapshot!(response, @r#"
        type Query {
          public: ID!
        }
        "#);

        // Not sure how to check it's called twice though.
        let response = gateway.introspect().await;
        insta::assert_snapshot!(response, @r#"
        type Query {
          public: ID!
        }
        "#);
    });
}

#[test]
fn invalid_cache_size() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl(
                "user",
                r#"
                extend schema @link(url: "contracts-19", import: ["@tag"])

                type Query {
                    public: ID! @tag(name: "public")
                    private: ID! @tag(name: "internal")
                }
            "#,
            )
            .with_extension("contracts-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true

                [graph.contracts]
                default_key = "{\"excludedTags\": [\"internal\"]}"
                cache.max_size = -1
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(gateway.unwrap_err(), @r#"
        Failed to parse configuration: invalid value: integer `-1`, expected usize
        in `graph.contracts.cache.max_size`
        "#);
    });
}

#[test]
fn invalid_contract() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl(
                "user",
                r#"
                extend schema @link(url: "contracts-19", import: ["@tag"])

                type Query {
                    public: ID! @tag(name: "public")
                    private: ID! @tag(name: "internal")
                }
            "#,
            )
            .with_extension("contracts-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true

                [graph.contracts]
                default_key = ""
                "#,
            )
            .build()
            .await;

        let response = gateway
            .post(cynic_introspection::IntrospectionQuery::with_capabilities(
                cynic_introspection::SpecificationVersion::October2021.capabilities(),
            ))
            .await;
        insta::assert_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Internal extension error",
              "extensions": {
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);
    });
}
