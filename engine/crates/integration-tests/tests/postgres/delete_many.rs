use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::{query_namespaced_postgres, query_postgres};

#[test]
fn namespaced() {
    let response = query_namespaced_postgres("postgres", |api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              postgres {
                userDeleteMany(filter: { name: { eq: "Musti" } }) { returning { id name } }
              }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              postgres {
                userCollection(first: 10) { edges { node { id name } } }
              }
            }
        "#};

        let expected = expect![[r#"
            {
              "data": {
                "postgres": {
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
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "postgres": {
              "userDeleteMany": {
                "returning": [
                  {
                    "id": 1,
                    "name": "Musti"
                  },
                  {
                    "id": 3,
                    "name": "Musti"
                  }
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn eq() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { name: { eq: "Musti" } }) { returning { id name } rowCount }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "name": "Musti"
                },
                {
                  "id": 3,
                  "name": "Musti"
                }
              ],
              "rowCount": 2
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn eq_not_returning() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { name: { eq: "Musti" } }) { rowCount }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "rowCount": 2
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn missing() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { name: { eq: "Pertti" } }) { returning { id name } rowCount }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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
                    },
                    {
                      "node": {
                        "id": 3,
                        "name": "Musti"
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [],
              "rowCount": 0
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn eq_null() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, null), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { name: { eq: null } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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
                        "id": 3,
                        "name": "Musti"
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "name": null
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn ne_null() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, null), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { name: { ne: null } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

        let expected = expect![[r#"
            {
              "data": {
                "userCollection": {
                  "edges": [
                    {
                      "node": {
                        "id": 2,
                        "name": null
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "name": "Musti"
                },
                {
                  "id": 3,
                  "name": "Musti"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn eq_two_fields() {
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
            INSERT INTO "User" (id, name, age) VALUES (1, 'Musti', 11), (2, 'Naukio', 11), (3, 'Musti', 12)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { name: { eq: "Musti" }, age: { eq: 12 } }) { returning { id name age } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name age } } }
            }
        "#};

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
                        "age": 11
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 3,
                  "name": "Musti",
                  "age": 12
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn eq_rename() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name_game VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name_game) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { nameGame: { eq: "Musti" } }) { returning { id nameGame } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id nameGame } } }
            }
        "#};

        let expected = expect![[r#"
            {
              "data": {
                "userCollection": {
                  "edges": [
                    {
                      "node": {
                        "id": 2,
                        "nameGame": "Naukio"
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "nameGame": "Musti"
                },
                {
                  "id": 3,
                  "nameGame": "Musti"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn ne() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { name: { ne: "Musti" } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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
                        "id": 3,
                        "name": "Musti"
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "name": "Naukio"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn gt() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { id: { gt: 1 } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "name": "Naukio"
                },
                {
                  "id": 3,
                  "name": "Musti"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn lt() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { id: { lt: 3 } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

        let expected = expect![[r#"
            {
              "data": {
                "userCollection": {
                  "edges": [
                    {
                      "node": {
                        "id": 3,
                        "name": "Musti"
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "name": "Musti"
                },
                {
                  "id": 2,
                  "name": "Naukio"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn gte() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { id: { gte: 2 } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "name": "Naukio"
                },
                {
                  "id": 3,
                  "name": "Musti"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn lte() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { id: { lte: 2 } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

        let expected = expect![[r#"
            {
              "data": {
                "userCollection": {
                  "edges": [
                    {
                      "node": {
                        "id": 3,
                        "name": "Musti"
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "name": "Musti"
                },
                {
                  "id": 2,
                  "name": "Naukio"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn r#in() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { id: { in: [1, 3] } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "name": "Musti"
                },
                {
                  "id": 3,
                  "name": "Musti"
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn nin() {
    // 9 inch

    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { id: { nin: [1, 3] } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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
                        "id": 3,
                        "name": "Musti"
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "name": "Naukio"
                }
              ]
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
            INSERT INTO "User" (id, name, age) VALUES (1, 'Musti', 11), (2, 'Naukio', 11), (3, 'Musti', 12)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { ALL: [
                { name: { eq: "Musti" } },
                { age: { eq: 11 } }
              ]}) {
                returning { id name age }
              }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name age } } }
            }
        "#};

        let expected = expect![[r#"
            {
              "data": {
                "userCollection": {
                  "edges": [
                    {
                      "node": {
                        "id": 2,
                        "name": "Naukio",
                        "age": 11
                      }
                    },
                    {
                      "node": {
                        "id": 3,
                        "name": "Musti",
                        "age": 12
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "name": "Musti",
                  "age": 11
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
            INSERT INTO "User" (id, name, age) VALUES (1, 'Musti', 11), (2, 'Naukio', 11), (3, 'Musti', 12)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { ANY: [
                { name: { eq: "Naukio" } },
                { age: { eq: 12 } }
              ]}) {
                returning { id name age }
              }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name age } } }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "name": "Naukio",
                  "age": 11
                },
                {
                  "id": 3,
                  "name": "Musti",
                  "age": 12
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
            INSERT INTO "User" (id, name, age) VALUES (1, 'Musti', 11), (2, 'Naukio', 12), (3, 'Pentti', 13)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { NONE: [
                { name: { eq: "Musti" } },
                { age: { eq: 13 } }
              ]}) {
                returning { id name age }
              }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name age } } }
            }
        "#};

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
                        "id": 3,
                        "name": "Pentti",
                        "age": 13
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "name": "Naukio",
                  "age": 12
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
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio'), (3, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { name: { not: { eq: "Musti" } } }) { returning { id name } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { id name } } }
            }
        "#};

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
                        "id": 3,
                        "name": "Musti"
                      }
                    }
                  ]
                }
              }
            }"#]];

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "name": "Naukio"
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

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { numbers: { eq: [3, 4] } }) { returning { id numbers } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) {
                edges { node { id numbers } }  
              }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "numbers": [
                    3,
                    4
                  ]
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

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { numbers: { ne: [3, 4] } }) { returning { id numbers } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) {
                edges { node { id numbers } }  
              }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "numbers": [
                    1,
                    2
                  ]
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

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { numbers: { gt: [1, 2] } }) { returning { id numbers } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) {
                edges { node { id numbers } }  
              }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "numbers": [
                    3,
                    4
                  ]
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

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { numbers: { contains: [1, 2, 2, 1] } }) { returning { id numbers } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) {
                edges { node { id numbers } }  
              }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "numbers": [
                    1,
                    2
                  ]
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

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { numbers: { contained: [3, 6, 4, 7] } }) { returning { id numbers } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) {
                edges { node { id numbers } }  
              }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 2,
                  "numbers": [
                    3,
                    4
                  ]
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

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { numbers: { overlaps: [1, 5, 5, 6] } }) { returning { id numbers } }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) {
                edges { node { id numbers } }  
              }
            }
        "#};

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

        let query_result = serde_json::to_string_pretty(&api.execute(query).await.to_graphql_response()).unwrap();
        expected.assert_eq(&query_result);

        mutation_result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "returning": [
                {
                  "id": 1,
                  "numbers": [
                    1,
                    2
                  ]
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
