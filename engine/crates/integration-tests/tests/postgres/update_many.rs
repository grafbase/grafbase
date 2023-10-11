use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::query_postgres;

#[test]
fn smoke() {
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
                (2, 'Naukio', 11),
                (3, 'Pertti', 12)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(filter: { age: { eq: 11 } }, input: { age: { set: 10 } }) {
                id
                name
                age
              }
            }
        "#};

        let result = serde_json::to_string_pretty(&api.execute(mutation).await.to_graphql_response()).unwrap();

        let expected = expect![[r#"
            {
              "data": {
                "userUpdateMany": [
                  {
                    "id": 1,
                    "name": "Musti",
                    "age": 10
                  },
                  {
                    "id": 2,
                    "name": "Naukio",
                    "age": 10
                  }
                ]
              }
            }"#]];

        expected.assert_eq(&result);

        let query = indoc! {r#"
            query {
              userCollection(first: 10) {
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
                    "age": 10
                  }
                },
                {
                  "node": {
                    "id": 2,
                    "name": "Naukio",
                    "age": 10
                  }
                },
                {
                  "node": {
                    "id": 3,
                    "name": "Pertti",
                    "age": 12
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
