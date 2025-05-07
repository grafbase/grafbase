use integration_tests::{gateway::Gateway, runtime};

#[ignore] // doens't work yet with @lookup validation.
#[test]
fn basic() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key"])

                type Query {
                    productBatch(input: ProductInput! @is(field: "{ id }")): Product! @lookup
                }

                input ProductInput {
                    id: ID!
                }

                type Product @key(fields: "id") {
                    id: ID!
                    code: String!
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
