use std::time::Duration;

use async_nats::ConnectOptions;
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
          @link(url: "{path_str}", import: ["@natsPublish", "@natsSubscription", "NatsPublishResult"])

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
          }
        }
    "#};

    let mut subscription = runner
        .graphql_subscription::<serde_json::Value>(query)
        .unwrap()
        .subscribe()
        .await
        .unwrap();

    tokio::spawn(async move {
        loop {
            let event = json!({ "email": "user1@example.com", "name": "User One" });
            let event = serde_json::to_vec(&event).unwrap();

            nats.publish("subscription.user.1.events", event.into()).await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let event = tokio::time::timeout(Duration::from_secs(5), subscription.next())
        .await
        .unwrap()
        .unwrap();

    insta::assert_json_snapshot!(&event, @r#"
    {
      "data": {
        "userEvents": {
          "email": "user1@example.com",
          "name": "User One"
        }
      }
    }
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
