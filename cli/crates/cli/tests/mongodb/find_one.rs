use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;
use wiremock::ResponseTemplate;

#[tokio::test(flavor = "multi_thread")]
async fn query() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String! @map(name: "real_name")
        }}
    "#};

    let filter = json!({
        "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
    });

    let projection = json!({
        "_id": 1,
        "real_name": 1,
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "document": {
            "_id": "5ca4bbc7a2dd94ee5816238d",
            "real_name": "Bob"
        }
    }));

    let server = Server::find_one(&config, "users", filter, projection, response).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          user(by: { id: "5ca4bbc7a2dd94ee5816238d" }) {
            id
            name
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "user": {
          "id": "5ca4bbc7a2dd94ee5816238d",
          "name": "Bob"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn query_with_fragment() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String! @map(name: "real_name")
        }}
    "#};

    let filter = json!({
        "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
    });

    let projection = json!({
        "_id": 1,
        "real_name": 1,
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "document": {
            "_id": "5ca4bbc7a2dd94ee5816238d",
            "real_name": "Bob"
        }
    }));

    let server = Server::find_one(config, "users", filter, projection, response).await;

    let request = server.request(indoc::indoc! {r#"
        fragment Person on User {
            id
            name
        }

        query {
          user(by: { id: "5ca4bbc7a2dd94ee5816238d" }) {
            ...Person
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "user": {
          "id": "5ca4bbc7a2dd94ee5816238d",
          "name": "Bob"
        }
      }
    }
    "###);
}
