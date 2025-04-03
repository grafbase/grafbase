use integration_tests::{gateway::Gateway, runtime};

#[test]
fn resolver_080() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-08-1.0.0", import: ["@config", "@resolve"])
                    @config(id: 879)

                type Query {
                    greeting: X @resolve(name: "hi")
                }

                type X {
                    id: Int,
                    name: String
                }
                "#,
            )
            .with_extension("resolver-08")
            .build()
            .await;

        let response = engine.post("query { greeting { id name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greeting": {
              "id": 879,
              "name": "hi"
            }
          }
        }
        "#);
    });
}

#[test]
fn resolver_090() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-09-1.0.0", import: ["@config", "@resolve"])
                    @config(id: 879)

                type Query {
                    greeting: X @resolve(name: "hi")
                }

                type X {
                    id: Int,
                    name: String
                }
                "#,
            )
            .with_extension("resolver-09")
            .build()
            .await;

        let response = engine.post("query { greeting { id name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greeting": {
              "id": 879,
              "name": "hi"
            }
          }
        }
        "#);
    });
}
