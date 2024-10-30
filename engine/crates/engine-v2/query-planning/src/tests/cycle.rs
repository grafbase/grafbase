use crate::{
    tests::{read_schema, TestOperation},
    OperationGraph,
};

const SCHEMA: &str = r###"
enum join__Graph {
  A @join__graph(name: "A", url: "http://localhost:4200/shared-root/category")
  B @join__graph(name: "B", url: "http://localhost:4200/shared-root/name")
  C @join__graph(name: "C", url: "http://localhost:4200/shared-root/c")
}

type Cycle 
  @join__type(graph: A, key: "id")
  @join__type(graph: B, key: "id")
  @join__type(graph: C, key: "id")
{
  id: ID!
  first: String @join__field(graph: A, requires: "second")
  second: String @join__field(graph: B, requires: "first")
  dummy: String @join__field(graph: B, requires: "first")
}

type Query
  @join__type(graph: C)
{
  cycle: Cycle 
}
"###;

#[test]
fn cycle() {
    let schema = read_schema(SCHEMA);
    let mut operation = TestOperation::bind(
        &schema,
        r#"
        query {
          cycle {
            dummy
          }
        }
    "#,
    );

    let mut graph = OperationGraph::new(&schema, &mut operation).unwrap();
    insta::assert_snapshot!("graph", graph.to_dot_graph(), &graph.to_pretty_dot_graph());

    let err = graph.solver().unwrap_err();
    assert!(matches!(err, crate::Error::RequirementCycleDetected));
}
