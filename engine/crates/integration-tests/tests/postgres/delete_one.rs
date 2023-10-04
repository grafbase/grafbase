use expect_test::expect;
use indoc::indoc;
use integration_tests::postgresql::query_postgres;

#[test]
fn single_pk() {
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

        let mutation = indoc! {r#"
            mutation {
              userDelete(by: { id: 1 }) { id name }
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
            "userDelete": {
              "id": 1,
              "name": "Musti"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn single_unique() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL UNIQUE
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDelete(by: { name: "Musti" }) { id name }
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
            "userDelete": {
              "id": 1,
              "name": "Musti"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn composite_pk() {
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
            INSERT INTO "User" (name, email) VALUES ('Musti', 'meow@example.com'), ('Musti', 'purr@example.com')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDelete(by: { nameEmail: { name: "Musti", email: "purr@example.com" } }) { name email }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { name email } } }
            }
        "#};

        let expected = expect![[r#"
            {
              "data": {
                "userCollection": {
                  "edges": [
                    {
                      "node": {
                        "name": "Musti",
                        "email": "meow@example.com"
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
            "userDelete": {
              "name": "Musti",
              "email": "purr@example.com"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn composite_key_with_nulls() {
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
            INSERT INTO "User" (name, email) VALUES ('Musti', 'meow@example.com'), ('Musti', NULL)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userDelete(by: { nameEmail: { name: "Musti" } }) { name email }
            }
        "#};

        let mutation_result = api.execute(mutation).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10) { edges { node { name email } } }
            }
        "#};

        let expected = expect![[r#"
            {
              "data": {
                "userCollection": {
                  "edges": [
                    {
                      "node": {
                        "name": "Musti",
                        "email": "meow@example.com"
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
            "userDelete": {
              "name": "Musti",
              "email": null
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
