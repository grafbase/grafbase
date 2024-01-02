mod pagination;

use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::{query_namespaced_postgres, query_postgres};

#[test]
fn eq_pk() {
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
              userCollection(first: 10, filter: { id: { eq: 1 } }) {
                edges { node { id name } }  
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
                    "id": 1,
                    "name": "Musti"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn first() {
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
                edges { node { id name } }  
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
                    "id": 1,
                    "name": "Musti"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn last() {
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
                edges { node { id name } }  
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
                    "id": 2,
                    "name": "Naukio"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn order_by() {
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
              userCollection(first: 10, orderBy: [{ name: DESC }]) {
                edges { node { id name } }  
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
                    "id": 2,
                    "name": "Naukio"
                  }
                },
                {
                  "node": {
                    "id": 1,
                    "name": "Musti"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn namespaced() {
    let response = query_namespaced_postgres("pg", |api| async move {
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
              pg {
                userCollection(first: 10, filter: { id: { eq: 1 } }) {
                  edges { node { id name } }  
                }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "pg": {
              "userCollection": {
                "edges": [
                  {
                    "node": {
                      "id": 1,
                      "name": "Musti"
                    }
                  }
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn eq_pk_rename() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id_field INT PRIMARY KEY,
                name_field VARCHAR(255) NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id_field, name_field) VALUES (1, 'Musti'), (2, 'Naukio')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { idField: { eq: 1 } }) {
                edges { node { idField nameField } }  
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
                    "idField": 1,
                    "nameField": "Musti"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn string_eq() {
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
              userCollection(first: 10, filter: { name: { eq: "Musti" } }) {
                edges { node { id name } }  
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
                    "id": 1,
                    "name": "Musti"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bytea_eq() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val BYTEA NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, '\xdeadbeef'::bytea), (2, '\xbeefdead'::bytea)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query Pg {
              userCollection(first: 10, filter: { val: { eq: "3q2+7w" }}) { edges { node { id val }} }
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
                    "id": 1,
                    "val": "3q2+7w"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_eq() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                numbers INT[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, numbers) VALUES (1, '{1, 2}'), (2, '{3, 4}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { numbers: { eq: [3, 4] } }) {
                edges { node { id numbers } }  
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
                    "id": 2,
                    "numbers": [
                      3,
                      4
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_ne() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                numbers INT[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, numbers) VALUES (1, '{1, 2}'), (2, '{3, 4}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { numbers: { ne: [3, 4] } }) {
                edges { node { id numbers } }  
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
                    "id": 1,
                    "numbers": [
                      1,
                      2
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_gt() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                numbers INT[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, numbers) VALUES (1, '{1, 2}'), (2, '{3, 4}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { numbers: { gt: [1, 2] } }) {
                edges { node { id numbers } }  
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
                    "id": 2,
                    "numbers": [
                      3,
                      4
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_contains() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                numbers INT[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, numbers) VALUES (1, '{1, 2}'), (2, '{3, 4}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { numbers: { contains: [1, 2, 2, 1] } }) {
                edges { node { id numbers } }  
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
                    "id": 1,
                    "numbers": [
                      1,
                      2
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_contained() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                numbers INT[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, numbers) VALUES (1, '{1, 2}'), (2, '{3, 4}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { numbers: { contained: [3, 6, 4, 7] } }) {
                edges { node { id numbers } }  
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
                    "id": 2,
                    "numbers": [
                      3,
                      4
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_overlaps() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                numbers INT[] NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, numbers) VALUES (1, '{1, 2}'), (2, '{3, 4}')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { numbers: { overlaps: [1, 5, 5, 6] } }) {
                edges { node { id numbers } }  
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
                    "id": 1,
                    "numbers": [
                      1,
                      2
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn two_field_eq() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                age INT NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name, age) VALUES (1, 'Musti', 11), (2, 'Musti', 12)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { name: { eq: "Musti" }, age: { eq: 11 } }) {
                edges { node { id name age } }  
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
                    "id": 1,
                    "name": "Musti",
                    "age": 11
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn string_ne() {
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
              userCollection(first: 10, filter: { name: { ne: "Musti" } }) {
                edges { node { id name } }  
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
                    "id": 2,
                    "name": "Naukio"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn string_gt() {
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
              userCollection(first: 10, filter: { name: { gt: "Musti" } }) {
                edges { node { id name } }  
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
                    "id": 2,
                    "name": "Naukio"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn string_lt() {
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
              userCollection(first: 10, filter: { name: { lt: "Naukio" } }) {
                edges { node { id name } }  
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
                    "id": 1,
                    "name": "Musti"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn string_gte() {
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
              userCollection(first: 10, filter: { name: { gte: "Musti" } }) {
                edges { node { id name } }  
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
                    "id": 1,
                    "name": "Musti"
                  }
                },
                {
                  "node": {
                    "id": 2,
                    "name": "Naukio"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn string_lte() {
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
              userCollection(first: 10, filter: { name: { lte: "Naukio" } }) {
                edges { node { id name } }  
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
                    "id": 1,
                    "name": "Musti"
                  }
                },
                {
                  "node": {
                    "id": 2,
                    "name": "Naukio"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn string_in() {
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
              userCollection(first: 10, filter: { name: { in: ["Musti", "Naukio"] } }) {
                edges { node { id name } }  
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
                    "id": 1,
                    "name": "Musti"
                  }
                },
                {
                  "node": {
                    "id": 2,
                    "name": "Naukio"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn string_nin() {
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
              userCollection(first: 10, filter: { name: { nin: ["Musti", "Naukio"] } }) {
                edges { node { id name } }  
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": []
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn all() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                age INT NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name, age) VALUES (1, 'Musti', 11), (2, 'Musti', 12)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { ALL: [
                { name: { eq: "Musti" } },
                { age: { eq: 11 } }
              ]}) {
                edges { node { id name age } }  
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
                    "id": 1,
                    "name": "Musti",
                    "age": 11
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn any() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                age INT NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name, age) VALUES (1, 'Musti', 12), (2, 'Naukio', 11)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { ANY: [
                { name: { eq: "Musti" } },
                { age: { eq: 11 } }
              ]}) {
                edges { node { id name age } }  
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
                    "id": 1,
                    "name": "Musti",
                    "age": 12
                  }
                },
                {
                  "node": {
                    "id": 2,
                    "name": "Naukio",
                    "age": 11
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn none() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                age INT NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name, age) VALUES
              (1, 'Musti', 11),
              (2, 'Naukio', 12),
              (3, 'Pentti', 13)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { NONE: [
                { name: { eq: "Musti" } },
                { age: { eq: 13 } }
              ]}) {
                edges { node { id name age } }  
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
                    "id": 2,
                    "name": "Naukio",
                    "age": 12
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn not() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                age INT NOT NULL
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name, age) VALUES
              (1, 'Musti', 11),
              (2, 'Naukio', 12),
              (3, 'Pentti', 13)
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { name: { not: { eq: "Pentti" } } }) {
                edges { node { id name age } }  
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
                    "id": 1,
                    "name": "Musti",
                    "age": 11
                  }
                },
                {
                  "node": {
                    "id": 2,
                    "name": "Naukio",
                    "age": 12
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn one_to_one_relation_filter() {
    let response = query_postgres(|api| async move {
        let user_table = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        "#};

        api.execute_sql(user_table).await;

        let profile_table = indoc! {r#"
            CREATE TABLE "Profile" (
                id INT PRIMARY KEY,
                user_id INT NULL UNIQUE,
                description TEXT NOT NULL,
                CONSTRAINT Profile_User_fkey FOREIGN KEY (user_id) REFERENCES "User" (id)
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
            INSERT INTO "Profile" (id, user_id, description) VALUES
              (1, 1, 'meowmeowmeow'),
              (2, 2, 'purrpurrpurr')
        "#};

        api.execute_sql(insert_profiles).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { profile: { description: { eq: "purrpurrpurr" } } }) {
                edges {
                  node {
                    id
                    name
                    profile { description }
                  }
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
                    "id": 2,
                    "name": "Naukio",
                    "profile": {
                      "description": "purrpurrpurr"
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
fn one_to_many_relation_filter_child_side() {
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
              blogCollection(first: 10, filter: { user: { id: { eq: 1 } } }) {
                edges {
                  node {
                    id
                    title
                    user { id name }
                  }
                }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "blogCollection": {
              "edges": [
                {
                  "node": {
                    "id": 1,
                    "title": "Hello, world!",
                    "user": {
                      "id": 1,
                      "name": "Musti"
                    }
                  }
                },
                {
                  "node": {
                    "id": 2,
                    "title": "Sayonara...",
                    "user": {
                      "id": 1,
                      "name": "Musti"
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
fn one_to_many_relation_filter_parent_side() {
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
              userCollection(first: 10, filter: { blogs: { contains: { id: { eq: 1 } } } }) {
                edges {
                  node {
                    id
                    name
                    blogs(first: 10) { edges { node { id title } } }
                  }
                }
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
                    "id": 1,
                    "name": "Musti",
                    "blogs": {
                      "edges": [
                        {
                          "node": {
                            "id": 1,
                            "title": "Hello, world!"
                          }
                        },
                        {
                          "node": {
                            "id": 2,
                            "title": "Sayonara..."
                          }
                        }
                      ]
                    }
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
