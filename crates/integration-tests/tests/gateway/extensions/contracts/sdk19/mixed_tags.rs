use serde_json::json;

#[test]
fn include_and_exclude_tags() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            user: User
            product: Product
            admin: Admin
        }

        type User {
            id: ID! @tag(name: "public")
            name: String! @tag(name: "public")
            email: String! @tag(name: "internal")
            secret: String! @tag(name: "secret")
        }

        type Product {
            id: ID! @tag(name: "public")
            name: String!
            price: Float! @tag(name: "internal")
        }

        type Admin @tag(name: "secret") {
            id: ID!
            permissions: [String!]!
        }
        "#,
        &json!({
            "includedTags": ["public", "internal"],
            "excludedTags": ["secret"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Product {
      id: ID!
      price: Float!
    }

    type Query {
      product: Product
      user: User
    }

    type User {
      email: String!
      id: ID!
      name: String!
    }
    "#);
}

#[test]
fn conflict_include_and_exclude_same_tag() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            user: User
            product: Product
        }

        type User {
            id: ID! @tag(name: "public")
            name: String! @tag(name: "conflict")
            email: String!
        }

        type Product @tag(name: "conflict") {
            id: ID!
            name: String!
        }
        "#,
        &json!({
            "includedTags": ["public", "conflict"],
            "excludedTags": ["conflict"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Query {
      user: User
    }

    type User {
      id: ID!
    }
    "#);
}

#[test]
fn multiple_tags_on_single_element() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            user: User
        }

        type User {
            id: ID! @tag(name: "public") @tag(name: "basic")
            name: String! @tag(name: "secret") @tag(name: "internal")
            email: String! @tag(name: "public") @tag(name: "secret")
        }

        "#,
        &json!({
            "includedTags": ["public"],
            "excludedTags": ["secret"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Query {
      user: User
    }

    type User {
      id: ID!
    }
    "#);
}

#[test]
fn union_with_mixed_tags() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            search: SearchResult
            media: Media
        }

        union SearchResult = User | Product | Article

        union Media @tag(name: "internal") = Book | Movie

        type User @tag(name: "public") {
            id: ID!
        }

        type Product @tag(name: "public") {
            name: String!
        }

        type Article @tag(name: "secret") {
            title: String!
        }

        type Book @tag(name: "public") {
            title: String!
        }

        type Movie @tag(name: "internal") {
            title: String!
        }
        "#,
        &json!({
            "includedTags": ["public", "internal"],
            "excludedTags": ["secret"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Book {
      title: String!
    }

    union Media = Book | Movie

    type Movie {
      title: String!
    }

    type Product {
      name: String!
    }

    type Query {
      media: Media
    }

    type User {
      id: ID!
    }
    "#);
}

#[test]
fn enum_values_with_mixed_tags() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            userRole: Role
            status: Status
        }

        enum Role {
            ADMIN @tag(name: "public") @tag(name: "internal")
            USER @tag(name: "public") 
            GUEST @tag(name: "public")
            SUPERUSER @tag(name: "secret")
        }

        enum Status @tag(name: "internal") {
            ACTIVE @tag(name: "public")
            INACTIVE @tag(name: "internal") 
            DELETED @tag(name: "secret")
        }
        "#,
        &json!({
            "includedTags": ["public", "internal"],
            "excludedTags": ["secret"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Query {
      status: Status
      userRole: Role
    }

    enum Role {
      ADMIN
      USER
      GUEST
    }

    enum Status {
      ACTIVE
      INACTIVE
    }
    "#);
}
