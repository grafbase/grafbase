use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn with_id() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let expected_request = json!({
        "document": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        }
    });

    let server = Server::create_one(&config, "users", expected_request).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            id: "5ca4bbc7a2dd94ee5816238d",
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}
