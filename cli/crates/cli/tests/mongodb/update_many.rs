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
            "real_name": { "$eq": "Herp" }
        },
        "update": {
            "$set": { "real_name": "Derp" }
        }
    });

    let server = Server::update_many(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdateMany(
            filter: { name: { eq: "Herp" } },
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
        "userUpdateMany": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn unset_false() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String @map(name: "real_name")
        }}
    "#};

    let expected_body = json!({});

    let mut server = Server::update_many(&config, "users", expected_body).await;
    server.expected_requests(0);

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdateMany(
            filter: { name: { eq: "Herp" } },
            input: { name: { unset: false } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdateMany": {
          "matchedCount": 0,
          "modifiedCount": 0
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn current_datetime_false_triggering_empty_update() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
          registered: DateTime
        }}
    "#};

    let expected_body = json!({});

    let mut server = Server::update_many(&config, "users", expected_body).await;
    server.expected_requests(0);

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdateMany(
            filter: { name: { eq: "Herp" } },
            input: { registered: { currentDate: false } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdateMany": {
          "matchedCount": 0,
          "modifiedCount": 0
        }
      }
    }
    "###);
}
