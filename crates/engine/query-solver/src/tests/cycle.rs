use operation::{Operation, OperationContext};
use schema::Schema;

use crate::{assert_solving_snapshots, solve::Solver, Query};

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

type Post
  @join__type(graph: C, key: "id")
  @join__type(graph: A, key: "id")
{
  id: ID!
  author: Author @join__field(graph: A, requires: "comments(limit: 3) { authorId }")
  comments(limit: Int): [Comment] @join__field(graph: A)
}

type Author
    @join__type(graph: A)
{
    id: ID!
    name: String
}

type Comment
    @join__type(graph: C, key: "id")
    @join__type(graph: A, key: "id")
{
    id: ID!
    authorId: ID @join__field(graph: C)
}

type Query
  @join__type(graph: C)
{
  requirementsCycle: RequirementsCycle
  partitionsCycle: PartitionsCycle
  feed: [Post]
}
"###;

#[tokio::test]
async fn requirements_cycle() {
    let schema = Schema::from_sdl_or_panic(SCHEMA).await;
    let operation = Operation::parse(
        &schema,
        None,
        r#"
        query {
          requirementsCycle {
            bootstrap
          }
        }
    "#,
    )
    .unwrap();

    let query = Query::generate_solution_space(&schema, &operation).unwrap();
    let ctx = OperationContext {
        schema: &schema,
        operation: &operation,
    };
    insta::assert_snapshot!(
        "requirements_cycle-graph",
        query.to_dot_graph(ctx),
        &query.to_pretty_dot_graph(ctx)
    );

    let err = Solver::initialize(&schema, &operation, &query).unwrap_err();
    assert!(matches!(err, crate::Error::RequirementCycleDetected));
}

#[tokio::test]
async fn query_partitions_cycle() {
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

#[tokio::test]
async fn query_partitions_nested_cycle_1() {
    // 'first' and 'third' cannot be in the same query partitions as it would lead to a cyclic
    // dependency between query partitions.
    assert_solving_snapshots!(
        "query_partitions_nested_cycle",
        SCHEMA,
        r#"
        query {
          feed {
            author {
              id
            }
            comments(limit: 3) {
              id
            }
          }
        }
        "#
    );
}

// As we use a direct graph, ordering of edges matter. This query will process fields in the
// reverse order ensuring we handle the directed edge correctly.
#[tokio::test]
async fn query_partitions_nested_cycle_2() {
    // 'first' and 'third' cannot be in the same query partitions as it would lead to a cyclic
    // dependency between query partitions.
    assert_solving_snapshots!(
        "query_partitions_nested_cycle2",
        SCHEMA,
        r#"
        query {
          feed {
            comments(limit: 3) {
              id
            }
            author {
              id
            }
          }
        }
        "#
    );
}
