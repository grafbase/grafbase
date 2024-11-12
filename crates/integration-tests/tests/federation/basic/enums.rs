use engine_v2::Engine;
use graphql_mocks::TeaShop;
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn inaccessible_enum_values_are_inaccessible() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(TeaShop::default()).build().await;

        engine
            .post("query { recommendedTeas { id name style } teaWithInaccessibleStyle { name style } }")
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "recommendedTeas": [
          {
            "id": 0,
            "name": "Earl Grey",
            "style": "BLACK"
          },
          {
            "id": 7,
            "name": "Tuóchá",
            "style": null
          }
        ],
        "teaWithInaccessibleStyle": null
      }
    }
    "#);
}
