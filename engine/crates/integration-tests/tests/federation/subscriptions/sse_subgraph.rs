use engine_v2::Engine;
use integration_tests::{
    federation::{DockerSubgraph, EngineV2Ext},
    runtime,
};
use pretty_assertions::assert_eq;

#[test]
fn docker_see_subgraph_is_working() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_docker_subgraph(DockerSubgraph::Sse)
            .build()
            .await;

        engine
            .post(
                r"
                query {
                    hello
                }
                ",
            )
            .await
    });

    insta::assert_json_snapshot!(response.body, @r###"
    {
      "data": {
        "hello": "world"
      }
    }
    "###);
}

#[test]
fn sse_supgraph_subscription() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_docker_subgraph(DockerSubgraph::Sse)
            .build()
            .await;

        let sse_response = engine
            .post(
                r"
                subscription {
                    greetings
                }
                ",
            )
            .into_sse_stream()
            .await;

        insta::assert_json_snapshot!(sse_response.collected_body, @r###"
        [
          {
            "data": {
              "greetings": "Hi"
            }
          },
          {
            "data": {
              "greetings": "Bonjour"
            }
          },
          {
            "data": {
              "greetings": "Hola"
            }
          },
          {
            "data": {
              "greetings": "Ciao"
            }
          },
          {
            "data": {
              "greetings": "Zdravo"
            }
          }
        ]
        "###);

        // Sanity check the client format has no impact.
        let multipart_response = engine
            .post(
                r"
                subscription {
                    greetings
                }
                ",
            )
            .into_multipart_stream()
            .await;

        assert_eq!(sse_response.collected_body, multipart_response.collected_body);
    });
}
