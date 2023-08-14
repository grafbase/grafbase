use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn simple_eq() {
    // Just a smoke test, we must test this properly with a real database.

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

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userDeleteMany(filter: { age: { gt: 30 } })
          {
            deletedCount
          }
        }   
    "#});

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

#[tokio::test(flavor = "multi_thread")]
async fn eq_with_rename() {
    // Just a smoke test, we must test this properly with a real database.

    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Int @map(name: "fake")
        }}
    "#};

    let body = json!({
        "filter": {
            "fake": { "$gt": 30 },
        },
    });

    let server = Server::delete_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userDeleteMany(filter: { age: { gt: 30 } })
          {
            deletedCount
          }
        }   
    "#});

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

#[tokio::test(flavor = "multi_thread")]
async fn nested_eq() {
    // Just a smoke test, we must test this properly with a real database.

    let config = indoc::formatdoc! {r#"
        type Inner {{
          age: Int @map(name: "fake")
        }}

        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          inner: Inner
        }}
    "#};

    let body = json!({
        "filter": {
            "inner.fake": { "$gt": 30 },
        },
    });

    let server = Server::delete_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userDeleteMany(filter: { inner: { age: { gt: 30 } } })
          {
            deletedCount
          }
        }   
    "#});

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
