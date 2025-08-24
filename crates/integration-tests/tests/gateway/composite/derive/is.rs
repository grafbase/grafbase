use integration_tests::{gateway::Gateway, runtime};

#[test]
fn invalid_type() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is", "@key", "@derive"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: Int!
                    user: User! @derive @is(field: "{ id: author_id }")
                }

                type User @key(fields: "id") {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Product.user, for directive @composite__derive: for associated @is directive: Cannot map Product.author_id (Int!) into User.id (ID!)
        24 |   id: ID!
        25 |   user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: "{ id: author_id }")
                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        26 | }
        "#);
    })
}

#[test]
fn invalid_wrapping() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is", "@key", "@derive"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_ids: [ID!]!
                    user: User! @derive @is(field: "{ id: author_ids }")
                }

                type User @key(fields: "id") {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Product.user, for directive @composite__derive: for associated @is directive: Incompatible wrapping, cannot map Product.author_ids ([ID!]!) into User.id (ID!)
        24 |   id: ID!
        25 |   user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: "{ id: author_ids }")
                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        26 | }
        "#);
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is", "@key", "@derive"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: ID!
                    user: User! @derive @is(field: "{[]}")
                }

                type User @key(fields: "id") {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Product.user, for directive @composite__derive: for associated @is directive: 
        {[]}
         ^
        invalid object

        24 |   id: ID!
        25 |   user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: "{[]}")
                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^
        26 | }
        "#);
    })
}

#[test]
fn federated_sdl_missing_field_argument() {
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
                  user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT)
                }

                type User
                  @join__type(graph: EXT, key: "id", resolvable: false)
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

        insta::assert_snapshot!(result.unwrap_err(), @r"
        * At site Product.user, for directive @composite__derive: for associated @is directive: missing field: field
        6 |   id: ID!
        7 |   user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT)
                                                                       ^^^^^^^^^^^^
        8 | }
        ");
    })
}

#[test]
fn federated_sdl_missing_graph_argument() {
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
                  user: User! @composite__derive(graph: EXT) @composite__is(field: "{ id: author_id }")
                }

                type User
                  @join__type(graph: EXT, key: "id", resolvable: false)
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
        * At site Product.user, for directive @composite__derive: for associated @is directive: missing field: graph
        6 |   id: ID!
        7 |   user: User! @composite__derive(graph: EXT) @composite__is(field: "{ id: author_id }")
                                                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        8 | }
        "#);
    })
}

#[test]
fn federated_sdl_invalid_directive_argument_type() {
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
                  user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: 0)
                }

                type User
                  @join__type(graph: EXT, key: "id", resolvable: false)
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

        insta::assert_snapshot!(result.unwrap_err(), @r"
        * At site Product.user, for directive @composite__derive: for associated @is directive: found a Int where we expected a String
        6 |   id: ID!
        7 |   user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: 0)
                                                                       ^^^^^^^^^^^^^^^^^^^^^^
        8 | }
        ");
    })
}

#[test]
fn federated_sdl_unknown_directive_argument() {
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
                  user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: "{ id: author_id }", yes: true)
                }

                type User
                  @join__type(graph: EXT, key: "id", resolvable: false)
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
        * At site Product.user, for directive @composite__derive: for associated @is directive: unknown field: yes
        6 |   id: ID!
        7 |   user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: "{ id: author_id }", yes: true)
                                                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        8 | }
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is", "@key", "@derive"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    user: User! @derive @is(field: "{ id: non_existent_field }")
                }

                type User @key(fields: "id") {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Product.user, for directive @composite__derive: for associated @is directive: Type Product does not have a field named 'non_existent_field'
        23 |   id: ID!
        24 |   user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: "{ id: non_existent_field }")
                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        25 | }
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is", "@key", "@derive"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    user: User! @derive @is(field: "{ non_existent_field: id }")
                }

                type User @key(fields: "id") {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Product.user, for directive @composite__derive: for associated @is directive: Product.user does not have a field named 'non_existent_field' or was present twice in the FieldSelectionMap
        23 |   id: ID!
        24 |   user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: "{ non_existent_field: id }")
                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        25 | }
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is", "@key", "@derive"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: ID
                    user: User! @derive @is(field: "{ id: author_id }")
                }

                type User @key(fields: "id") {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Product.user, for directive @composite__derive: for associated @is directive: Incompatible wrapping, cannot map Product.author_id (ID) into User.id (ID!)
        24 |   id: ID!
        25 |   user: User! @composite__derive(graph: EXT) @composite__is(graph: EXT, field: "{ id: author_id }")
                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        26 | }
        "#);
    })
}
