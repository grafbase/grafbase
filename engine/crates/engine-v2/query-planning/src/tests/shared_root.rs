use crate::{
    tests::{read_schema, TestOperation},
    OperationGraph, Solver,
};

const SCHEMA: &str = r###"
enum join__Graph {
  CATEGORY @join__graph(name: "category", url: "http://localhost:4200/shared-root/category")
  NAME @join__graph(name: "name", url: "http://localhost:4200/shared-root/name")
  PRICE @join__graph(name: "price", url: "http://localhost:4200/shared-root/price")
}

type Product
  @join__type(graph: CATEGORY)
  @join__type(graph: NAME)
  @join__type(graph: PRICE)
{
  id: ID!
  category: String @join__field(graph: CATEGORY)
  name: String @join__field(graph: NAME)
  price: Float @join__field(graph: PRICE)
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
fn all_fields() {
    let schema = read_schema(SCHEMA);
    let mut operation = TestOperation::bind(
        &schema,
        r#"
        query {
          products {
            price
            category
            id
            name
          }
        }
    "#,
    );

    let mut graph = OperationGraph::new(&schema, &mut operation).unwrap();
    insta::assert_snapshot!("all_fields-graph", graph.to_dot_graph(), &graph.to_pretty_dot_graph());

    let mut solver = Solver::initialize(&graph).unwrap();
    insta::assert_snapshot!(
        "all_fields-solver",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );

    solver.execute().unwrap();
    insta::assert_snapshot!(
        "all_fields-solved",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );

    graph.solve().unwrap();
    insta::assert_snapshot!(
        "all_fields-solved-graph",
        graph.to_dot_graph(),
        &graph.to_pretty_dot_graph()
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

    let mut solver = Solver::initialize(&graph).unwrap();
    insta::assert_snapshot!(
        "single_field-solver",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );

    solver.execute().unwrap();
    insta::assert_snapshot!(
        "single_field-solved",
        solver.to_dot_graph(),
        &solver.to_pretty_dot_graph()
    );

    graph.solve().unwrap();
    insta::assert_snapshot!(
        "single_field-solved-graph",
        graph.to_dot_graph(),
        &graph.to_pretty_dot_graph()
    );
}
