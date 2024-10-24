use crate::{
    tests::{read_schema, strdiff, TestOperation},
    OperationGraph,
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

    let mut graph = OperationGraph::new(&schema, &mut operation);
    insta::assert_snapshot!("all_fields-graph", graph.to_dot_graph(), &graph.to_pretty_dot_graph());

    let before = graph.to_dot_graph();

    graph.prune_resolvers_not_leading_to_any_scalar_node();
    insta::assert_snapshot!(strdiff(&before, &graph.to_dot_graph()), @"");
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

    let mut graph = OperationGraph::new(&schema, &mut operation);
    insta::assert_snapshot!("single_field-graph", graph.to_dot_graph(), &graph.to_pretty_dot_graph());

    let before = graph.to_dot_graph();

    graph.prune_resolvers_not_leading_to_any_scalar_node();
    insta::assert_snapshot!(strdiff(&before, &graph.to_dot_graph()), @r##"
    -    3 [ Root#category]
    -    4 [ products@Root#category]
    -    5 [ Root#name]
    -    6 [ products@Root#name]
    -    0 -> 3 [ label = "CreateChildResolver(1)" ]
    -    0 -> 3 [ label = "HasChildResolver" ]
    -    3 -> 4 [ label = "CanProvide(0)" ]
    -    4 -> 2 [ label = "Provides" ]
    -    0 -> 5 [ label = "CreateChildResolver(1)" ]
    -    0 -> 5 [ label = "HasChildResolver" ]
    -    5 -> 6 [ label = "CanProvide(0)" ]
    -    6 -> 2 [ label = "Provides" ]
    "##);
}
