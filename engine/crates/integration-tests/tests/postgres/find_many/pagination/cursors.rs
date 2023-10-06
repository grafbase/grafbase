use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::query_postgres;

#[test]
fn root_level_implicit_order_with_single_pk() {
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

        let query = indoc! {r#"
            query {
              userCollection(first: 2) {
                edges { node { name } cursor }
              }
            }
        "#};

        api.execute(query).await
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
                  "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgEUFAgBByQBPAEBAQcoAiQB"
                },
                {
                  "node": {
                    "name": "Naukio"
                  },
                  "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgIUFAgBByQBPAEBAQcoAiQB"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn root_level_implicit_order_with_compound_pk() {
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
            INSERT INTO "User" (name, email) VALUES ('Musti', 'murr@purr.com'), ('Naukio', 'purr@murr.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 2) {
                edges { node { name } cursor }
              }
            }
        "#};

        api.execute(query).await
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
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWLyUDAQMRLyQUFBQFZW1haWwADW11cnJAcHVyci5jb20ACUFzY2VuZGluZwADRF1TAwEDESgiFBQUAjUIJCQBdQEBAQkoAiQB"
                },
                {
                  "node": {
                    "name": "Naukio"
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACUFzY2VuZGluZwADFjAmAwEDETAlFBQUBWVtYWlsAA1wdXJyQG11cnIuY29tAAlBc2NlbmRpbmcAA0ReVAMBAxEoIhQUFAI1CCQkAXYBAQEJKAIkAQ"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn root_level_implicit_order_with_nullable_compound_key() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NULL,
                CONSTRAINT "User_pkey" UNIQUE (name, email)
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (name, email) VALUES ('Musti', NULL), ('Naukio', 'purr@murr.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 2) {
                edges { node { name } cursor }
              }
            }
        "#};

        api.execute(query).await
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
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWLyUDAQMRLyQUFBQFZW1haWwACUFzY2VuZGluZwADNU5EAwEDERkAFBQAAiYIJCQBZgEBAQkoAiQB"
                },
                {
                  "node": {
                    "name": "Naukio"
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACUFzY2VuZGluZwADFjAmAwEDETAlFBQUBWVtYWlsAA1wdXJyQG11cnIuY29tAAlBc2NlbmRpbmcAA0ReVAMBAxEoIhQUFAI1CCQkAXYBAQEJKAIkAQ"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn root_level_explicit_order() {
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

        let query = indoc! {r#"
            query {
              userCollection(first: 2, orderBy: [{ name: DESC }]) {
                edges { node { id } cursor }
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": [
                {
                  "node": {
                    "id": 2
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACkRlc2NlbmRpbmcAAxcxJwMBAxIxJhQUFAJpZAAJQXNjZW5kaW5nAAMzTUMDAQMRFgIUFAgCIwgkJAFlAQEBCSgCJAE"
                },
                {
                  "node": {
                    "id": 1
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzAmAwEDEjAlFBQUAmlkAAlBc2NlbmRpbmcAAzNMQgMBAxEWARQUCAIjCCQkAWQBAQEJKAIkAQ"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn root_level_explicit_order_two_columns() {
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

        let query = indoc! {r#"
            query {
              userCollection(first: 2, orderBy: [{ name: DESC }, { id: DESC }]) {
                edges { node { id } cursor }
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": [
                {
                  "node": {
                    "id": 2
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABk5hdWtpbwBkaXJlY3Rpb24ACkRlc2NlbmRpbmcAAxcxJwMBAxIxJhQUFAJpZAAKRGVzY2VuZGluZwADNE5EAwEDEhcCFBQIAiQIJCQBZgEBAQkoAiQB"
                },
                {
                  "node": {
                    "id": 1
                  },
                  "cursor": "ZmllbGRzAG5hbWUABG5hbWUAdmFsdWUABU11c3RpAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzAmAwEDEjAlFBQUAmlkAApEZXNjZW5kaW5nAAM0TUMDAQMSFwEUFAgCJAgkJAFlAQEBCSgCJAE"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn nested_ordering_cursors_implicit_order() {
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

        let query = indoc! {r#"
            query {
              userCollection(first: 1000, filter: { id: { eq: 1 } }) {
                edges {
                  node {
                    blogs(first: 2) { edges { node { id title } cursor } }
                  }
                  cursor
                }
              }
            }
        "#};

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
                      ]
                    }
                  },
                  "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgEUFAgBByQBPAEBAQcoAiQB"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn nested_ordering_cursors_explicit_order() {
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

        let query = indoc! {r#"
            query {
              userCollection(first: 1000, filter: { id: { eq: 1 } }) {
                edges {
                  node {
                    blogs(first: 2, orderBy: [{ title: DESC }]) { edges { node { id title } cursor } }
                  }
                  cursor
                }
              }
            }
        "#};

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
                            "id": 2,
                            "title": "Sayonara..."
                          },
                          "cursor": "ZmllbGRzAG5hbWUABXRpdGxlAHZhbHVlAAtTYXlvbmFyYS4uLgBkaXJlY3Rpb24ACkRlc2NlbmRpbmcAAxc3LAMBAxI3KxQUFAJpZAAJQXNjZW5kaW5nAAMzU0gDAQMRFgIUFAgCIwgkJAFrAQEBCSgCJAE"
                        },
                        {
                          "node": {
                            "id": 1,
                            "title": "Hello, world!"
                          },
                          "cursor": "ZmllbGRzAG5hbWUABXRpdGxlAHZhbHVlAA1IZWxsbywgd29ybGQhAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzkuAwEDEjktFBQUAmlkAAlBc2NlbmRpbmcAAzNVSgMBAxEWARQUCAIjCCQkAW0BAQEJKAIkAQ"
                        }
                      ]
                    }
                  },
                  "cursor": "ZmllbGRzAG5hbWUAAmlkAHZhbHVlAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWJh4DAQMRJgEUFAgBByQBPAEBAQcoAiQB"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
