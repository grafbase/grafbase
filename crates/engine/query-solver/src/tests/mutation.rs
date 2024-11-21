use crate::assert_solving_snapshots;

const SCHEMA: &str = r###"
enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts", url: "http://accounts:4001/graphql")
  PRODUCTS @join__graph(name: "products", url: "http://products:4003/graphql")
  REVIEWS @join__graph(name: "reviews", url: "http://reviews:4004/graphql")
  A @join__graph(name: "a", url: "http://accounts:4001/a")
  B @join__graph(name: "b", url: "http://products:4003/b")
  C @join__graph(name: "c", url: "http://reviews:4004/c")
}

type Product
  @join__type(graph: PRODUCTS, key: "upc")
  @join__type(graph: REVIEWS, key: "upc")
{
  upc: String!
}

type Mutation 
  @join__type(graph: ACCOUNTS)
  @join__type(graph: PRODUCTS)
  @join__type(graph: REVIEWS)
  @join__type(graph: A)
  @join__type(graph: B)
  @join__type(graph: C)
{
  createUser: User @join__field(graph: ACCOUNTS)
  updateUser: User @join__field(graph: ACCOUNTS)
  createProduct: Product @join__field(graph: PRODUCTS)
  updateProduct: Product @join__field(graph: PRODUCTS)
  createReview: Review @join__field(graph: REVIEWS)
  updateReview: Review @join__field(graph: REVIEWS)

  multiply(by: Int!, requestId: String!): Int! @join__field(graph: A)
  delete(requestId: String!): Int! @join__field(graph: B)
  add(num: Int!, requestId: String!): Int! @join__field(graph: C)
}

type Query @join__type(graph: ACCOUNTS) {
  me: User @join__field(graph: ACCOUNTS)
}

type Review
  @join__type(graph: REVIEWS, key: "id")
{
  id: ID!
}

type User
  @join__type(graph: ACCOUNTS, key: "id")
  @join__type(graph: REVIEWS, key: "id")
{
  id: ID!
}
"###;

#[test]
fn single_subgraph() {
    assert_solving_snapshots!(
        "single_subgraph",
        SCHEMA,
        r#"
        mutation {
          createUser { id }
          updateUser { id }
        }
        "#
    );
}

#[test]
fn consecutive_subgraphs() {
    assert_solving_snapshots!(
        "consecutive_subgraphs",
        SCHEMA,
        r#"
        mutation {
          createUser { id }
          createProduct { upc }
          createReview { id }
        }
        "#
    );
}

#[test]
fn consecutive_subgraphs_with_multiple_fields() {
    assert_solving_snapshots!(
        "consecutive_subgraphs_with_multiple_fields",
        SCHEMA,
        r#"
        mutation {
          createUser { id }
          updateUser { id }
          createProduct { upc }
          updateProduct { upc }
          createReview { id }
          updateReview { id }
        }
        "#
    );
}

#[test]
fn interleaved_subgraph_fields() {
    assert_solving_snapshots!(
        "interleaved_subgraph_fields",
        SCHEMA,
        r#"
        mutation {
          createUser { id }
          createProduct { upc }
          createReview { id }
          updateProduct { upc }
          updateUser { id }
          updateReview { id }
        }
        "#
    );
}

#[test]
fn audit_test() {
    assert_solving_snapshots!(
        "audit_test",
        SCHEMA,
        r#"
        mutation {
          five: add(num: 5, requestId: "${randomId}")
          ten: multiply(by: 2, requestId: "${randomId}")
          twelve: add(num: 2, requestId: "${randomId}")
          final: delete(requestId: "${randomId}")
        }
        "#
    );
}
