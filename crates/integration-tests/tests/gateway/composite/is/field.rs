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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: Cannot map Product.author_id (Int!) into User.id (ID!)
        See schema at 25:29:
        (graph: EXT, field: "{ id: author_id }")
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
fn parsing_failure() {
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
                    category: Category!
                    user: User! @is(field: "{[]}")
                }

                type User {
                    id: ID!
                    category: Category!
                }

                type Category {
                    id: ID!
                    name: String!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: 
        {[]}
         ^
        invalid object

        See schema at 26:29:
        (graph: EXT, field: "{[]}")
        "#);
    })
}

#[test]
fn missing_field_argument() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_federated_sdl(
                r#"
                type Product
                  @join__type(graph: EXT)
                {
                  author_id: ID!
                  code: String!
                  id: ID!
                  user: User! @composite__is(graph: EXT)
                }

                type User
                  @join__type(graph: EXT)
                {
                  id: ID!
                }

                type Query
                {
                  productBatch(ids: [ID!]!): [Product!]! @join__field(graph: EXT)
                }

                enum join__Graph
                {
                  EXT @join__graph(name: "ext")
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: missing field: field
        See schema at 7:29:
        (graph: EXT)
        "#);
    })
}

#[test]
fn missing_graph_argument() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_federated_sdl(
                r#"
                type Product
                  @join__type(graph: EXT)
                {
                  author_id: ID!
                  code: String!
                  id: ID!
                  user: User! @composite__is(field: "{ id: author_id }")
                }

                type User
                  @join__type(graph: EXT)
                {
                  id: ID!
                }

                type Query
                {
                  productBatch(ids: [ID!]!): [Product!]! @join__field(graph: EXT)
                }

                enum join__Graph
                {
                  EXT @join__graph(name: "ext")
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: missing field: graph
        See schema at 7:29:
        (field: "{ id: author_id }")
        "#);
    })
}

#[test]
fn invalid_directive_argument_type() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_federated_sdl(
                r#"
                type Product
                  @join__type(graph: EXT)
                {
                  author_id: ID!
                  code: String!
                  id: ID!
                  user: User! @composite__is(graph: EXT, field: 0)
                }

                type User
                  @join__type(graph: EXT)
                {
                  id: ID!
                }

                type Query
                {
                  productBatch(ids: [ID!]!): [Product!]! @join__field(graph: EXT)
                }

                enum join__Graph
                {
                  EXT @join__graph(name: "ext")
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: found a Int where we expected a String
        See schema at 7:29:
        (graph: EXT, field: 0)
        "#);
    })
}

#[test]
fn unknown_directive_argument() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_federated_sdl(
                r#"
                type Product
                  @join__type(graph: EXT)
                {
                  author_id: ID!
                  code: String!
                  id: ID!
                  user: User! @composite__is(graph: EXT, field: "{ id: author_id }", yes: true)
                }

                type User
                  @join__type(graph: EXT)
                {
                  id: ID!
                }

                type Query
                {
                  productBatch(ids: [ID!]!): [Product!]! @join__field(graph: EXT)
                }

                enum join__Graph
                {
                  EXT @join__graph(name: "ext")
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: unknown field: yes
        See schema at 7:29:
        (graph: EXT, field: "{ id: author_id }", yes: true)
        "#);
    })
}

#[test]
fn cannot_map_an_object_into_a_field() {
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
                    category: Category!
                    user: User! @is(field: "{ id: author_id category }")
                }

                type User {
                    id: ID!
                    category: Category!
                }

                type Category {
                    id: ID!
                    name: String!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: Fields must be explictely selected on Product.category (Category!), it's not a scalar or enum
        See schema at 26:29:
        (graph: EXT, field: "{ id: author_id category }")
        "#);

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
                    category: Category!
                    user: User! @is(field: "{ id: author_id category: category.{ id name } }")
                }

                type User {
                    id: ID!
                    category: Category!
                }

                type Category {
                    id: ID!
                    name: String!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: Computed object fields can only be mapped to parent scalar/enum fields
        See schema at 26:29:
        (graph: EXT, field: "{ id: author_id category: category.{ id name } }")
        "#);
    })
}

#[test]
fn missing_required_field() {
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
                    user: User! @is(field: "{ id: author_id }")
                }

                type User {
                    id: ID!
                    category: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: For Product.user, field 'category' is required but doesn't have any mapping
        See schema at 26:29:
        (graph: EXT, field: "{ id: author_id }")
        "#);
    })
}

#[test]
fn missing_required_external_field() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.apollo.dev/federation/v2.0", import: ["@external"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is", "@key"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: ID!
                    category_id: ID!
                    user: User! @is(field: "{ id: author_id }")
                }

                type User @key(fields: "id") {
                    id: ID!
                    category: String! @external
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
fn missing_nullable_field() {
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
                    user: User! @is(field: "{ id: author_id }")
                }

                type User {
                    id: ID!
                    name: String
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user_id, for directive @composite__is: @is can only be used on fields to compute an object/interface.
        See schema at 25:3:
        user_id: ID! @composite__is(graph: EXT, field: "author_id")
        "#);
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: Incompatible wrapping, cannot map Product.author_ids ([ID!]!) into User.id (ID!)
        See schema at 25:29:
        (graph: EXT, field: "{ id: author_ids }")
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: Type Product does not have a field named 'non_existent_field'
        See schema at 24:29:
        (graph: EXT, field: "{ id: non_existent_field }")
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: Field 'non_existent_field' does not exist on Product.user
        See schema at 24:29:
        (graph: EXT, field: "{ non_existent_field: id }")
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__is: Incompatible wrapping, cannot map Product.author_id (ID) into User.id (ID!)
        See schema at 25:29:
        (graph: EXT, field: "{ id: author_id }")
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
