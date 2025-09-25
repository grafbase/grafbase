mod error;
mod selection;

use integration_tests::{gateway::Gateway, runtime};

use super::{Resolve, gql_product};

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_product())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@require", "@key"])

                type Product @key(fields: "id") {
                    id: ID!
                    dummy(id: ID! @require(field: "id")): JSON @resolve
                }

                scalar JSON
                "#,
            )
            .with_extension(Resolve::with(Ok))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "dummy": {
                  "id": "1"
                }
              },
              {
                "id": "2",
                "dummy": {
                  "id": "2"
                }
              }
            ]
          }
        }
        "#);
    })
}
