use expect_test::expect;
use indoc::indoc;
use integration_tests::postgresql::{introspect_namespaced_neon, introspect_neon};

#[test]
fn table_with_serial_primary_key() {
    let response = introspect_neon(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        type Query {
          """
            Query a single User by a field
          """
          user(by: UserByInput!): User
        }

        type User {
          id: Int!
        }

        input UserByInput {
          id: Int
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_enum_field() {
    let response = introspect_neon(|api| async move {
        let r#type = indoc! {r#"
            CREATE TYPE street_light AS ENUM ('red', 'yellow', 'green');
        "#};

        api.execute_sql(r#type).await;

        let table = indoc! {r#"
            CREATE TABLE "A" (
              id INT PRIMARY KEY,
              val street_light NOT NULL
            );
        "#};

        api.execute_sql(table).await;
    });

    let expected = expect![[r#"
        type A {
          id: Int!
          val: StreetLight!
        }

        input AByInput {
          id: Int
        }

        type Query {
          """
            Query a single A by a field
          """
          a(by: AByInput!): A
        }

        enum StreetLight {
          RED
          YELLOW
          GREEN
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_int_primary_key() {
    let response = introspect_neon(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        type Query {
          """
            Query a single User by a field
          """
          user(by: UserByInput!): User
        }

        type User {
          id: Int!
        }

        input UserByInput {
          id: Int
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_int_unique() {
    let response = introspect_neon(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        type Query {
          """
            Query a single User by a field
          """
          user(by: UserByInput!): User
        }

        type User {
          id: Int!
        }

        input UserByInput {
          id: Int
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_serial_primary_key_string_unique() {
    let response = introspect_neon(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        type Query {
          """
            Query a single User by a field
          """
          user(by: UserByInput!): User
        }

        type User {
          id: Int!
          email: String!
        }

        input UserByInput {
          email: String
          id: Int
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_composite_primary_key() {
    let response = introspect_neon(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                CONSTRAINT "User_pkey" PRIMARY KEY (name, email)
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        type Query {
          """
            Query a single User by a field
          """
          user(by: UserByInput!): User
        }

        type User {
          name: String!
          email: String!
        }

        input UserByInput {
          nameEmail: UserNameEmailInput
        }

        input UserNameEmailInput {
          name: String!
          email: String!
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn two_schemas_same_table_name() {
    let response = introspect_neon(|api| async move {
        api.execute_sql(r#"CREATE SCHEMA private"#).await;

        let schema = indoc! {r#"
            CREATE TABLE private."User" (
                id SERIAL PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;

        let schema = indoc! {r#"
            CREATE TABLE public."User" (
                id SERIAL PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        type PrivateUser {
          id: Int!
        }

        input PrivateUserByInput {
          id: Int
        }

        type PublicUser {
          id: Int!
        }

        input PublicUserByInput {
          id: Int
        }

        type Query {
          """
            Query a single PrivateUser by a field
          """
          privateUser(by: PrivateUserByInput!): PrivateUser
          """
            Query a single PublicUser by a field
          """
          publicUser(by: PublicUserByInput!): PublicUser
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_serial_primary_key_namespaced() {
    let response = introspect_namespaced_neon("Neon", |api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY
            )
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        type NeonQuery {
          """
            Query a single NeonUser by a field
          """
          user(by: NeonUserByInput!): NeonUser
        }

        type NeonUser {
          id: Int!
        }

        input NeonUserByInput {
          id: Int
        }

        type Query {
          neon: NeonQuery
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn two_tables_with_single_column_foreign_key() {
    let response = introspect_neon(|api| async move {
        let create_user = indoc! {r#"
           CREATE TABLE "User" (
               id SERIAL PRIMARY KEY,
               name VARCHAR(255) NOT NULL 
           );
       "#};

        api.execute_sql(create_user).await;

        let create_blog = indoc! {r#"
            CREATE TABLE "Blog" (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT,
                user_id INT NOT NULL,
                CONSTRAINT "Blog_User" FOREIGN KEY (user_id) REFERENCES "User"(id)
            )    
        "#};

        api.execute_sql(create_blog).await;
    });

    let expected = expect![[r#"
        type Blog {
          id: Int!
          title: String!
          content: String
          userId: Int!
          user: User!
        }

        input BlogByInput {
          id: Int
        }

        type BlogConnection {
          edges: [BlogEdge]!
          pageInfo: PageInfo!
        }

        type BlogEdge {
          node: Blog!
          cursor: String!
        }

        input BlogOrderByInput {
          id: OrderByDirection
          title: OrderByDirection
          content: OrderByDirection
          userId: OrderByDirection
        }

        enum OrderByDirection {
          ASC
          DESC
        }

        type PageInfo {
          hasNextPage: Boolean!
          hasPreviousPage: Boolean!
          startCursor: String
          endCursor: String
        }

        type Query {
          """
            Query a single Blog by a field
          """
          blog(by: BlogByInput!): Blog
          """
            Query a single User by a field
          """
          user(by: UserByInput!): User
        }

        type User {
          id: Int!
          name: String!
          blogs(first: Int, last: Int, before: String, after: String, orderBy: [BlogOrderByInput!]): BlogConnection
        }

        input UserByInput {
          id: Int
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}
