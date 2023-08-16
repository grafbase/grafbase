use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn single_set() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String! @map(name: "real_name")
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$set": { "real_name": "Derp" }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { name: { set: "Derp" } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn current_timestamp() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          registered: Timestamp
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$currentDate": { "registered": { "$type": "timestamp" } }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { registered: { currentDate: true } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn current_datetime() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          registered: DateTime
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$currentDate": { "registered": { "$type": "date" } }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { registered: { currentDate: true } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}
