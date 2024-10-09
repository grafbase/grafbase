use crate::{
    tests::{read_schema, TestOperation},
    OperationGraph,
};

const SCHEMA: &str = r###"
enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts", url: "http://accounts:4001/graphql")
  INVENTORY @join__graph(name: "inventory", url: "http://inventory:4002/graphql")
  PRODUCTS @join__graph(name: "products", url: "http://products:4003/graphql")
  REVIEWS @join__graph(name: "reviews", url: "http://reviews:4004/graphql")
}

type Product
  @join__type(graph: INVENTORY, key: "upc")
  @join__type(graph: PRODUCTS, key: "upc")
  @join__type(graph: REVIEWS, key: "upc")
{
  upc: String!
  weight: Int @join__field(graph: INVENTORY, external: true) @join__field(graph: PRODUCTS)
  price: Int @join__field(graph: INVENTORY, external: true) @join__field(graph: PRODUCTS)
  inStock: Boolean @join__field(graph: INVENTORY)
  shippingEstimate: Int @join__field(graph: INVENTORY, requires: "price weight")
  name: String @join__field(graph: PRODUCTS)
  reviews: [Review] @join__field(graph: REVIEWS)
}

type Query
  @join__type(graph: ACCOUNTS)
  @join__type(graph: INVENTORY)
  @join__type(graph: PRODUCTS)
  @join__type(graph: REVIEWS)
{
  me: User @join__field(graph: ACCOUNTS)
  user(id: ID!): User @join__field(graph: ACCOUNTS)
  users: [User] @join__field(graph: ACCOUNTS)
  topProducts(first: Int = 5): [Product] @join__field(graph: PRODUCTS)
}

type Review
  @join__type(graph: REVIEWS, key: "id")
{
  id: ID!
  body: String
  product: Product
  author: User @join__field(graph: REVIEWS, provides: "username")
}

type User
  @join__type(graph: ACCOUNTS, key: "id")
  @join__type(graph: REVIEWS, key: "id")
{
  id: ID!
  name: String @join__field(graph: ACCOUNTS)
  username: String @join__field(graph: ACCOUNTS) @join__field(graph: REVIEWS, external: true)
  birthday: Int @join__field(graph: ACCOUNTS)
  reviews: [Review] @join__field(graph: REVIEWS)
}
"###;

#[test]
fn test_basic_operation_graph() {
    let schema = read_schema(SCHEMA);
    let mut operation = TestOperation::bind(
        &schema,
        r#"
    {
        topProducts {
            name
            reviews {
                author {
                    name
                }
            }
        }
    }
    "#,
    );

    let mut graph = OperationGraph::new(&schema, &mut operation);
    insta::assert_snapshot!("graph", graph.to_dot_graph(), &graph.to_prettty_dot_graph());

    graph.estimate_resolver_costs();
    insta::assert_snapshot!("cost", graph.to_dot_graph(), &graph.to_prettty_dot_graph());
}
