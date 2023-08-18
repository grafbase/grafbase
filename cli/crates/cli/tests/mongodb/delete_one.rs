use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn mutation() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String! @map(name: "real_name")
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        }
    });

    let server = Server::delete_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userDelete(by: { id: "5ca4bbc7a2dd94ee5816238d" }) {
            deletedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userDelete": {
          "deletedCount": 1
        }
      }
    }
    "###);
}
