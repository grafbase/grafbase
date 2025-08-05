use integration_tests::{gateway::Gateway, runtime};

use crate::gateway::extensions::{field_resolver::StaticFieldResolverExt, resolver::ResolverExt};

#[test]
fn field_resolver_extension_with_url_subgraph_returns_error() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_toml_config(
                r#"
                [subgraphs.products]
                url = "http://localhost:4001/graphql"
                "#,
            )
            .with_subgraph_sdl(
                "products",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@resolve"])

                type Query {
                    product(id: ID!): Product @resolve
                }

                type Product {
                    id: ID!
                    name: String!
                    price: Float!
                }
                "#,
            )
            .with_extension(StaticFieldResolverExt::json(
                r#"{"id": "1", "name": "Test Product", "price": 99.99}"#.into(),
            ))
            .try_build()
            .await;

        let error = result.unwrap_err();
        let error_message = error.to_string();

        insta::assert_snapshot!(error_message, @"Field resolver extensions can only be used with virtual subgraphs (subgraphs without a URL).");
    });
}

#[test]
fn resolver_extension_with_url_subgraph_returns_error() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_toml_config(
                r#"
                [subgraphs.products]
                url = "http://localhost:4001/graphql"
                "#,
            )
            .with_subgraph_sdl(
                "products",
                r#"
                extend schema
                    @link(url: "resolver-1.0.0", import: ["@resolve"])

                scalar JSON

                type Query {
                    product(id: ID!): JSON @resolve
                }
                "#,
            )
            .with_extension(ResolverExt::json(
                serde_json::json!({"id": "1", "name": "Test Product", "price": 99.99}),
            ))
            .try_build()
            .await;

        let error = result.unwrap_err();
        let error_message = error.to_string();

        insta::assert_snapshot!(error_message, @"Resolver extensions can only be used with virtual subgraphs (subgraphs without a URL).");
    });
}
