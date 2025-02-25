use std::time::Duration;

use async_nats::{jetstream::stream::Config, ConnectOptions};
use futures::StreamExt;
use grafbase_sdk::test::{DynamicSchema, ExtensionOnlySubgraph, TestConfig, TestRunner};
use indoc::{formatdoc, indoc};
use serde_json::json;

const CLI_PATH: &str = "../../target/debug/grafbase";
const GATEWAY_PATH: &str = "../../target/debug/grafbase-gateway";

async fn nats_client() -> async_nats::Client {
    let opts = ConnectOptions::new().user_and_password("grafbase".to_string(), "grafbase".to_string());
    let addrs = vec!["nats://localhost:4222"];

    async_nats::connect_with_options(addrs, opts).await.unwrap()
}

fn subgraph() -> ExtensionOnlySubgraph {
    let extension_path = std::env::current_dir().unwrap().join("build");
    let path_str = format!("file://{}", extension_path.display());

    let schema = formatdoc! {r#"
        extend schema
          @link(url: "https://specs.apollo.dev/federation/v2.0", import: ["@key", "@shareable"])
          @link(
            url: "{path_str}",
            import: ["@natsPublish", "@natsSubscription", "NatsPublishResult", "NatsStreamDeliverPolicy"]
          )

        type Query {{
          hello: String!
        }}

        type Mutation {{
          publishUserEvent(id: Int!, input: UserEventInput!): NatsPublishResult! @natsPublish(
            subject: "publish.user.{{{{args.id}}}}.events"
          )
        }}

        type Subscription {{
          userEvents(id: Int!): UserEvent! @natsSubscription(
            subject: "subscription.user.{{{{args.id}}}}.events"
            selection: "{{ email, name, number }}"
          )

          persistenceEvents(id: Int!): UserEvent! @natsSubscription(
            subject: "persistence.user.{{{{args.id}}}}.events"
            selection: "{{ email, name, number }}"
            streamConfig: {{
              streamName: "testStream"
              consumerName: "testConsumer"
              durableName: "testConsumer"
              description: "Test Description"
            }}
          )

          nonexistingEvents(id: Int!): UserEvent! @natsSubscription(
            subject: "persistence.user.{{{{args.id}}}}.events"
            selection: "{{ email, name, number }}"
            streamConfig: {{
              streamName: "nonExistingStream"
              consumerName: "testConsumer"
              durableName: "testConsumer"
              description: "Test Description"
            }}
          )
        }}

        type NatsPublishResult {{
          success: Boolean!
        }}

        input UserEventInput {{
          email: String!
          name: String!
        }}

        type UserEvent {{
          email: String!
          name: String!
          number: Int!
        }}
    "#};

    DynamicSchema::builder(schema)
        .into_extension_only_subgraph("test", &extension_path)
        .unwrap()
}

fn config() -> &'static str {
    indoc! {r#"
        [[extensions.nats.config.endpoint]]
        name = "default"
        servers = ["nats://localhost:4222"]

        [extensions.nats.config.endpoint.authentication]
        username = "grafbase"
        password = "grafbase"
    "#}
}

#[tokio::test]
async fn test_subscribe() {
    let nats = nats_client().await;

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config())
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        subscription {
          userEvents(id: 1) {
            email
            name
            number
          }
        }
    "#};

    let subscription1 = runner
        .graphql_subscription::<serde_json::Value>(query)
        .unwrap()
        .subscribe()
        .await
        .unwrap();

    let query = indoc! {r#"
        subscription {
          userEvents(id: 2) {
            email
            name
            number
          }
        }
    "#};

    let subscription2 = runner
        .graphql_subscription::<serde_json::Value>(query)
        .unwrap()
        .subscribe()
        .await
        .unwrap();

    tokio::spawn(async move {
        for i in 0.. {
            let event1 = json!({ "email": "user1@example.com", "name": "User One", "number": i });
            let event2 = json!({ "email": "user2@example.com", "name": "User Two", "number": i });

            let event1 = serde_json::to_vec(&event1).unwrap();
            let event2 = serde_json::to_vec(&event2).unwrap();

            nats.publish("subscription.user.1.events", event1.into()).await.unwrap();
            nats.publish("subscription.user.2.events", event2.into()).await.unwrap();

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let events = tokio::time::timeout(Duration::from_secs(5), subscription1.take(2).collect::<Vec<_>>())
        .await
        .unwrap();

    insta::assert_json_snapshot!(&events, @r#"
    [
      {
        "data": {
          "userEvents": {
            "email": "user1@example.com",
            "name": "User One",
            "number": 1
          }
        }
      },
      {
        "data": {
          "userEvents": {
            "email": "user1@example.com",
            "name": "User One",
            "number": 2
          }
        }
      }
    ]
    "#);

    let events = tokio::time::timeout(Duration::from_secs(5), subscription2.take(2).collect::<Vec<_>>())
        .await
        .unwrap();

    insta::assert_json_snapshot!(&events, @r#"
    [
      {
        "data": {
          "userEvents": {
            "email": "user2@example.com",
            "name": "User Two",
            "number": 1
          }
        }
      },
      {
        "data": {
          "userEvents": {
            "email": "user2@example.com",
            "name": "User Two",
            "number": 2
          }
        }
      }
    ]
    "#);
}

#[tokio::test]
async fn test_publish() {
    let nats = nats_client().await;
    let mut subscriber = nats.subscribe("publish.user.>").await.unwrap();

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config())
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        mutation {
          publishUserEvent(id: 1, input: { email: "alice@example.com", name: "Alice" }) {
            success
          }
        }
    "#};

    let result: serde_json::Value = runner.graphql_query(query).send().await.unwrap();
    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "publishUserEvent": {
          "success": true
        }
      }
    }
    "#);

    let event = subscriber.next().await.unwrap();
    assert_eq!(event.subject.as_str(), "publish.user.1.events");

    let event: serde_json::Value = serde_json::from_slice(event.payload.as_ref()).unwrap();
    insta::assert_json_snapshot!(&event, @r#"
    {
      "email": "alice@example.com",
      "name": "Alice"
    }
    "#);
}

#[tokio::test]
async fn test_existing_stream() {
    let nats = nats_client().await;
    let context = async_nats::jetstream::new(nats);

    let _ = context.delete_stream("testStream").await;

    context
        .create_stream(Config {
            name: String::from("testStream"),
            subjects: vec![String::from("persistence.user.1.events")],
            ..Default::default()
        })
        .await
        .unwrap();

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config())
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    tokio::spawn(async move {
        for i in 1.. {
            let event = json!({ "email": "user1@example.com", "name": "User One", "number": i });
            let event = serde_json::to_vec(&event).unwrap();

            context
                .publish("persistence.user.1.events", event.into())
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let query = indoc! {r#"
        subscription {
          persistenceEvents(id: 1) {
            email
            name
            number
          }
        }
    "#};

    let subscription = runner
        .graphql_subscription::<serde_json::Value>(query)
        .unwrap()
        .subscribe()
        .await
        .unwrap();

    let events = tokio::time::timeout(Duration::from_secs(5), subscription.take(2).collect::<Vec<_>>())
        .await
        .unwrap();

    insta::assert_json_snapshot!(&events, @r#"
    [
      {
        "data": {
          "persistenceEvents": {
            "email": "user1@example.com",
            "name": "User One",
            "number": 1
          }
        }
      },
      {
        "data": {
          "persistenceEvents": {
            "email": "user1@example.com",
            "name": "User One",
            "number": 2
          }
        }
      }
    ]
    "#);
}

#[tokio::test]
async fn test_non_existing_stream() {
    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config())
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        subscription {
          nonexistingEvents(id: 1) {
            email
            name
            number
          }
        }
    "#};

    let subscription = runner
        .graphql_subscription::<serde_json::Value>(query)
        .unwrap()
        .subscribe()
        .await
        .unwrap();

    let events = tokio::time::timeout(Duration::from_secs(5), subscription.take(2).collect::<Vec<_>>())
        .await
        .unwrap();

    insta::assert_json_snapshot!(&events, @r#"
    [
      {
        "data": null,
        "errors": [
          {
            "message": "Failed to subscribe to subject 'persistence.user.1.events': jetstream error: stream not found (code 404, error code 10059)",
            "extensions": {
              "code": "INTERNAL_SERVER_ERROR"
            }
          }
        ]
      }
    ]
    "#);
}
