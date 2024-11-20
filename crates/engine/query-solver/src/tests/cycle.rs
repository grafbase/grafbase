use crate::{
    assert_solving_snapshots,
    solve::build_solver_with_shortest_path_algorithm,
    tests::{read_schema, TestOperation},
    OperationGraph,
};

const SCHEMA: &str = r###"
enum join__Graph {
  A @join__graph(name: "A", url: "http://localhost:4200/shared-root/a")
  B @join__graph(name: "B", url: "http://localhost:4200/shared-root/b")
  C @join__graph(name: "C", url: "http://localhost:4200/shared-root/c")
}

type RequirementsCycle
  @join__type(graph: A, key: "id")
  @join__type(graph: B, key: "id")
  @join__type(graph: C, key: "id")
{
  id: ID!
  first: String @join__field(graph: A, requires: "second")
  second: String @join__field(graph: B, requires: "first")
  bootstrap: String @join__field(graph: B, requires: "first")
}

type PartitionsCycle
  @join__type(graph: A, key: "id")
  @join__type(graph: B, key: "id")
  @join__type(graph: C, key: "id")
{
  id: ID!
  first: String @join__field(graph: A)
  second: String @join__field(graph: B, requires: "first")
  third: String @join__field(graph: A, requires: "second")
}


type Query
  @join__type(graph: C)
{
  requirementsCycle: RequirementsCycle
partitionsCycle: PartitionsCycle
}
"###;

#[test]
fn requirements_cycle() {
    let schema = read_schema(SCHEMA);
    let mut operation = TestOperation::bind(
        &schema,
        r#"
        query {
          requirementsCycle {
            bootstrap
          }
        }
    "#,
    );

    let graph = OperationGraph::new(&schema, &mut operation).unwrap();
    insta::assert_snapshot!(
        "requirements_cycle-graph",
        graph.to_dot_graph(),
        &graph.to_pretty_dot_graph()
    );

    let Err(err) = build_solver_with_shortest_path_algorithm(&graph) else {
        unreachable!("expected error");
    };
    assert!(matches!(err, crate::Error::RequirementCycleDetected));
}

#[test]
fn query_partitions_cycle() {
    // 'first' and 'third' cannot be in the same query partitions as it would lead to a cyclic
    // dependency between query partitions.
    assert_solving_snapshots!(
        "query_partitions_cycle",
        SCHEMA,
        r#"
        query {
          partitionsCycle {
            third
            first
            second
          }
        }
        "#
    );
}
