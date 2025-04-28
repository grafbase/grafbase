mod key;

use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use crate::gateway::extensions::selection_set_resolver::StaticSelectionSetResolverExt;

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                            products: [Product!]!
                        }

                        type Product @key(fields: "id") {
                            id: ID!
                        }
                    "#,
                )
                .with_resolver("Query", "products", json!([{"id": "1"}, {"id": "2"}]))
                .into_subgraph("gql"),
            )
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schema/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(ids: [ID!]!): [Product!]! @lookup
                }

                type Product @key(fields: "id") {
                    id: ID!
                    code: String!
                }
                "#,
            )
            .with_extension(StaticSelectionSetResolverExt::json(
                json!([{"code": "C1"}, {"code": "C2"}]),
            ))
            .build()
            .await;

        let response = engine.post("query { products { id code } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "code": "C1"
              },
              {
                "id": "2",
                "code": "C2"
              }
            ]
          }
        }
        "#);
    })
}
