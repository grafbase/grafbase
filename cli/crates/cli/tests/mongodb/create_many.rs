use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn with_id_and_mapped_string() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String! @map(name: "real_name")
        }}
    "#};

    let expected_request = json!({
        "documents": [
            {
              "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
              "real_name": "Jack",
            },
            {
              "_id": { "$oid": "5ca4bbc7a2dd94ee5816238e" },
              "real_name": "Bob",
            },
        ]
    });

    let server = Server::create_many(&config, "users", expected_request).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreateMany(input: [
            {
              id: "5ca4bbc7a2dd94ee5816238d",
              name: "Jack"
            },
            {
              id: "5ca4bbc7a2dd94ee5816238e",
              name: "Bob"
            }
          ]) {
            insertedIds
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreateMany": {
          "insertedIds": [
            "5ca4bbc7a2dd94ee5816238d",
            "5ca4bbc7a2dd94ee5816238e"
          ]
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_id_and_mapped_string_namespaced() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String! @map(name: "real_name")
        }}
    "#};

    let expected_request = json!({
        "documents": [
            {
              "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
              "real_name": "Jack",
            },
            {
              "_id": { "$oid": "5ca4bbc7a2dd94ee5816238e" },
              "real_name": "Bob",
            },
        ]
    });

    let server = Server::create_many_namespaced("bongo", &config, "users", expected_request).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          bongo {
            userCreateMany(input: [
              {
                id: "5ca4bbc7a2dd94ee5816238d",
                name: "Jack"
              },
              {
                id: "5ca4bbc7a2dd94ee5816238e",
                name: "Bob"
              }
            ]) {
              insertedIds
            }
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "bongo": {
          "userCreateMany": {
            "insertedIds": [
              "5ca4bbc7a2dd94ee5816238d",
              "5ca4bbc7a2dd94ee5816238e"
            ]
          }
        }
      }
    }
    "###);
}
