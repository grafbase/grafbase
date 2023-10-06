mod types;

use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::{query_namespaced_postgres, query_postgres};

#[test]
fn pk_explicit_int() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { id: 1 }) {
                id
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "id": 1
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn namespaced() {
    let response = query_namespaced_postgres("Neon", |api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              neon {
                userCreate(input: { id: 1 }) {
                  id
                }
              }
            }    
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "neon": {
              "userCreate": {
                "id": 1
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn renamed() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id_field INT PRIMARY KEY
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { idField: 1 }) {
                idField
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "idField": 1
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn serial_id() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id_field SERIAL PRIMARY KEY
            )
        "#};

        api.execute_sql(schema).await;

        let mutation = indoc! {r#"
            mutation {
              userCreate(input: {}) {
                idField
              }
            }
        "#};

        let result = api.execute(mutation).await;

        assert_eq!(1, api.row_count("User").await);

        result
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCreate": {
              "idField": 1
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
