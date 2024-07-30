use engine_v2::Engine;
use graphql_mocks::{MockGraphQlServer, StateMutationSchema, Subgraph};
use integration_tests::{federation::EngineV2Ext, runtime};

struct Stateful;

impl Subgraph for Stateful {
    fn name(&self) -> String {
        String::from("stateful")
    }

    async fn start(self) -> MockGraphQlServer {
        MockGraphQlServer::new(StateMutationSchema::default()).await
    }
}

#[test]
fn subgraph_retries_mutations_disabled() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(Stateful)
            .with_sdl_config(
                r#"
                extend schema @subgraph(
                    name: "stateful",
                    retry: {
                        minPerSecond: 1,
                        retryPercent: 0.01,
                    }
                )
            "#,
            )
            .build()
            .await;

        let response = engine.execute("query { incrementAndFailIfLessThan(n: 5) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "incrementAndFailIfLessThan": 5
          }
        }
        "###);

        // Now mutations: retries are not enabled for mutations.
        let response = engine.execute("mutation { incrementAndFailIfLessThan(n: 7) }").await;

        insta::assert_json_snapshot!(response, {
            ".errors[0].message" => "REDACTED".to_owned(),
        });

        // Queries can still be retried...
        let response = engine.execute("query { incrementAndFailIfLessThan(n: 10) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "incrementAndFailIfLessThan": 10
          }
        }
        "###);

        // But not too often.
        let response = engine.execute("query { incrementAndFailIfLessThan(n: 500) }").await;

        insta::assert_json_snapshot!(response, {
            ".errors[0].message" => "REDACTED".to_owned(),
        });
    });
}

#[test]
fn subgraph_retries_mutations_enabled() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(Stateful)
            .with_sdl_config(
                r#"
                extend schema @subgraph(
                    name: "stateful",
                    retry: {
                      retryMutations: true
                    }
                )
            "#,
            )
            .build()
            .await;

        let response = engine.execute("mutation { incrementAndFailIfLessThan(n: 5) }").await;

        insta::assert_json_snapshot!(response, {
            ".errors[0].message" => "REDACTED".to_owned(),
        });
    });
}
