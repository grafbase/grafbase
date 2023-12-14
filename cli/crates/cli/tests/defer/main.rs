#![allow(unused_crate_dependencies, clippy::panic)]
#[path = "../utils/mod.rs"]
mod utils;

use std::collections::HashMap;

use backend::project::GraphType;
use futures_util::StreamExt;
use reqwest_eventsource::RequestBuilderExt;
use serde_json::Value;
use utils::{async_client::AsyncClient, environment::Environment};

use crate::utils::consts::AUTH_QUERY_TODOS;

const SCHEMA: &str = r#"
type Todo {
    id: ID!
    title: String
}

extend type Query {
    todoCollection(first: Int!): [Todo!]! @resolver(name: "todos")
}
"#;

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn defer_multipart_test() {
    // Tests that deferring with a multipart transport works..

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, SCHEMA).await;

    let response = client
        .gql::<Value>(
            r"
                    query {
                        todoCollection(first: 1) {
                            __typename
                            id
                            title
                        }
                        ... @defer {
                            deferred: todoCollection(first: 10) {
                                __typename
                                id
                                title
                            }
                        }
                    }
                ",
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
          "todoCollection": [
            {
              "__typename": "Todo",
              "id": "1",
              "title": "Defer Things"
            }
          ]
        },
        "hasNext": true
      },
      {
        "data": {
          "deferred": [
            {
              "__typename": "Todo",
              "id": "1",
              "title": "Defer Things"
            },
            {
              "__typename": "Todo",
              "id": "2",
              "title": "Defer Things"
            }
          ]
        },
        "hasNext": false,
        "path": []
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn defer_sse_test() {
    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, SCHEMA).await;

    let events = client
        .gql::<Value>(
            r"
                    query {
                        todoCollection(first: 1) {
                            __typename
                            title
                            id
                        }
                        ... @defer {
                            deferred: todoCollection(first: 10) {
                                __typename
                                title
                                id
                            }
                        }
                    }
                ",
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
                data: "{\"data\":{\"todoCollection\":[{\"__typename\":\"Todo\",\"title\":\"Defer Things\",\"id\":\"1\"}]},\"hasNext\":true}",
                id: "",
                retry: None,
            },
        ),
        Message(
            Event {
                event: "next",
                data: "{\"data\":{\"deferred\":[{\"__typename\":\"Todo\",\"title\":\"Defer Things\",\"id\":\"1\"},{\"__typename\":\"Todo\",\"title\":\"Defer Things\",\"id\":\"2\"}]},\"path\":[],\"hasNext\":false}",
                id: "",
                retry: None,
            },
        ),
    ]
    "###);
}

const JWT_SCHEMA: &str = r#"
  schema
    @auth(
      providers: [{ type: jwt, issuer: "{{ env.ISSUER_URL }}", secret: "{{ env.JWT_SECRET }}" }]
      rules: [{ allow: groups, groups: ["backend"] }]
    ) {
    query: Query
  }

  type Todo @model {
    id: ID!
    title: String!
  }
"#;

const JWT_ISSUER_URL: &str = "https://some.issuer.test";
const JWT_SECRET: &str = "topsecret";

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn test_auth_with_multipart() {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(JWT_SCHEMA);
    env.set_variables(HashMap::from([
        ("ISSUER_URL".to_string(), JWT_ISSUER_URL.to_string()),
        ("JWT_SECRET".to_string(), JWT_SECRET.to_string()),
    ]));
    env.grafbase_dev();

    let client = env.create_async_client();
    client.poll_endpoint(30, 300).await;

    let response = client
        .gql::<Value>(AUTH_QUERY_TODOS)
        .into_reqwest_builder()
        .header("accept", "multipart/mixed")
        .send()
        .await
        .unwrap();

    let parts = multipart_stream::parse(response.bytes_stream(), "-")
        .map(|result| serde_json::from_slice::<Value>(&result.unwrap().body).unwrap())
        .collect::<Vec<_>>()
        .await;

    assert_eq!(parts.len(), 1);

    let error: String = dot_get_opt!(parts[0], "errors.0.message").expect("should end with an auth failure");
    assert!(error.contains("Unauthorized"), "error: {error:#?}");
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn test_auth_with_sse() {
    // Tests that authentication with the SSE transport works..

    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(JWT_SCHEMA);
    env.set_variables(HashMap::from([
        ("ISSUER_URL".to_string(), JWT_ISSUER_URL.to_string()),
        ("JWT_SECRET".to_string(), JWT_SECRET.to_string()),
    ]));
    env.grafbase_dev();

    let client = env.create_async_client();
    client.poll_endpoint(30, 300).await;

    let response = client
        .gql::<Value>(AUTH_QUERY_TODOS)
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

    assert_eq!(response.len(), 2);

    let reqwest_eventsource::Event::Message(event) = &response[1] else {
        panic!("resposne wasn't a message");
    };

    let json_data = serde_json::from_str::<Value>(&event.data).unwrap();

    let error: String = dot_get_opt!(json_data, "errors.0.message").expect("should end with an auth failure");
    assert!(error.contains("Unauthorized"), "error: {error:#?}");
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str>) -> AsyncClient {
    env.grafbase_init(GraphType::Single);
    env.write_schema(schema);
    env.set_variables([("API_KEY", "BLAH")]);
    env.grafbase_dev_watch();
    env.write_file(
        "resolvers/todos.js",
        r#"
            export default function Resolver(_, {first}) {
                const data = [
                    {"id": "1", "title": "Defer Things"},
                    {"id": "2", "title": "Defer Things"},
                ];
                return data.slice(0, first);
            }
    "#,
    );

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
}
