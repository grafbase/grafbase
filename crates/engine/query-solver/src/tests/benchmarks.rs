use crate::assert_solution_snapshots;

const SCHEMA: &str = r#"
enum join__Graph {
  SUB0 @join__graph(name: "sub0", url: "http://localhost:7000/graphql")
  SUB1 @join__graph(name: "sub1", url: "http://localhost:7000/graphql")
  SUB2 @join__graph(name: "sub2", url: "http://localhost:7000/graphql")
  SUB3 @join__graph(name: "sub3", url: "http://localhost:7000/graphql")
}

type Query @join__type(graph: SUB0) @join__type(graph: SUB1) @join__type(graph: SUB2) @join__type(graph: SUB3) {
  node: Node
}

type Node
  @join__type(graph: SUB0, key: "id0")
  @join__type(graph: SUB1, key: "id0")
  @join__type(graph: SUB2, key: "id0")
  @join__type(graph: SUB3, key: "id0") {
  id0: ID!
  n0: Node @join__field(graph: SUB0) @join__field(graph: SUB2) @join__field(graph: SUB3)
  f0: String @join__field(graph: SUB0) @join__field(graph: SUB2) @join__field(graph: SUB3)
  n1: Node @join__field(graph: SUB1) @join__field(graph: SUB2) @join__field(graph: SUB3)
  f1: String @join__field(graph: SUB1) @join__field(graph: SUB2) @join__field(graph: SUB3)
  n2: Node @join__field(graph: SUB0) @join__field(graph: SUB1) @join__field(graph: SUB3)
  f2: String @join__field(graph: SUB0) @join__field(graph: SUB1) @join__field(graph: SUB3)
  n3: Node @join__field(graph: SUB0) @join__field(graph: SUB1) @join__field(graph: SUB2)
  f3: String @join__field(graph: SUB0) @join__field(graph: SUB1) @join__field(graph: SUB2)
}
"#;

#[test]
fn alternating_subgraphs_1() {
    assert_solution_snapshots!(
        "alternating_subgraphs_1",
        SCHEMA,
        r#"
        query {
          node {
            n1 {
              n0 { n1 { f0 f1 f2 } n2 { f0 f1 f3 } n3 { f0 f2 f3 } }
              n2 { n1 { f0 f1 f2 } n2 { f0 f1 f3 } n3 { f0 f2 f3 } }
            }
            n2 {
              n1 { n0 { f0 f1 f3 } n1 { f0 f2 f3 } n2 { f1 f2 f3 } }
              n2 { n0 { f0 f1 f3 } n1 { f0 f2 f3 } n2 { f1 f2 f3 } }
            }
          }
        }
        "#
    );
}
