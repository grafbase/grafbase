use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn single_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
            extend schema
                @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key", "@is"])

            type Query {
                post: Post!
            }

            type Post {
                id: ID!
                authId: ID!
                author: User! @derive @is(field: "{ id: authId }")
            }

            type User @key(fields: "id") {
                id: ID!
            }
            "#,
                )
                .with_resolver("Query", "post", json!({"id": "post_1", "authId": "user_1"}))
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine.post("query { post { id author { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": {
                "id": "user_1"
              }
            }
          }
        }
        "#);
    })
}

#[test]
fn composite_keys() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key", "@is"])

                type Query {
                    post: Post!
                }

                type Post {
                    id: ID!
                    a: ID!
                    b: ID!
                    author: User! @derive @is(field: "{ idA: a idB: b }")
                }

                type User @key(fields: "idA idB") {
                    idA: ID!
                    idB: ID!
                }
                "#,
                )
                .with_resolver("Query", "post", json!({"id": "post_1", "a": "user_a", "b": "user_b"}))
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine.post("query { post { id author { idA idB } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": {
                "idA": "user_a",
                "idB": "user_b"
              }
            }
          }
        }
        "#);
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
        At site Product.user, for directive @composite__derive: for associated @is directive: Cannot map Product.author_id (Int!) into User.id (ID!)
        See schema at 25:60:
        (graph: EXT, field: "{ id: author_id }")
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
        At site Product.user, for directive @composite__derive: for associated @is directive: Incompatible wrapping, cannot map Product.author_ids ([ID!]!) into User.id (ID!)
        See schema at 25:60:
        (graph: EXT, field: "{ id: author_ids }")
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
        At site Product.user, for directive @composite__derive: for associated @is directive: 
        {[]}
         ^
        invalid object

        See schema at 25:60:
        (graph: EXT, field: "{[]}")
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__derive: for associated @is directive: missing field: field
        See schema at 7:60:
        (graph: EXT)
        "#);
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
        At site Product.user, for directive @composite__derive: for associated @is directive: missing field: graph
        See schema at 7:60:
        (field: "{ id: author_id }")
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.user, for directive @composite__derive: for associated @is directive: found a Int where we expected a String
        See schema at 7:60:
        (graph: EXT, field: 0)
        "#);
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
        At site Product.user, for directive @composite__derive: for associated @is directive: unknown field: yes
        See schema at 7:60:
        (graph: EXT, field: "{ id: author_id }", yes: true)
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
        At site Product.user, for directive @composite__derive: for associated @is directive: Type Product does not have a field named 'non_existent_field'
        See schema at 24:60:
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
        At site Product.user, for directive @composite__derive: for associated @is directive: Field 'non_existent_field' does not exist on Product.user
        See schema at 24:60:
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
        At site Product.user, for directive @composite__derive: for associated @is directive: Incompatible wrapping, cannot map Product.author_id (ID) into User.id (ID!)
        See schema at 25:60:
        (graph: EXT, field: "{ id: author_id }")
        "#);
    })
}
