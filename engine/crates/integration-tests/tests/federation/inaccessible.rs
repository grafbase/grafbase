use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn inaccessible_arguments_are_inaccessible() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_federated_sdl(
                r#"
            directive @join__field(graph: join__Graph, requires: String, provides: String) on FIELD_DEFINITION
            directive @join__graph(name: String!, url: String!) on ENUM_VALUE

            enum join__Graph {
              PRODUCTS @join__graph(name: "products", url: "http://127.0.0.1:46697")
            }

            type Query {
              topProducts(one: Int, two: Boolean @inaccessible): [Int!] @join__field(graph: PRODUCTS)
            }
        "#,
            )
            .build()
            .await;

        engine
            .post(
                r#"query {
                    topProducts(one: 1)
                    topProductsWithInaccessibleArg: topProducts(one: 3, two: true)
                }"#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "errors": [
        {
          "message": "The field `Query.topProducts` does not have an argument named `two",
          "locations": [
            {
              "line": 3,
              "column": 21
            }
          ],
          "extensions": {
            "code": "OPERATION_VALIDATION_ERROR"
          }
        }
      ]
    }
    "#);
}
