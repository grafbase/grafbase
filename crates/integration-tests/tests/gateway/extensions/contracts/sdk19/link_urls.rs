use serde_json::json;

use integration_tests::{gateway::Gateway, runtime};

#[test]
fn import_tag() {
    let contract = runtime().block_on(async move {
        Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                extend schema 
                    @link(url: "https://example.com/tag", import: ["@tag"])
                    @link(url: "https://specs.apollo.dev/federation/v2.3", import: ["@composeDirective"])
                    @composeDirective(name: "@tag")


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
            )
            .with_extension("contracts-19")
            .with_extension("hooks-19")
            .with_toml_config(
                r#"
            [graph]
            introspection = true
            "#,
            )
            .build()
            .await
            .introspect()
            .header(
                "contract-key",
                serde_json::to_vec(&json!({
                    "includedTags": ["public", "internal"],
                    "excludedTags": ["secret"]
                }))
                .unwrap(),
            )
            .await
    });

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
fn import_tag_as_other() {
    let contract = runtime().block_on(async move {
        Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                extend schema 
                    @link(url: "https://example.com/tag", import: [{name: "@tag", as: "@other"}])
                    @link(url: "https://specs.apollo.dev/federation/v2.3", import: ["@composeDirective"])
                    @composeDirective(name: "@other")


                type Query @other(name: "public") {
                    user: User
                    product: Product
                    admin: Admin
                }

                type User {
                    id: ID! @other(name: "public")
                    name: String! @other(name: "public")
                    email: String! @other(name: "internal")
                    secret: String! @other(name: "secret")
                }

                type Product {
                    id: ID! @other(name: "public")
                    name: String!
                    price: Float! @other(name: "internal")
                }

                type Admin @other(name: "secret") {
                    id: ID!
                    permissions: [String!]!
                }
                "#,
            )
            .with_extension("contracts-19")
            .with_extension("hooks-19")
            .with_toml_config(
                r#"
            [graph]
            introspection = true
            "#,
            )
            .build()
            .await
            .introspect()
            .header(
                "contract-key",
                serde_json::to_vec(&json!({
                    "includedTags": ["public", "internal"],
                    "excludedTags": ["secret"]
                }))
                .unwrap(),
            )
            .await
    });

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
fn import_namespace() {
    let contract = runtime().block_on(async move {
        Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                extend schema 
                    @link(url: "https://example.com/tag")
                    @link(url: "https://specs.apollo.dev/federation/v2.3", import: ["@composeDirective"])
                    @composeDirective(name: "@tag__tag")


                type Query @tag__tag(name: "public") {
                    user: User
                    product: Product
                    admin: Admin
                }

                type User {
                    id: ID! @tag__tag(name: "public")
                    name: String! @tag__tag(name: "public")
                    email: String! @tag__tag(name: "internal")
                    secret: String! @tag__tag(name: "secret")
                }

                type Product {
                    id: ID! @tag__tag(name: "public")
                    name: String!
                    price: Float! @tag__tag(name: "internal")
                }

                type Admin @tag__tag(name: "secret") {
                    id: ID!
                    permissions: [String!]!
                }
                "#,
            )
            .with_extension("contracts-19")
            .with_extension("hooks-19")
            .with_toml_config(
                r#"
            [graph]
            introspection = true
            "#,
            )
            .build()
            .await
            .introspect()
            .header(
                "contract-key",
                serde_json::to_vec(&json!({
                    "includedTags": ["public", "internal"],
                    "excludedTags": ["secret"]
                }))
                .unwrap(),
            )
            .await
    });

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
fn import_as_namespace() {
    let contract = runtime().block_on(async move {
        Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                extend schema 
                    @link(url: "https://example.com/tag", as: "other")
                    @link(url: "https://specs.apollo.dev/federation/v2.3", import: ["@composeDirective"])
                    @composeDirective(name: "@other__tag")


                type Query @other__tag(name: "public") {
                    user: User
                    product: Product
                    admin: Admin
                }

                type User {
                    id: ID! @other__tag(name: "public")
                    name: String! @other__tag(name: "public")
                    email: String! @other__tag(name: "internal")
                    secret: String! @other__tag(name: "secret")
                }

                type Product {
                    id: ID! @other__tag(name: "public")
                    name: String!
                    price: Float! @other__tag(name: "internal")
                }

                type Admin @other__tag(name: "secret") {
                    id: ID!
                    permissions: [String!]!
                }
                "#,
            )
            .with_extension("contracts-19")
            .with_extension("hooks-19")
            .with_toml_config(
                r#"
            [graph]
            introspection = true
            "#,
            )
            .build()
            .await
            .introspect()
            .header(
                "contract-key",
                serde_json::to_vec(&json!({
                    "includedTags": ["public", "internal"],
                    "excludedTags": ["secret"]
                }))
                .unwrap(),
            )
            .await
    });

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
