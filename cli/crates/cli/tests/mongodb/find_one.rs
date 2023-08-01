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

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" }
        },
        "projection": {
            "_id": 1,
            "real_name": 1,
        }
    });

    let mut server = Server::find_one(&config, "users", expected_body).await;
    server.set_response(ResponseTemplate::new(200).set_body_json(json!({
        "document": {
            "_id": "5ca4bbc7a2dd94ee5816238d",
            "real_name": "Bob"
        }
    })));

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
async fn nested_query() {
    let config = indoc::formatdoc! {r#"
        type Address {{
          street: String! @map(name: "street_name")
          city: String!
        }}

        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          address: Address! @map(name: "real_address")
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "projection": {
            "_id": 1,
            "real_address.street_name": 1,
            "real_address.city": 1,
        }
    });

    let mut server = Server::find_one(&config, "users", expected_body).await;
    server.set_response(ResponseTemplate::new(200).set_body_json(json!({
        "document": {
            "real_address": {
              "street_name": "Wall",
              "city": "Street"
            }
        }
    })));

    let request = server.request(indoc::indoc! {r#"
        query {
          user(by: { id: "5ca4bbc7a2dd94ee5816238d" }) {
            address { street city }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "user": {
          "address": {
            "street": "Wall",
            "city": "Street"
          }
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

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "projection": {
            "_id": 1,
            "real_name": 1,
        }
    });

    let mut server = Server::find_one(&config, "users", expected_body).await;
    server.set_response(ResponseTemplate::new(200).set_body_json(json!({
        "document": {
            "_id": "5ca4bbc7a2dd94ee5816238d",
            "real_name": "Bob"
        }
    })));

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
