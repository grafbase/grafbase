mod joins;
mod types;

use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::{query_namespaced_postgres, query_postgres};

#[test]
fn by_pk_no_rename() {
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
              user(by: { id: 1 }) { id name }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 1,
              "name": "Musti"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

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
            INSERT INTO "User" (id, name) VALUES (1, 'Musti'), (2, 'Naukio')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              postgres {
                user(by: { id: 1 }) { id name }
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "postgres": {
              "user": {
                "id": 1,
                "name": "Musti"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn by_pk_with_rename() {
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

        let query = indoc! {r#"
            query {
              user(by: { idField: 1 }) { idField nameField }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "idField": 1,
              "nameField": "Musti"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn by_compound_pk() {
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
              ('Musti', 'meow@meow.com'),
              ('Naukio', 'purr@meow.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              user(by: { nameEmail: { name: "Naukio", email: "purr@meow.com" } }) {
                name
                email
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Naukio",
              "email": "purr@meow.com"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn by_compound_unique_with_nullable_column() {
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
            INSERT INTO "User" (name, email) VALUES
              ('Musti', 'meow@meow.com'),
              ('Naukio', NULL),
              ('Naukio', 'purr@meow.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              user(by: { nameEmail: { name: "Naukio", email: null } }) {
                name
                email
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Naukio",
              "email": null
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn by_compound_unique_with_nullable_column_emitting_field() {
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
            INSERT INTO "User" (name, email) VALUES
              ('Musti', 'meow@meow.com'),
              ('Naukio', NULL),
              ('Naukio', 'purr@meow.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              user(by: { nameEmail: { name: "Naukio" } }) {
                name
                email
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Naukio",
              "email": null
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn by_unique() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, email) VALUES
              (1, 'meow@meow.com'),
              (2, 'purr@meow.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              user(by: { email: "purr@meow.com" }) {
                id
                email
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 2,
              "email": "purr@meow.com"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn by_id_when_having_another_unique() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, email) VALUES
              (1, 'meow@meow.com'),
              (2, 'purr@meow.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              user(by: { id: 2 }) {
                id
                email
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 2,
              "email": "purr@meow.com"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn by_compound_unique() {
    let response = query_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                CONSTRAINT User_name_email_key UNIQUE (name, email)
            )    
        "#};

        api.execute_sql(schema).await;

        let insert = indoc! {r#"
            INSERT INTO "User" (id, name, email) VALUES
              (1, 'Musti', 'meow@meow.com'),
              (2, 'Naukio', 'purr@meow.com')
        "#};

        api.execute_sql(insert).await;

        let query = indoc! {r#"
            query {
              user(by: { nameEmail: { name: "Naukio", email: "purr@meow.com" } }) {
                id
                name
                email
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 2,
              "name": "Naukio",
              "email": "purr@meow.com"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
