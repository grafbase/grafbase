use crate::{
    tests::{read_schema, TestOperation},
    OperationGraph,
};

const SCHEMA: &str = r###"
enum join__Graph {
  CATEGORY @join__graph(name: "category", url: "http://localhost:4200/shared-root/category")
  NAME @join__graph(name: "name", url: "http://localhost:4200/shared-root/name")
  PRICE @join__graph(name: "price", url: "http://localhost:4200/shared-root/price")
  REVIEW @join__graph(name: "review", url: "http://localhost:4200/shared-root/review")
}

type Product
  @join__type(graph: CATEGORY, key: "id")
  @join__type(graph: NAME, key: "id")
  @join__type(graph: PRICE, key: "id")
  @join__type(graph: REVIEW, key: "id")
{
  id: ID!
  reviews: [Review] @join__field(graph: REVIEW)
  category: String @join__field(graph: CATEGORY)
  name: String @join__field(graph: NAME)
  price: Float @join__field(graph: PRICE)
}

type Review @join__type(graph: REVIEW)
{
  stars: Int @join__field(graph: REVIEW)
}

type Query
  @join__type(graph: CATEGORY)
  @join__type(graph: NAME)
  @join__type(graph: PRICE)
{
  product: Product
  products: [Product]
}
"###;

#[test]
fn two_fields() {
    let schema = read_schema(SCHEMA);
    let mut operation = TestOperation::bind(
        &schema,
        r#"
        query {
          products {
            price
            category
          }
        }
    "#,
    );

    let mut graph = OperationGraph::new(&schema, &mut operation).unwrap();
    insta::assert_snapshot!("two_fields-graph", graph.to_dot_graph(), &graph.to_pretty_dot_graph());

    let mut solver = graph.solver().unwrap();
    insta::assert_snapshot!(
        "two_fields-solver",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );

    solver.solve().unwrap();
    insta::assert_snapshot!(
        "two_fields-solved",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );
}

#[test]
fn single_field() {
    let schema = read_schema(SCHEMA);
    let mut operation = TestOperation::bind(
        &schema,
        r#"
        query {
          products {
            price
          }
        }
    "#,
    );

    let mut graph = OperationGraph::new(&schema, &mut operation).unwrap();
    insta::assert_snapshot!("single_field-graph", graph.to_dot_graph(), &graph.to_pretty_dot_graph());

    let mut solver = graph.solver().unwrap();
    insta::assert_snapshot!(
        "single_field-solver",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );

    solver.solve().unwrap();
    insta::assert_snapshot!(
        "single_field-solved",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );
}

#[test]
fn nested_join() {
    let schema = read_schema(SCHEMA);
    let mut operation = TestOperation::bind(
        &schema,
        r#"
        query {
          products {
            reviews {
              stars
            }
          }
        }
    "#,
    );

    let mut graph = OperationGraph::new(&schema, &mut operation).unwrap();
    insta::assert_snapshot!("nested_join-graph", graph.to_dot_graph(), &graph.to_pretty_dot_graph());

    let mut solver = graph.solver().unwrap();
    insta::assert_snapshot!(
        "nested_join-solver",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );

    solver.solve().unwrap();
    insta::assert_snapshot!(
        "nested_join-solved",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );
}

#[test]
fn nested_join_with_name() {
    let schema = read_schema(SCHEMA);
    let mut operation = TestOperation::bind(
        &schema,
        r#"
        query {
          products {
            name
            reviews {
              stars
            }
          }
        }
    "#,
    );

    let mut graph = OperationGraph::new(&schema, &mut operation).unwrap();
    insta::assert_snapshot!(
        "nested_join_with_name-graph",
        graph.to_dot_graph(),
        &graph.to_pretty_dot_graph()
    );

    let mut solver = graph.solver().unwrap();
    insta::assert_snapshot!(
        "nested_join_with_name-solver",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );

    solver.solve().unwrap();
    insta::assert_snapshot!(
        "nested_join_with_name-solved",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );
}
