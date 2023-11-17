use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn simple_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Int
        }}
    "#};

    let body = json!({
        "filter": {
            "age": { "$gt": 30 },
        },
    });

    let server = Server::delete_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r"
        mutation {
          userDeleteMany(filter: { age: { gt: 30 } })
          {
            deletedCount
          }
        }   
    "});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userDeleteMany": {
          "deletedCount": 2
        }
      }
    }
    "###);
}
