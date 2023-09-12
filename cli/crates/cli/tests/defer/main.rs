#![allow(unused_crate_dependencies)]
#[path = "../utils/mod.rs"]
mod utils;

use backend::project::ConfigType;
use futures_util::StreamExt;
use reqwest_eventsource::RequestBuilderExt;
use serde_json::Value;
use utils::{async_client::AsyncClient, environment::Environment};

#[tokio::test(flavor = "multi_thread")]
async fn defer_multipart_test() {
    // Tests that deferring with a multipart transport works..

    let schema = "type Todo @model { title: String }";
    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema).await;

    let response = client
        .gql::<Value>(
            r#"
                mutation {
                    one: todoCreate(input: {title: "Defer Things"}) { todo { id } }
                    two: todoCreate(input: {title: "Defer Things"}) { todo { id } }
                }
            "#,
        )
        .await;

    assert!(dot_get_opt!(response, "errors", Vec<Value>).is_none(), "{response:?}");

    let response = client
        .gql::<Value>(
            r#"
                    query {
                        todoCollection(first: 1) {
                            __typename
                            edges {
                                node {
                                    title
                                }
                            }
                        }
                        ... @defer {
                            deferred: todoCollection(first: 10) {
                                __typename
                                edges {
                                    node {
                                        title
                                    }
                                }
                            }
                        }
                    }
                "#,
        )
        .into_reqwest_builder()
        .header("accept", "multipart/mixed")
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "multipart/mixed; boundary=\"-\""
    );

    let parts = multipart_stream::parse(response.bytes_stream(), "-")
        .map(|result| serde_json::from_slice::<Value>(&result.unwrap().body).unwrap())
        .collect::<Vec<_>>()
        .await;

    insta::assert_json_snapshot!(parts, @r###"
    [
      {
        "data": {
          "todoCollection": {
            "__typename": "TodoConnection",
            "edges": [
              {
                "node": {
                  "title": "Defer Things"
                }
              }
            ]
          }
        }
      },
      {
        "data": {
          "deferred": {
            "__typename": "TodoConnection",
            "edges": [
              {
                "node": {
                  "title": "Defer Things"
                }
              },
              {
                "node": {
                  "title": "Defer Things"
                }
              }
            ]
          }
        },
        "path": [],
        "hasNext": false
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn defer_sse_test() {
    let schema = "type Todo @model { title: String }";
    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema).await;

    let response = client
        .gql::<Value>(
            r#"
                mutation {
                    one: todoCreate(input: {title: "Defer Things"}) { todo { id } }
                    two: todoCreate(input: {title: "Defer Things"}) { todo { id } }
                }
            "#,
        )
        .await;

    assert!(dot_get_opt!(response, "errors", Vec<Value>).is_none(), "{response:?}");

    let events = client
        .gql::<Value>(
            r#"
                    query {
                        todoCollection(first: 1) {
                            __typename
                            edges {
                                node {
                                    title
                                }
                            }
                        }
                        ... @defer {
                            deferred: todoCollection(first: 10) {
                                __typename
                                edges {
                                    node {
                                        title
                                    }
                                }
                            }
                        }
                    }
                "#,
        )
        .into_reqwest_builder()
        .eventsource()
        .unwrap()
        .take_while(|event| {
            let mut complete = false;
            let event = event.as_ref().unwrap();
            if let reqwest_eventsource::Event::Message(message) = event {
                complete = message.event == "complete";
            };
            async move { !complete }
        })
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect::<Vec<_>>();

    insta::assert_debug_snapshot!(events, @r###"
    [
        Open,
        Message(
            Event {
                event: "next",
                data: "{\"data\":{\"todoCollection\":{\"__typename\":\"TodoConnection\",\"edges\":[{\"node\":{\"title\":\"Defer Things\"}}]}}}",
                id: "",
                retry: None,
            },
        ),
        Message(
            Event {
                event: "next",
                data: "{\"data\":{\"deferred\":{\"__typename\":\"TodoConnection\",\"edges\":[{\"node\":{\"title\":\"Defer Things\"}},{\"node\":{\"title\":\"Defer Things\"}}]}},\"path\":[],\"hasNext\":false}",
                id: "",
                retry: None,
            },
        ),
    ]
    "###);
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str>) -> AsyncClient {
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(schema);
    env.set_variables([("API_KEY", "BLAH")]);
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
}
