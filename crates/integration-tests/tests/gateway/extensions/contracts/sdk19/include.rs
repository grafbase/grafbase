use serde_json::json;

#[test]
fn field_definition() {
    let contract = super::run(
        r#"
        type Query {
            user: User @tag(name: "public")
            product: Product @tag(name: "internal")
        }

        type User {
            id: ID! @tag(name: "public")
            name: String!
            email: String!
        }

        type Product {
            id: ID!
            name: String!
            price: Float!
        }
        "#,
        &json!({
            "includedTags": ["public"]
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
fn object_type() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            user: User
            product: Product
        }

        type User @tag(name: "public") {
            id: ID!
            name: String!
        }

        type Product @tag(name: "internal") {
            id: ID!
            price: Float!
        }
        "#,
        &json!({
            "includedTags": ["public"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Query {
      user: User
    }

    type User {
      id: ID!
      name: String!
    }
    "#);
}

#[test]
fn interface_type() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            node: Node
            entity: Entity
        }

        interface Node @tag(name: "public") {
            id: ID!
        }

        interface Entity @tag(name: "internal") {
            name: String!
        }

        type User implements Node @tag(name: "public") {
            id: ID!
            email: String!
        }
        "#,
        &json!({
            "includedTags": ["public"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    interface Node {
      id: ID!
    }

    type Query {
      node: Node
    }

    type User implements Node {
      email: String!
      id: ID!
    }
    "#);
}

#[test]
fn union_type() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            search: SearchResult
            other: OtherResult
        }

        union SearchResult @tag(name: "public") = User | Product

        union OtherResult @tag(name: "internal") = Article | Comment

        type User @tag(name: "public") {
            id: ID!
        }

        type Product {
            name: String!
        }

        type Article @tag(name: "public") {
            title: String!
        }

        type Comment {
            text: String!
        }
        "#,
        &json!({
            "includedTags": ["public"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Article {
      title: String!
    }

    type Query {
      search: SearchResult
    }

    union SearchResult = User

    type User {
      id: ID!
    }
    "#);
}

#[test]
fn argument_definition() {
    let contract = super::run(
        r#"
        type Query {
            user(
                id: ID! @tag(name: "public")
                email: String @tag(name: "internal")
            ): User
            product(
                sku: String!
                category: String
            ): Product @tag(name: "public")
        }

        type User @tag(name: "public") {
            id: ID!
        }

        type Product @tag(name: "public") {
            sku: String!
        }
        "#,
        &json!({
            "includedTags": ["public"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Product {
      sku: String!
    }

    type Query {
      product(sku: String!, category: String): Product
      user(id: ID!): User
    }

    type User {
      id: ID!
    }
    "#);
}

#[test]
fn scalar_type() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            timestamp: DateTime
            uuid: UUID
        }

        scalar DateTime @tag(name: "public")
        scalar UUID @tag(name: "internal")
        "#,
        &json!({
            "includedTags": ["public"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    scalar DateTime

    type Query {
      timestamp: DateTime
    }
    "#);
}

#[test]
fn enum_type() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            userRole: Role
            status: Status
        }

        enum Role @tag(name: "public") {
            ADMIN
            USER
            GUEST
        }

        enum Status @tag(name: "internal") {
            ACTIVE
            INACTIVE
        }
        "#,
        &json!({
            "includedTags": ["public"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Query {
      userRole: Role
    }

    enum Role {
      ADMIN
      USER
      GUEST
    }
    "#);
}

#[test]
fn enum_value() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            userRole: Role
        }

        enum Role {
            ADMIN @tag(name: "public")
            USER @tag(name: "public")
            GUEST @tag(name: "internal")
            SUPERUSER @tag(name: "internal")
        }
        "#,
        &json!({
            "includedTags": ["public"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Query {
      userRole: Role
    }

    enum Role {
      ADMIN
      USER
    }
    "#);
}

#[test]
fn input_object() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            searchUsers(filter: UserFilter): [User]
            searchProducts(filter: ProductFilter): [Product]
        }

        input UserFilter @tag(name: "public") {
            name: String
            email: String
        }

        input ProductFilter @tag(name: "internal") {
            category: String
            minPrice: Float
        }

        type User @tag(name: "public") {
            id: ID!
        }

        type Product @tag(name: "public") {
            id: ID!
        }
        "#,
        &json!({
            "includedTags": ["public"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Product {
      id: ID!
    }

    type Query {
      searchProducts: [Product]
      searchUsers(filter: UserFilter): [User]
    }

    type User {
      id: ID!
    }

    input UserFilter {
      name: String
      email: String
    }
    "#);
}

#[test]
fn input_field_definition() {
    let contract = super::run(
        r#"
        type Query @tag(name: "public") {
            searchUsers(filter: UserFilter): [User]
        }

        input UserFilter {
            name: String @tag(name: "public")
            email: String @tag(name: "public")
            internalId: ID @tag(name: "internal")
            secretField: String @tag(name: "internal")
        }

        type User @tag(name: "public") {
            id: ID!
        }
        "#,
        &json!({
            "includedTags": ["public"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Query {
      searchUsers(filter: UserFilter): [User]
    }

    type User {
      id: ID!
    }

    input UserFilter {
      name: String
      email: String
    }
    "#);
}
