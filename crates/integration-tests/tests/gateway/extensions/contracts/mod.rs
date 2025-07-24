mod sdk19;

use integration_tests::{gateway::Gateway, runtime};

#[test]
fn no_contract() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl(
                "user",
                r#"
                extend schema @link(url: "contracts-19-0.1.0", import: ["@tag"])

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
                extend schema @link(url: "contracts-19-0.1.0", import: ["@tag"])

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
                contract_key = "{\"excludedTags\": [\"internal\"]}"
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
