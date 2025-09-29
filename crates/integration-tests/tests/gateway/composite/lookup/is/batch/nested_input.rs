use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoLookup, gql_id};

#[test]
fn nested_input() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productLookup(input: [LookupInput!]! @is(field: "{ ids: [id] }")): [Product!]! @lookup @echo
                }

                input LookupInput {
                    ids: [ID!]!
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::batch())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "input": {
                    "ids": [
                      "1"
                    ]
                  }
                }
              }
            ]
          }
        }
        "#);
    })
}
