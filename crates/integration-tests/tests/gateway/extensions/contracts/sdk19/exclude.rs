use serde_json::json;

#[test]
fn field_definition() {
    let contract = super::run(
        r#"
        type Query {
            user: User
            product: Product
        }

        type User {
            id: ID!
            name: String!
            email: String! @tag(name: "internal")
        }

        type Product {
            id: ID!
            name: String!
            price: Float! @tag(name: "public")
        }
        "#,
        &json!({
            "excludedTags": ["internal"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Product {
      id: ID!
      name: String!
      price: Float!
    }

    type Query {
      product: Product
      user: User
    }

    type User {
      id: ID!
      name: String!
    }
    "#);
}

#[test]
fn object_type() {
    let contract = super::run(
        r#"
        type Query {
            user: User
            product: Product
        }

        type User @tag(name: "internal") {
            id: ID!
            name: String!
        }

        type Product @tag(name: "public") {
            id: ID!
            price: Float!
        }
        "#,
        &json!({
            "excludedTags": ["internal"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Product {
      id: ID!
      price: Float!
    }

    type Query {
      product: Product
    }
    "#);
}

#[test]
fn interface_type() {
    let contract = super::run(
        r#"
        type Query {
            node: Node
            entity: Entity
        }

        interface Node @tag(name: "internal") {
            id: ID!
        }

        interface Entity @tag(name: "public") {
            name: String!
        }

        type User implements Node & Entity {
            id: ID!
            name: String!
            email: String!
        }
        "#,
        &json!({
            "excludedTags": ["internal"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    interface Entity {
      name: String!
    }

    type Query {
      entity: Entity
    }

    type User implements Entity {
      email: String!
      id: ID!
      name: String!
    }
    "#);
}

#[test]
fn union_type() {
    let contract = super::run(
        r#"
        type Query {
            search: SearchResult
            other: OtherResult
        }

        union SearchResult @tag(name: "internal") = User | Product

        union OtherResult @tag(name: "public") = Article | Comment

        type User {
            id: ID!
        }

        type Product {
            name: String!
        }

        type Article {
            title: String!
        }

        type Comment @tag(name: "internal") {
            text: String!
        }
        "#,
        &json!({
            "excludedTags": ["internal"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Article {
      title: String!
    }

    union OtherResult = Article

    type Product {
      name: String!
    }

    type Query {
      other: OtherResult
    }

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
                category: String @tag(name: "public")
            ): Product
        }

        type User {
            id: ID!
        }

        type Product {
            sku: String!
        }
        "#,
        &json!({
            "excludedTags": ["internal"]
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
        type Query {
            timestamp: DateTime
            uuid: UUID
        }

        scalar DateTime @tag(name: "internal")
        scalar UUID @tag(name: "public")
        "#,
        &json!({
            "excludedTags": ["internal"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Query {
      uuid: UUID
    }

    scalar UUID
    "#);
}

#[test]
fn enum_type() {
    let contract = super::run(
        r#"
        type Query {
            userRole: Role
            status: Status
        }

        enum Role @tag(name: "internal") {
            ADMIN
            USER
            GUEST
        }

        enum Status @tag(name: "public") {
            ACTIVE
            INACTIVE
        }
        "#,
        &json!({
            "excludedTags": ["internal"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Query {
      status: Status
    }

    enum Status {
      ACTIVE
      INACTIVE
    }
    "#);
}

#[test]
fn enum_value() {
    let contract = super::run(
        r#"
        type Query {
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
            "excludedTags": ["internal"]
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
        type Query {
            searchUsers(filter: UserFilter): [User]
            searchProducts(filter: ProductFilter): [Product]
        }

        input UserFilter @tag(name: "internal") {
            name: String
            email: String
        }

        input ProductFilter @tag(name: "public") {
            category: String
            minPrice: Float
        }

        type User {
            id: ID!
        }

        type Product {
            id: ID!
        }
        "#,
        &json!({
            "excludedTags": ["internal"]
        }),
    );
    insta::assert_snapshot!(contract, @r#"
    type Product {
      id: ID!
    }

    input ProductFilter {
      category: String
      minPrice: Float
    }

    type Query {
      searchProducts(filter: ProductFilter): [Product]
      searchUsers: [User]
    }

    type User {
      id: ID!
    }
    "#);
}

#[test]
fn input_field_definition() {
    let contract = super::run(
        r#"
        type Query {
            searchUsers(filter: UserFilter): [User]
        }

        input UserFilter {
            name: String @tag(name: "public")
            email: String @tag(name: "public")
            internalId: ID @tag(name: "internal")
            secretField: String @tag(name: "internal")
        }

        type User {
            id: ID!
        }
        "#,
        &json!({
            "excludedTags": ["internal"]
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
