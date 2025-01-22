use integration_tests::federation::DeterministicEngine;
use serde_json::json;

#[ignore]
#[test]
fn subgraph_error() {
    let response = integration_tests::runtime().block_on(async {
        DeterministicEngine::new(
            r#"
            enum join__Graph {
              SUB1 @join__graph(name: "sub1", url: "http://localhost:4000/graphql")
              SUB2 @join__graph(name: "sub2", url: "http://localhost:4000/graphql")
            }

            type Query @join__type(graph: SUB1) @join__type(graph: SUB2) {
              node: Node
            }

            type Node @join__type(graph: SUB1) @join__type(graph: SUB2) {
              f1: String @join__field(graph: SUB1)
              f2: String @join__field(graph: SUB2)
            }
            "#,
            r#"
            query {
                node {
                    f1
                    f2
                }
            }
            "#,
            &[json!({"data":{"node": {"f1": "1", "f2": "2"}}}), json!(null)],
        )
        .await
        .execute()
        .await
    });
    // Broken today, we return `node: null` instead of something like `node: { f1: "1", f2: null }`
    insta::assert_json_snapshot!(response, @r#"
    "#);
}
