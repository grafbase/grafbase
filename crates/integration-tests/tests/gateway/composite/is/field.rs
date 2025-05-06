use integration_tests::{gateway::Gateway, runtime};

#[test]
fn basic() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: ID!
                    user: User! @is(field: "{ id: author_id }")
                }

                type User {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        if let Err(err) = result {
            panic!("{err}");
        }
    })
}

#[test]
fn invalid_type() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: Int!
                    user: User! @is(field: "{ id: author_id }")
                }

                type User {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Product.user, for directive @composite__is Cannot map Product.author_id (Int!) into User.id (ID!). See schema at 25:29:\n(graph: EXT, field: \"{ id: author_id }\")",
        )
        "#);
    })
}

#[test]
fn multiple_fields_mapping() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: ID!
                    category_id: ID!
                    user: User! @is(field: "{ id: author_id category: category_id }")
                }

                type User {
                    id: ID!
                    category: ID!
                }
                "#,
            )
            .try_build()
            .await;

        if let Err(err) = result {
            panic!("{err}");
        }
    })
}

#[test]
fn direct_field_mapping() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: ID!
                    user_id: ID! @is(field: "author_id")
                }
                "#,
            )
            .try_build()
            .await;

        if let Err(err) = result {
            panic!("{err}");
        }
    })
}

#[test]
fn incompatible_wrapping() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_ids: [ID!]!
                    user: User! @is(field: "{ id: author_ids }")
                }

                type User {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Product.user, for directive @composite__is Incompatible wrapping, cannot map Product.author_ids ([ID!]!) into User.id (ID!). See schema at 25:29:\n(graph: EXT, field: \"{ id: author_ids }\")",
        )
        "#);
    })
}

#[test]
fn inexistent_field() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    user: User! @is(field: "{ id: non_existent_field }")
                }

                type User {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Product.user, for directive @composite__is Type Product does not have a field named 'non_existent_field'. See schema at 24:29:\n(graph: EXT, field: \"{ id: non_existent_field }\")",
        )
        "#);
    })
}

#[test]
fn inexistent_target_field() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    user: User! @is(field: "{ non_existent_field: id }")
                }

                type User {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Product.user, for directive @composite__is Field 'non_existent_field' does not exist on Product.user. See schema at 24:29:\n(graph: EXT, field: \"{ non_existent_field: id }\")",
        )
        "#);
    })
}

#[test]
fn nullable_vs_required() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: ID
                    user: User! @is(field: "{ id: author_id }")
                }

                type User {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Product.user, for directive @composite__is Incompatible wrapping, cannot map Product.author_id (ID) into User.id (ID!). See schema at 25:29:\n(graph: EXT, field: \"{ id: author_id }\")",
        )
        "#);
    })
}

#[test]
fn multiple_fields_without_rename() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: ID!
                    category: String!
                    user: User! @is(field: "{ id: author_id category }")
                }

                type User {
                    id: ID!
                    category: String!
                }
                "#,
            )
            .try_build()
            .await;

        if let Err(err) = result {
            panic!("{err}");
        }
    })
}
