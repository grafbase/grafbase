mod nested;

use expect_test::expect;
use indoc::{formatdoc, indoc};
use integration_tests::postgres::query_postgres;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserCollection {
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
    user_collection: UserCollection,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    data: ResponseData,
}

#[test]
fn id_pk_implicit_order_with_after() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 1) {
                edges { node { name } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.end_cursor;

        assert!(page_info.has_next_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(first: 1, after: "{cursor}") {{
                edges {{ node {{ name }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Naukio"
                  },
                  "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB"
                }
              ],
              "pageInfo": {
                "hasNextPage": false,
                "hasPreviousPage": false,
                "startCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB",
                "endCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn id_pk_implicit_order_with_before() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(last: 1) {
                edges { node { name } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.start_cursor;

        assert!(page_info.has_previous_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(last: 1, before: "{cursor}") {{
                edges {{ node {{ name }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Musti"
                  },
                  "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFycfAwEDEicBFBQIAQckAT0BAQEHKAIkAQ"
                }
              ],
              "pageInfo": {
                "hasNextPage": false,
                "hasPreviousPage": false,
                "startCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFycfAwEDEicBFBQIAQckAT0BAQEHKAIkAQ",
                "endCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFycfAwEDEicBFBQIAQckAT0BAQEHKAIkAQ"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn id_pk_explicit_order_with_after() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 1, orderBy: [{ id: DESC }]) {
                edges { node { name } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.end_cursor;

        assert!(page_info.has_next_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(first: 1, orderBy: [{{ id: DESC }}], after: "{cursor}") {{
                edges {{ node {{ name }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Musti"
                  },
                  "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFycfAwEDEicBFBQIAQckAT0BAQEHKAIkAQ"
                }
              ],
              "pageInfo": {
                "hasNextPage": false,
                "hasPreviousPage": false,
                "startCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFycfAwEDEicBFBQIAQckAT0BAQEHKAIkAQ",
                "endCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFycfAwEDEicBFBQIAQckAT0BAQEHKAIkAQ"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn id_pk_explicit_order_with_before() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(last: 1, orderBy: [{ id: DESC }]) {
                edges { node { name } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.start_cursor;

        assert!(page_info.has_previous_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(last: 1, before: "{cursor}", orderBy: [{{ id: DESC }}]) {{
                edges {{ node {{ name }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Naukio"
                  },
                  "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB"
                }
              ],
              "pageInfo": {
                "hasNextPage": false,
                "hasPreviousPage": false,
                "startCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB",
                "endCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn compound_pk_implicit_order_with_after() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                CONSTRAINT "User_pkey" PRIMARY KEY (name, email)
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (name, email) VALUES
                ('Musti', 'meow1@example.com'),
                ('Musti', 'meow2@example.com'),
                ('Naukio', 'meow3@example.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 1) {
                edges { node { name email } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.end_cursor;

        assert!(page_info.has_next_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(first: 1, after: "{cursor}") {{
                edges {{ node {{ name email }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Musti",
                    "email": "meow2@example.com"
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWLyUDAQMRLyQUFBQFZW1haWwAEW1lb3cyQGV4YW1wbGUuY29tAAlBc2NlbmRpbmcAA0hhVwMBAxEsJhQUFAI5CCQkAXkBAQEJKAIkAQ"
                }
              ],
              "pageInfo": {
                "hasNextPage": true,
                "hasPreviousPage": false,
                "startCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWLyUDAQMRLyQUFBQFZW1haWwAEW1lb3cyQGV4YW1wbGUuY29tAAlBc2NlbmRpbmcAA0hhVwMBAxEsJhQUFAI5CCQkAXkBAQEJKAIkAQ",
                "endCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWLyUDAQMRLyQUFBQFZW1haWwAEW1lb3cyQGV4YW1wbGUuY29tAAlBc2NlbmRpbmcAA0hhVwMBAxEsJhQUFAI5CCQkAXkBAQEJKAIkAQ"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn compound_pk_implicit_order_with_before() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                CONSTRAINT "User_pkey" PRIMARY KEY (name, email)
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (name, email) VALUES
                ('Musti', 'meow1@example.com'),
                ('Naukio', 'meow3@example.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(last: 1) {
                edges { node { name email } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.start_cursor;

        assert!(page_info.has_previous_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(last: 1, before: "{cursor}") {{
                edges {{ node {{ name email }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Musti",
                    "email": "meow1@example.com"
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzAmAwEDEjAlFBQUBWVtYWlsABFtZW93MUBleGFtcGxlLmNvbQAKRGVzY2VuZGluZwADSmNZAwEDEi0nFBQUAjoIJCQBewEBAQkoAiQB"
                }
              ],
              "pageInfo": {
                "hasNextPage": false,
                "hasPreviousPage": false,
                "startCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzAmAwEDEjAlFBQUBWVtYWlsABFtZW93MUBleGFtcGxlLmNvbQAKRGVzY2VuZGluZwADSmNZAwEDEi0nFBQUAjoIJCQBewEBAQkoAiQB",
                "endCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzAmAwEDEjAlFBQUBWVtYWlsABFtZW93MUBleGFtcGxlLmNvbQAKRGVzY2VuZGluZwADSmNZAwEDEi0nFBQUAjoIJCQBewEBAQkoAiQB"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn compound_pk_explicit_order_with_after() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                CONSTRAINT "User_pkey" PRIMARY KEY (name, email)
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (name, email) VALUES
                ('Musti', 'meow1@example.com'),
                ('Musti', 'meow2@example.com'),
                ('Naukio', 'meow3@example.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 1, orderBy: [{ name: ASC }, { email: DESC }]) {
                edges { node { name email } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.end_cursor;

        assert!(page_info.has_next_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(first: 1, orderBy: [{{ name: ASC }}, {{ email: DESC }}], after: "{cursor}") {{
                edges {{ node {{ name email }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Musti",
                    "email": "meow1@example.com"
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWLyUDAQMRLyQUFBQFZW1haWwAEW1lb3cxQGV4YW1wbGUuY29tAApEZXNjZW5kaW5nAANJYlgDAQMSLScUFBQCOggkJAF6AQEBCSgCJAE"
                }
              ],
              "pageInfo": {
                "hasNextPage": true,
                "hasPreviousPage": false,
                "startCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWLyUDAQMRLyQUFBQFZW1haWwAEW1lb3cxQGV4YW1wbGUuY29tAApEZXNjZW5kaW5nAANJYlgDAQMSLScUFBQCOggkJAF6AQEBCSgCJAE",
                "endCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWLyUDAQMRLyQUFBQFZW1haWwAEW1lb3cxQGV4YW1wbGUuY29tAApEZXNjZW5kaW5nAANJYlgDAQMSLScUFBQCOggkJAF6AQEBCSgCJAE"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn compound_pk_explicit_order_with_before() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                CONSTRAINT "User_pkey" PRIMARY KEY (name, email)
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (name, email) VALUES
                ('Musti', 'meow1@example.com'),
                ('Musti', 'meow2@example.com'),
                ('Naukio', 'meow3@example.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(last: 1, orderBy: [{ name: ASC }, { email: DESC }]) {
                edges { node { name email } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.start_cursor;

        assert!(page_info.has_previous_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(last: 1, orderBy: [{{ name: ASC }}, {{ email: DESC }}], before: "{cursor}") {{
                edges {{ node {{ name email }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Musti",
                    "email": "meow1@example.com"
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzAmAwEDEjAlFBQUBWVtYWlsABFtZW93MUBleGFtcGxlLmNvbQAJQXNjZW5kaW5nAANJYlgDAQMRLCYUFBQCOQgkJAF6AQEBCSgCJAE"
                }
              ],
              "pageInfo": {
                "hasNextPage": false,
                "hasPreviousPage": true,
                "startCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzAmAwEDEjAlFBQUBWVtYWlsABFtZW93MUBleGFtcGxlLmNvbQAJQXNjZW5kaW5nAANJYlgDAQMRLCYUFBQCOQgkJAF6AQEBCSgCJAE",
                "endCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzAmAwEDEjAlFBQUBWVtYWlsABFtZW93MUBleGFtcGxlLmNvbQAJQXNjZW5kaW5nAANJYlgDAQMRLCYUFBQCOQgkJAF6AQEBCSgCJAE"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn compound_pk_implicit_order_with_nulls_and_after() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NULL,
                CONSTRAINT "User_key" UNIQUE (name, email)
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (name, email) VALUES
                ('Musti', NULL),
                ('Naukio', NULL),
                ('Naukio', 'meow3@example.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 1) {
                edges { node { name email } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.end_cursor;

        assert!(page_info.has_next_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(first: 1, after: "{cursor}") {{
                edges {{ node {{ name email }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Naukio",
                    "email": null
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACUFzY2VuZGluZwADFjAmAwEDETAlFBQUBWVtYWlsAAlBc2NlbmRpbmcAAzVPRQMBAxEZABQUAAImCCQkAWcBAQEJKAIkAQ"
                }
              ],
              "pageInfo": {
                "hasNextPage": true,
                "hasPreviousPage": false,
                "startCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACUFzY2VuZGluZwADFjAmAwEDETAlFBQUBWVtYWlsAAlBc2NlbmRpbmcAAzVPRQMBAxEZABQUAAImCCQkAWcBAQEJKAIkAQ",
                "endCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACUFzY2VuZGluZwADFjAmAwEDETAlFBQUBWVtYWlsAAlBc2NlbmRpbmcAAzVPRQMBAxEZABQUAAImCCQkAWcBAQEJKAIkAQ"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn compound_pk_implicit_order_with_nulls_and_before() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NULL,
                CONSTRAINT "User_key" UNIQUE (name, email)
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (name, email) VALUES
                ('Musti', NULL),
                ('Naukio', NULL),
                ('Naukio', 'meow3@example.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(last: 1) {
                edges { node { name email } cursor }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        let response: Response = api.execute_as(query).await;
        let page_info = response.data.user_collection.page_info;
        let cursor = page_info.start_cursor;

        assert!(page_info.has_previous_page);

        let query = formatdoc! {r#"
            query {{
              userCollection(last: 1, before: "{cursor}") {{
                edges {{ node {{ name email }} cursor }}
                pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }}
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
                    "name": "Naukio",
                    "email": null
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACkRlc2NlbmRpbmcAAxcxJwMBAxIxJhQUFAVlbWFpbAAKRGVzY2VuZGluZwADN1FHAwEDEhoAFBQAAicIJCQBaQEBAQkoAiQB"
                }
              ],
              "pageInfo": {
                "hasNextPage": false,
                "hasPreviousPage": true,
                "startCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACkRlc2NlbmRpbmcAAxcxJwMBAxIxJhQUFAVlbWFpbAAKRGVzY2VuZGluZwADN1FHAwEDEhoAFBQAAicIJCQBaQEBAQkoAiQB",
                "endCursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACkRlc2NlbmRpbmcAAxcxJwMBAxIxJhQUFAVlbWFpbAAKRGVzY2VuZGluZwADN1FHAwEDEhoAFBQAAicIJCQBaQEBAQkoAiQB"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn nested_page_info_no_limit() {
    let response = query_postgres(|api| async move {
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
              (3, 2, 'Meow meow?')
        "#};

        api.execute_sql(insert_profiles).await;

        let query = indoc! {r"
            query {
              userCollection(first: 1000, filter: { id: { eq: 1 } }) {
                edges {
                  node {
                    blogs(first: 1000) {
                      edges { node { id title } cursor }
                      pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
                    }
                  }
                  cursor
                }
                pageInfo { hasNextPage hasPreviousPage startCursor endCursor }
              }
            }
        "};

        api.execute(query).await
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
                            "id": 1,
                            "title": "Hello, world!"
                          },
                          "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgEUFAgBByQBPAEBAQcoAiQB"
                        },
                        {
                          "node": {
                            "id": 2,
                            "title": "Sayonara..."
                          },
                          "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB"
                        }
                      ],
                      "pageInfo": {
                        "hasNextPage": false,
                        "hasPreviousPage": false,
                        "startCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgEUFAgBByQBPAEBAQcoAiQB",
                        "endCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB"
                      }
                    }
                  },
                  "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgEUFAgBByQBPAEBAQcoAiQB"
                }
              ],
              "pageInfo": {
                "hasNextPage": false,
                "hasPreviousPage": false,
                "startCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgEUFAgBByQBPAEBAQcoAiQB",
                "endCursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgEUFAgBByQBPAEBAQcoAiQB"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
