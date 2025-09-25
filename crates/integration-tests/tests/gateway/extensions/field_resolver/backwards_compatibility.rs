use integration_tests::{gateway::Gateway, runtime};

#[test]
fn resolver_0_10_0() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-10", import: ["@config", "@resolve"])
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
            .with_extension("resolver-10")
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
