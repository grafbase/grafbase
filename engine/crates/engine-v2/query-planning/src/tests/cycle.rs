use crate::{
    tests::{read_schema, TestOperation},
    OperationGraph,
};

const SCHEMA: &str = r###"
enum join__Graph {
  A @join__graph(name: "A", url: "http://localhost:4200/shared-root/category")
  B @join__graph(name: "B", url: "http://localhost:4200/shared-root/name")
}

type Cycle 
  @join__type(graph: A)
  @join__type(graph: B)
{
  first: String @join__field(graph: A, requires: "second")
  second: String @join__field(graph: B, requires: "second")
}

type Query
  @join__type(graph: A)
  @join__type(graph: B)
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
            first
            second
          }
        }
    "#,
    );

    let err = OperationGraph::new(&schema, &mut operation).unwrap_err();
    assert!(matches!(err, crate::Error::RequirementCycleDetected));
}
