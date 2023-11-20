use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::query_postgres;

#[test]
fn string_set_with_returning() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdate(by: { id: 1 }, input: { name: { set: "Naukio" } }) {
                returning {
                  id
                  name
                }
                rowCount
              }
            }
        "#};

        let result = serde_json::to_string_pretty(&api.execute(mutation).await.to_graphql_response()).unwrap();

        let expected = expect![[r#"
            {
              "data": {
                "userUpdate": {
                  "returning": {
                    "id": 1,
                    "name": "Naukio"
                  },
                  "rowCount": 1
                }
              }
            }"#]];

        expected.assert_eq(&result);

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                name
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "name": "Naukio"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn string_set_no_returning() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES (1, 'Musti')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdate(by: { id: 1 }, input: { name: { set: "Naukio" } }) {
                rowCount
              }
            }
        "#};

        let result = serde_json::to_string_pretty(&api.execute(mutation).await.to_graphql_response()).unwrap();

        let expected = expect![[r#"
            {
              "data": {
                "userUpdate": {
                  "rowCount": 1
                }
              }
            }"#]];

        expected.assert_eq(&result);

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                name
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "name": "Naukio"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int2_increment() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val INT2 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 1)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { increment: 68 } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": 69
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int2_decrement() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val INT2 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 70)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { decrement: 1 } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": 69
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int2_multiply() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val INT2 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 6)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { multiply: 8 } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": 48
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int2_divide() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val INT2 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 138)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { divide: 2 } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": 69
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int4_increment() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val INT4 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 1)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { increment: 68 } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": 69
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn int8_increment() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val INT8 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 1)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { increment: "68" } }) {
                returning { id }
              }
            }
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": "69"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float_increment() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val FLOAT4 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 1.0)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { increment: 68.0 } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": 69.0
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn double_increment() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val FLOAT8 NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 1.0)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { increment: 68.0 } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": 69.0
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn numeric_increment() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val NUMERIC NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 1.0)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { increment: "68.0" } }) {
                returning { id }
              }
            }
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": "69.0"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn money_increment() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val MONEY NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, 1.0)
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { increment: "68.0" } }) {
                returning { id }
              }
            }
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": "$69.00"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_set() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val INT2[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, '{1, 2}')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { set: [3, 4] } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }    
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": [
                3,
                4
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_append() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val INT[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, '{1}')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { append: [2, 3] } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": [
                1,
                2,
                3
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_prepend() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val INT[] NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, '{1}')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { prepend: [2, 3] } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": [
                2,
                3,
                1
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn jsonb_append() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val JSONB NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, '[1]')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { append: [2, 3] } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": [
                1,
                2,
                3
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn jsonb_prepend() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val JSONB NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, '[1]')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { prepend: [2, 3] } }) {
                returning { id }
              }
            }
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": [
                2,
                3,
                1
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn jsonb_delete_key_from_object() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val JSONB NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, '{ "foo": 1, "bar": 2 }')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { deleteKey: "foo" } }) {
                returning { id }
              }
            }
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": {
                "bar": 2
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn jsonb_delete_key_from_array() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val JSONB NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, '["foo", "bar"]')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { deleteKey: "foo" } }) {
                returning { id }
              }
            }
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": [
                "bar"
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn jsonb_delete_at_path() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                val JSONB NOT NULL
            )
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, val) VALUES (1, '["a", { "b": 1 }]')
        "#};

        api.execute_sql(insert).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdate(by: { id: 1 }, input: { val: { deleteAtPath: ["1", "b"] } }) {
                returning { id }
              }
            }
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                val
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "val": [
                "a",
                {}
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
