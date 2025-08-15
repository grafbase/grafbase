use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
enum join__Graph {
  SUB0 @join__graph(name: "sub0", url: "http://localhost:1000")
  SUB1 @join__graph(name: "sub1", url: "http://localhost:1001")
  SUB2 @join__graph(name: "sub2", url: "http://localhost:1002")
}

type Query @join__type(graph: SUB0) @join__type(graph: SUB1) @join__type(graph: SUB2) {
    node: Node
}
type Node @join__type(graph: SUB0, key: "id0") @join__type(graph: SUB1, key: "id0") @join__type(graph: SUB2, key: "id0") {
    id0: ID!
    n0: Node @join__field(graph: SUB1) @join__field(graph: SUB2)
    f0: String @join__field(graph: SUB1) @join__field(graph: SUB2)
    n1: Node @join__field(graph: SUB0) @join__field(graph: SUB2)
    f1: String @join__field(graph: SUB0) @join__field(graph: SUB2)
    n2: Node @join__field(graph: SUB0) @join__field(graph: SUB1)
    f2: String @join__field(graph: SUB0) @join__field(graph: SUB1)
}
"#;

#[tokio::test]
async fn alternating_subgraphs_1() {
    assert_solving_snapshots!(
        "alternating_subgraphs_1",
        SCHEMA,
        r#"
        query {
          node {
            n0 {
              n0 {
                f0
                f1
                f2
              }
              n2 {
                f0
                f1
                f2
              }
            }
          }
        }
        "#
    );
}
