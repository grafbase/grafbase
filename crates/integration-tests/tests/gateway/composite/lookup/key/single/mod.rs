mod composite;
mod nested_key;
mod nested_output;
mod oneof;
mod oneof_composite;

use integration_tests::{gateway::Gateway, runtime};

use super::super::{EchoLookup, gql_id};

#[test]
fn arg_with_same_name() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(id: ID!): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "id": "1"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_type_compatibility_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(id: ID): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "id": "1"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_with_same_name_and_extra_optional_arg_with_matching_type() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(id: ID!, anything: ID): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "id": "1"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_with_different_name() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(productId: ID!): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "productId": "1"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_with_different_name_and_extra_optional_arg_with_matching_name() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(productId: ID!, id: Int): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "productId": "1"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_with_default_value() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(id: ID!, extra: Boolean! = true): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "id": "1",
                  "extra": true
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_with_default_value_coercion() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(id: ID!, extra: [Boolean!]! = true): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "id": "1",
                  "extra": [
                    true
                  ]
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn no_arguments() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup: Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.ProductLookup, for directive @lookup no matching @key directive was found
        See schema at 29:3:
        ProductLookup: Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn no_matching_argument() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(somethign: Int): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.ProductLookup, for directive @lookup no matching @key directive was found
        See schema at 29:3:
        ProductLookup(somethign: Int): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn cannot_inject_nullable_into_required() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(id: ID!): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.ProductLookup, for directive @lookup no matching @key directive was found
        See schema at 29:3:
        ProductLookup(id: ID!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn good_name_bad_type() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(id: Int): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.ProductLookup, for directive @lookup no matching @key directive was found
        See schema at 29:3:
        ProductLookup(id: Int): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn good_name_but_is_a_list() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(id: ID!): [Product!] @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.ProductLookup, for directive @lookup no matching @key directive was found
        See schema at 29:3:
        ProductLookup(id: ID!): [Product!] @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn ambiguous_multiple_matches() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(a: ID!, b: ID!): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.ProductLookup, for directive @lookup no matching @key directive was found
        See schema at 29:3:
        ProductLookup(a: ID!, b: ID!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn extra_required_argument() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    ProductLookup(id: ID!, required: Boolean!): Product! @lookup @echo
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.ProductLookup, for directive @lookup no matching @key directive was found
        See schema at 29:3:
        ProductLookup(id: ID!, required: Boolean!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
        "#);
    })
}
