use std::future::IntoFuture;

use engine::Engine;
use futures::FutureExt;
use integration_tests::{
    federation::{DockerSubgraph, EngineExt},
    runtime,
};
use pretty_assertions::assert_eq;
use serde_json::json;

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
fn sse_subgraph_subscription() {
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
            .await
            .collect()
            .await;

        insta::assert_json_snapshot!(sse_response.messages, @r###"
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
            .await
            .collect()
            .await;

        assert_eq!(sse_response.messages, multipart_response.messages);
    });
}

#[test]
fn gqlgen_subgraph_sse_subscription_with_initial_data() {
    let user = ulid::Ulid::new().to_string();
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_docker_subgraph(DockerSubgraph::Gqlgen)
            .build()
            .await;

        let post_message = |content: &'static str| {
            let engine = engine.clone();
            let user = user.clone();
            async move {
                engine
                    .clone()
                    .post("mutation($user: String!, $content: String!) { postMessage(user: $user, content: $content) }")
                    .variables(json!({"user": user, "content": content}))
                    .await
            }
        };

        let response = post_message("Hello").await;
        insta::assert_json_snapshot!(response["errors"], @"null");

        let mut subscriptions = engine
            .post(
                r"
                subscription($user: String!) {
                   message(user: $user) {
                      content
                   }
                }
                ",
            )
            .variables(json!({"user": user}))
            .into_sse_stream()
            .await;

        insta::assert_json_snapshot!(subscriptions.next().await, @r#"
        {
          "data": {
            "message": {
              "content": "Hello"
            }
          }
        }
        "#);

        let next_message = subscriptions.next();
        let delayed_message = async {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            post_message("Hi").await
        };

        let (msg, post) = tokio::join!(next_message, delayed_message);
        insta::assert_json_snapshot!(post["errors"], @"null");
        insta::assert_json_snapshot!(msg, @r#"
        {
          "data": {
            "message": {
              "content": "Hi"
            }
          }
        }
        "#);
    });
}

#[test]
fn gqlgen_subgraph_sse_subscription_without_initial_data() {
    let user = ulid::Ulid::new().to_string();
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_docker_subgraph(DockerSubgraph::Gqlgen)
            .build()
            .await;

        let post_message = |content: &'static str| {
            let engine = engine.clone();
            let user = user.clone();
            async move {
                engine
                    .clone()
                    .post("mutation($user: String!, $content: String!) { postMessage(user: $user, content: $content) }")
                    .variables(json!({"user": user, "content": content}))
                    .await
            }
        };

        let message = engine
            .post(
                r"
                subscription($user: String!) {
                   message(user: $user) {
                      content
                   }
                }
                ",
            )
            .variables(json!({"user": user}))
            .into_sse_stream()
            .into_future()
            .then(|mut stream| async move { stream.next().await });

        let delayed_message = async {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            post_message("Hello").await
        };

        let (msg, post) = tokio::join!(message, delayed_message);

        insta::assert_json_snapshot!(msg, @r#"
        {
          "data": {
            "message": {
              "content": "Hello"
            }
          }
        }
        "#);
        insta::assert_json_snapshot!(post["errors"], @"null");
    });
}
