use expect_test::expect;
use indoc::{formatdoc, indoc};
use integration_tests::postgresql::query_postgresql;
use serde_json::Value;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserNode {
    blogs: Collection<Edge<Value>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Edge<T> {
    node: T,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Collection<T> {
    edges: Vec<T>,
    page_info: PageInfo,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    has_previous_page: bool,
    start_cursor: String,
    end_cursor: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseData {
    user_collection: Collection<Edge<UserNode>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    data: ResponseData,
}

#[test]
fn single_pk_implicit_order_after() {
    let response = query_postgresql(|api| async move {
        let user_table = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        "#};

        api.execute_sql(user_table).await;

        let profile_table = indoc! {r#"
            CREATE TABLE "Blog" (
                id INT PRIMARY KEY,
                user_id INT NOT NULL,
                title VARCHAR(255) NOT NULL,
                CONSTRAINT Blog_User_fkey FOREIGN KEY (user_id) REFERENCES "User" (id)
            )  
        "#};

        api.execute_sql(profile_table).await;

        let insert_users = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES
              (1, 'Musti'),
              (2, 'Naukio')
        "#};

        api.execute_sql(insert_users).await;

        let insert_profiles = indoc! {r#"
            INSERT INTO "Blog" (id, user_id, title) VALUES
              (1, 1, 'Hello, world!'),
              (2, 1, 'Sayonara...'),
              (3, 1, 'Meow meow?')
        "#};

        api.execute_sql(insert_profiles).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { id: { eq: 1 } }) {
                edges {
                  node {
                    blogs(first: 1) {
                      edges { node { id title } cursor }
                      pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
                    }
                  }
                }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "#};

        let response: Response = api.execute_as(query).await;
        let page_info = &response.data.user_collection.edges[0].node.blogs.page_info;
        let cursor = &page_info.end_cursor;

        assert!(page_info.has_next_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(first: 10, filter: {{ id: {{ eq: 1 }} }}) {{
                edges {{
                  node {{
                    blogs(first: 1, after: "{cursor}") {{
                      edges {{ node {{ id title }} cursor }}
                      pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
                    }}
                  }}
                }}
              }}
            }}
        "#};

        api.execute(&query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": [
                {
                  "node": {
                    "blogs": {
                      "edges": [
                        {
                          "node": {
                            "id": 2,
                            "title": "Sayonara..."
                          },
                          "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB"
                        }
                      ],
                      "pageInfo": {
                        "hasNextPage": true,
                        "hasPreviousPage": false,
                        "startCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB",
                        "endCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB"
                      }
                    }
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn single_pk_implicit_order_before() {
    let response = query_postgresql(|api| async move {
        let user_table = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        "#};

        api.execute_sql(user_table).await;

        let profile_table = indoc! {r#"
            CREATE TABLE "Blog" (
                id INT PRIMARY KEY,
                user_id INT NOT NULL,
                title VARCHAR(255) NOT NULL,
                CONSTRAINT Blog_User_fkey FOREIGN KEY (user_id) REFERENCES "User" (id)
            )  
        "#};

        api.execute_sql(profile_table).await;

        let insert_users = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES
              (1, 'Musti'),
              (2, 'Naukio')
        "#};

        api.execute_sql(insert_users).await;

        let insert_profiles = indoc! {r#"
            INSERT INTO "Blog" (id, user_id, title) VALUES
              (1, 1, 'Hello, world!'),
              (2, 1, 'Sayonara...'),
              (3, 1, 'Meow meow?')
        "#};

        api.execute_sql(insert_profiles).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { id: { eq: 1 } }) {
                edges {
                  node {
                    blogs(last: 1) {
                      edges { node { id title } cursor }
                      pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
                    }
                  }
                  cursor
                }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "#};

        let response: Response = api.execute_as(query).await;
        let page_info = &response.data.user_collection.edges[0].node.blogs.page_info;
        let cursor = &page_info.start_cursor;

        assert!(page_info.has_previous_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(first: 10, filter: {{ id: {{ eq: 1 }} }}) {{
                edges {{
                  node {{
                    blogs(last: 1, before: "{cursor}") {{
                      edges {{ node {{ id title }} cursor }}
                      pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
                    }}
                  }}
                }}
              }}
            }}
        "#};

        api.execute(&query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": [
                {
                  "node": {
                    "blogs": {
                      "edges": [
                        {
                          "node": {
                            "id": 2,
                            "title": "Sayonara..."
                          },
                          "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFycfAwEDEicCFBQIAQckAT0BAQEHKAIkAQ"
                        }
                      ],
                      "pageInfo": {
                        "hasNextPage": false,
                        "hasPreviousPage": true,
                        "startCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFycfAwEDEicCFBQIAQckAT0BAQEHKAIkAQ",
                        "endCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFycfAwEDEicCFBQIAQckAT0BAQEHKAIkAQ"
                      }
                    }
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
