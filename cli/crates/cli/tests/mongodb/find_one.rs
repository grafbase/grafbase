use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

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

    let server = Server::find_one(&config, "users", expected_body).await;

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
        "user": null
      }
    }
    "###);
}
