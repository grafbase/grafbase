use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn id_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$eq": { "$oid": "5ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } }, first: 100) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}
