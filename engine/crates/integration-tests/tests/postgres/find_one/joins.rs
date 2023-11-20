use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::query_postgres;

#[test]
fn one_to_one_join_parent_side() {
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

        let query = indoc! {r"
            query {
              user(by: { id: 2 }) {
                id
                name
                profile { description }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 2,
              "name": "Naukio",
              "profile": {
                "description": "purrpurrpurr"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn one_to_one_join_between_schemas() {
    let response = query_postgres(|api| async move {
        let private_schema = indoc! {r#"
            CREATE SCHEMA "private";
        "#};

        api.execute_sql(private_schema).await;

        let public_table = indoc! {r#"
            CREATE TABLE "public"."User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        "#};

        api.execute_sql(public_table).await;

        let private_table = indoc! {r#"
            CREATE TABLE "private"."Secret" (
                id INT PRIMARY KEY,
                secret_name VARCHAR(255) NOT NULL,
                user_id INT NULL UNIQUE,
                CONSTRAINT User_User_fkey FOREIGN KEY (user_id) REFERENCES "public"."User" (id)
            );
        "#};

        api.execute_sql(private_table).await;

        let insert_public = indoc! {r#"
            INSERT INTO "public"."User" (id, name) VALUES
              (1, 'Musti'),
              (2, 'Naukio')
        "#};

        api.execute_sql(insert_public).await;

        let insert_private = indoc! {r#"
            INSERT INTO "private"."Secret" (id, user_id, secret_name) VALUES
              (1, 1, 'Naukio'),
              (2, 2, 'Musti')
        "#};

        api.execute_sql(insert_private).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                id
                name
                secret { secretName }
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
              "name": "Musti",
              "secret": {
                "secretName": "Naukio"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn one_to_one_join_between_schemas_using_duplicate_table_names() {
    let response = query_postgres(|api| async move {
        let private_schema = indoc! {r#"
            CREATE SCHEMA "private";
        "#};

        api.execute_sql(private_schema).await;

        let public_table = indoc! {r#"
            CREATE TABLE "public"."User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        "#};

        api.execute_sql(public_table).await;

        let private_table = indoc! {r#"
            CREATE TABLE "private"."User" (
                id INT PRIMARY KEY,
                secret_name VARCHAR(255) NOT NULL,
                user_id INT NULL UNIQUE,
                CONSTRAINT User_User_fkey FOREIGN KEY (user_id) REFERENCES "public"."User" (id)
            );
        "#};

        api.execute_sql(private_table).await;

        let insert_public = indoc! {r#"
            INSERT INTO "public"."User" (id, name) VALUES
              (1, 'Musti'),
              (2, 'Naukio')
        "#};

        api.execute_sql(insert_public).await;

        let insert_private = indoc! {r#"
            INSERT INTO "private"."User" (id, user_id, secret_name) VALUES
              (1, 1, 'Naukio'),
              (2, 2, 'Musti')
        "#};

        api.execute_sql(insert_private).await;

        let query = indoc! {r"
            query {
              publicUser(by: { id: 1 }) {
                id
                name
                privateUser { secretName }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "publicUser": {
              "id": 1,
              "name": "Musti",
              "privateUser": {
                "secretName": "Naukio"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn one_to_many_join_between_schemas() {
    let response = query_postgres(|api| async move {
        let private_schema = indoc! {r#"
            CREATE SCHEMA "private";
        "#};

        api.execute_sql(private_schema).await;

        let public_table = indoc! {r#"
            CREATE TABLE "public"."User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        "#};

        api.execute_sql(public_table).await;

        let private_table = indoc! {r#"
            CREATE TABLE "private"."User" (
                id INT PRIMARY KEY,
                secret_name VARCHAR(255) NOT NULL,
                user_id INT NULL,
                CONSTRAINT User_User_fkey FOREIGN KEY (user_id) REFERENCES "public"."User" (id)
            );
        "#};

        api.execute_sql(private_table).await;

        let insert_public = indoc! {r#"
            INSERT INTO "public"."User" (id, name) VALUES
              (1, 'Musti'),
              (2, 'Naukio')
        "#};

        api.execute_sql(insert_public).await;

        let insert_private = indoc! {r#"
            INSERT INTO "private"."User" (id, user_id, secret_name) VALUES
              (1, 1, 'Naukio'),
              (2, 1, 'Musti'),
              (3, 2, 'Pertti'),
              (4, 2, 'Matti')
        "#};

        api.execute_sql(insert_private).await;

        let query = indoc! {r"
            query {
              publicUser(by: { id: 1 }) {
                id
                name
                privateUsers(first: 1000) { edges { node { secretName } } }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "publicUser": {
              "id": 1,
              "name": "Musti",
              "privateUsers": {
                "edges": [
                  {
                    "node": {
                      "secretName": "Naukio"
                    }
                  },
                  {
                    "node": {
                      "secretName": "Musti"
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
fn one_to_one_join_parent_side_compound_fk() {
    let response = query_postgres(|api| async move {
        let user_table = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                CONSTRAINT User_name_email_pk PRIMARY KEY (name, email)
            );
        "#};

        api.execute_sql(user_table).await;

        let profile_table = indoc! {r#"
            CREATE TABLE "Profile" (
                user_name VARCHAR(255) NULL,
                user_email VARCHAR(255) NULL,
                description TEXT NOT NULL,
                CONSTRAINT Profile_name_email_key UNIQUE (user_name, user_email),
                CONSTRAINT Profile_User_fkey FOREIGN KEY (user_name, user_email) REFERENCES "User" (name, email)
            )  
        "#};

        api.execute_sql(profile_table).await;

        let insert_users = indoc! {r#"
            INSERT INTO "User" (name, email) VALUES
              ('Musti', 'meow1@hotmail.com'),
              ('Musti', 'meow2@hotmail.com')
        "#};

        api.execute_sql(insert_users).await;

        let insert_profiles = indoc! {r#"
            INSERT INTO "Profile" (user_name, user_email, description) VALUES
              ('Musti', 'meow1@hotmail.com', 'meowmeowmeow'),
              ('Musti', 'meow2@hotmail.com', 'purrpurrpurr')
        "#};

        api.execute_sql(insert_profiles).await;

        let query = indoc! {r#"
            query {
              user(by: { nameEmail: { name: "Musti", email: "meow2@hotmail.com" } }) {
                name
                email
                profile { description }
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Musti",
              "email": "meow2@hotmail.com",
              "profile": {
                "description": "purrpurrpurr"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn one_to_one_join_child_side() {
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

        let query = indoc! {r"
            query {
              profile(by: { id: 2 }) {
                description
                user { id name }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "profile": {
              "description": "purrpurrpurr",
              "user": {
                "id": 2,
                "name": "Naukio"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn one_to_one_to_one_join() {
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

        let extra_table = indoc! {r#"
            CREATE TABLE "Extra" (
                id INT PRIMARY KEY,
                profile_id INT NULL UNIQUE,
                number int NOT NULL,
                CONSTRAINT Extra_Profile_fkey FOREIGN KEY (profile_id) REFERENCES "Profile" (id)
            )  
        "#};

        api.execute_sql(extra_table).await;

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

        let insert_extras = indoc! {r#"
            INSERT INTO "Extra" (id, profile_id, number) VALUES
              (1, 1, 420),
              (2, 2, 666)
        "#};

        api.execute_sql(insert_extras).await;

        let query = indoc! {r"
            query {
              user(by: { id: 2 }) {
                id
                name
                profile { description extra { number } }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "id": 2,
              "name": "Naukio",
              "profile": {
                "description": "purrpurrpurr",
                "extra": {
                  "number": 666
                }
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn one_to_many_join_child_side() {
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
              blog(by: { id: 2 }) {
                title
                user { id name }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "blog": {
              "title": "Sayonara...",
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
fn one_to_many_join_parent_side() {
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
              user(by: { id: 1 }) {
                name
                blogs(first: 10000) { edges { node { id title } } }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
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
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn nested_one_to_many_joins_parent_side() {
    let response = query_postgres(|api| async move {
        let user_table = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        "#};

        api.execute_sql(user_table).await;

        let blog_table = indoc! {r#"
            CREATE TABLE "Blog" (
                id INT PRIMARY KEY,
                user_id INT NOT NULL,
                title VARCHAR(255) NOT NULL,
                CONSTRAINT Blog_User_fkey FOREIGN KEY (user_id) REFERENCES "User" (id)
            )  
        "#};

        api.execute_sql(blog_table).await;

        let post_table = indoc! {r#"
            CREATE TABLE "Post" (
                id INT PRIMARY KEY,
                blog_id INT NOT NULL,
                content TEXT NOT NULL,
                CONSTRAINT Post_Blog_fkey FOREIGN KEY (blog_id) REFERENCES "Blog" (id)
            )  
        "#};

        api.execute_sql(post_table).await;

        let insert_users = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES
              (1, 'Musti'),
              (2, 'Naukio')
        "#};

        api.execute_sql(insert_users).await;

        let insert_blogs = indoc! {r#"
            INSERT INTO "Blog" (id, user_id, title) VALUES
              (1, 1, 'Hello, world!'),
              (2, 1, 'Sayonara...'),
              (3, 2, 'Meow meow?')
        "#};

        api.execute_sql(insert_blogs).await;

        let insert_blogs = indoc! {r#"
            INSERT INTO "Post" (id, blog_id, content) VALUES
              (1, 1, 'meowmeow'),
              (2, 2, 'uwuwuwuwu'),
              (3, 3, 'Meow meow?')
        "#};

        api.execute_sql(insert_blogs).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                name
                blogs(first: 1000) { 
                  edges {
                    node {
                      id
                      title
                      posts(first: 1000) {
                        edges {
                          node {
                            id
                            content
                          }
                        }
                      }
                    }
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
            "user": {
              "name": "Musti",
              "blogs": {
                "edges": [
                  {
                    "node": {
                      "id": 1,
                      "title": "Hello, world!",
                      "posts": {
                        "edges": [
                          {
                            "node": {
                              "id": 1,
                              "content": "meowmeow"
                            }
                          }
                        ]
                      }
                    }
                  },
                  {
                    "node": {
                      "id": 2,
                      "title": "Sayonara...",
                      "posts": {
                        "edges": [
                          {
                            "node": {
                              "id": 2,
                              "content": "uwuwuwuwu"
                            }
                          }
                        ]
                      }
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
fn one_to_many_join_parent_side_with_first() {
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
              user(by: { id: 1 }) {
                name
                blogs(first: 1) { edges { node { id title } } }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Musti",
              "blogs": {
                "edges": [
                  {
                    "node": {
                      "id": 1,
                      "title": "Hello, world!"
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
fn one_to_many_join_parent_side_with_last() {
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
              user(by: { id: 1 }) {
                name
                blogs(last: 1) { edges { node { id title } } }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Musti",
              "blogs": {
                "edges": [
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
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn one_to_many_join_parent_side_with_single_column_descending_order() {
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
              user(by: { id: 1 }) {
                name
                blogs(first: 10, orderBy: [{ id: DESC }]) { edges { node { id title } } }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Musti",
              "blogs": {
                "edges": [
                  {
                    "node": {
                      "id": 2,
                      "title": "Sayonara..."
                    }
                  },
                  {
                    "node": {
                      "id": 1,
                      "title": "Hello, world!"
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
fn one_to_many_join_parent_side_with_compound_column_ordering_with_last() {
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
                description VARCHAR(255) NOT NULL,
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
            INSERT INTO "Blog" (id, user_id, description, title) VALUES
              (1, 1, 'a', 'a'),
              (2, 1, 'a', 'b'),
              (3, 1, 'b', 'c')
        "#};

        api.execute_sql(insert_profiles).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                name
                blogs(last: 2, orderBy: [{ description: DESC }, { title: DESC }]) {
                  edges {
                    node {
                      id
                      description
                      title
                    }
                  } 
                }
              }
            }
        "};

        api.execute(query).await
    });

    // description order: b, a, a
    // title order:       c, b, a
    // choosing the last two: (a, b) and (a, a)
    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Musti",
              "blogs": {
                "edges": [
                  {
                    "node": {
                      "id": 2,
                      "description": "a",
                      "title": "b"
                    }
                  },
                  {
                    "node": {
                      "id": 1,
                      "description": "a",
                      "title": "a"
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
fn one_to_many_join_parent_side_with_single_column_descending_order_with_last() {
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
              user(by: { id: 1 }) {
                name
                blogs(last: 1, orderBy: [{ id: DESC }]) { edges { node { id title } } }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Musti",
              "blogs": {
                "edges": [
                  {
                    "node": {
                      "id": 1,
                      "title": "Hello, world!"
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
fn two_one_to_many_joins_parent_side() {
    let response = query_postgres(|api| async move {
        let user_table = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        "#};

        api.execute_sql(user_table).await;

        let blog_table = indoc! {r#"
            CREATE TABLE "Blog" (
                id INT PRIMARY KEY,
                user_id INT NOT NULL,
                title VARCHAR(255) NOT NULL,
                CONSTRAINT Blog_User_fkey FOREIGN KEY (user_id) REFERENCES "User" (id)
            )  
        "#};

        api.execute_sql(blog_table).await;

        let cat_table = indoc! {r#"
            CREATE TABLE "Cat" (
                id INT PRIMARY KEY,
                user_id INT NOT NULL,
                name VARCHAR(255) NOT NULL,
                CONSTRAINT Cat_User_fkey FOREIGN KEY (user_id) REFERENCES "User" (id)
            )  
        "#};

        api.execute_sql(cat_table).await;

        let insert_users = indoc! {r#"
            INSERT INTO "User" (id, name) VALUES
              (1, 'Musti'),
              (2, 'Naukio')
        "#};

        api.execute_sql(insert_users).await;

        let insert_blogs = indoc! {r#"
            INSERT INTO "Blog" (id, user_id, title) VALUES
              (1, 1, 'Hello, world!'),
              (2, 1, 'Sayonara...'),
              (3, 2, 'Meow meow?')
        "#};

        api.execute_sql(insert_blogs).await;

        let insert_cats = indoc! {r#"
            INSERT INTO "Cat" (id, user_id, name) VALUES
              (1, 1, 'Musti'),
              (2, 1, 'Naukio'),
              (3, 2, 'Pertti')
        "#};

        api.execute_sql(insert_cats).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                name
                blogs(first: 1000) { edges { node { id title } } }
                cats(first: 100) { edges { node { id name } } }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
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
              },
              "cats": {
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
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn one_to_one_with_one_to_many_joins_parent_side() {
    let response = query_postgres(|api| async move {
        let user_table = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        "#};

        api.execute_sql(user_table).await;

        let blog_table = indoc! {r#"
            CREATE TABLE "Blog" (
                id INT PRIMARY KEY,
                user_id INT NOT NULL,
                title VARCHAR(255) NOT NULL,
                CONSTRAINT Blog_User_fkey FOREIGN KEY (user_id) REFERENCES "User" (id)
            )  
        "#};

        api.execute_sql(blog_table).await;

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

        let insert_blogs = indoc! {r#"
            INSERT INTO "Blog" (id, user_id, title) VALUES
              (1, 1, 'Hello, world!'),
              (2, 1, 'Sayonara...'),
              (3, 2, 'Meow meow?')
        "#};

        api.execute_sql(insert_blogs).await;

        let insert_profiles = indoc! {r#"
            INSERT INTO "Profile" (id, user_id, description) VALUES
              (1, 1, 'meow'),
              (2, 2, 'uwu')
        "#};

        api.execute_sql(insert_profiles).await;

        let query = indoc! {r"
            query {
              user(by: { id: 1 }) {
                name
                blogs(first: 10) { edges { node { id title } } }
                profile { description }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
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
              },
              "profile": {
                "description": "meow"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}
