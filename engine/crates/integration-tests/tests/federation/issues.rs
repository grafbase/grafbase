use integration_tests::{federation::MockFederationEngine, runtime};
use serde_json::json;

#[test]
fn gb6873_wrong_enum_sent_to_subgraph() {
    const SDL: &str = r###"
        enum join__Graph {
          GA
            @join__graph(
              name: "b"
              url: "https://b/graphql"
            )
          GB
            @join__graph(
              name: "a"
              url: "https://a/graphql"
            )
        }

        type Query {
          order: Order @join__field(graph: GA)
          doStuff(input: SomeInput!): String! @join__field(graph: GB)
        }

        enum Order {
          ASC
          DESC
        }

        enum Dummy {
          DESCOPE
        }

        input SomeInput {
          dummy: Dummy!
          token: String!
        }
        "###;

    runtime().block_on(async move {
        let mut engine = MockFederationEngine::new(SDL).await;

        engine.mock("a", vec![json!({"data": {"doStuff": "Hi!"}})]);
        let response = engine
            .execute(
                r#"
                query RequestUserToken {
                    doStuff(
                        input: {
                            token: "<token>"
                            dummy: DESCOPE
                        }
                    )
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "doStuff": "Hi!"
          }
        }
        "###);

        let requests = engine
            .received_requests()
            .into_iter()
            .map(|req| serde_json::from_str::<serde_json::Value>(&req).unwrap())
            .collect::<Vec<_>>();
        insta::with_settings!({ sort_maps => true}, {
            insta::assert_json_snapshot!(requests, @r###"
            [
              {
                "query": "query($var0: SomeInput!) {\n  doStuff(input: $var0)\n}\n",
                "variables": {
                  "var0": {
                    "dummy": "DESCOPE",
                    "token": "<token>"
                  }
                }
              }
            ]
            "###)
        });
    });
}
