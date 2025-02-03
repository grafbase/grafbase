use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
enum join__Graph {
    A @join__graph(name: "a", url: "http://localhost:4200/requires-with-argument/a")
    B @join__graph(name: "b", url: "http://localhost:4200/requires-with-argument/b")
    C @join__graph(name: "c", url: "http://localhost:4200/requires-with-argument/c")
    D @join__graph(name: "d", url: "http://localhost:4200/requires-with-argument/d")
}

type Author implements Node
    @join__type(graph: D)
{
    id: ID!
    name: String
}

type Comment implements Node
    @join__type(graph: C, key: "id")
    @join__type(graph: D, key: "id")
{
    id: ID!
    authorId: ID @join__field(graph: C)
    body: String! @join__field(graph: C)
    date: String @join__field(graph: D)
}

type Post implements Node
    @join__type(graph: C, key: "id")
    @join__type(graph: D, key: "id")
{
    id: ID!
    author: Author @join__field(graph: D, requires: "comments(limit: 3) { authorId }")
    comments(limit: Int!): [Comment] @join__field(graph: D)
}

type Product
    @join__type(graph: A, key: "upc")
    @join__type(graph: B, key: "upc")
{
    upc: String!
    weight: Int @join__field(graph: B)
    price(currency: String!): Int @join__field(graph: B)
    shippingEstimate: Int @join__field(graph: A, requires: "price(currency: \"USD\") weight")
    name: String @join__field(graph: B)
}

interface Node {
    id: ID!
}

type Query
    @join__type(graph: A)
    @join__type(graph: B)
    @join__type(graph: C)
    @join__type(graph: D)
{
    node: Node @join__field(graph: D)
    products: [Product] @join__field(graph: B)
    feed: [Post] @join__field(graph: C)
}
"#;

#[tokio::test]
async fn comments_fields_should_be_flattened() {
    assert_solving_snapshots!(
        "comments_fields_should_be_flattened",
        SCHEMA,
        r#"
        query ($limit: Int = 1) {
          feed {
            author {
              id
            }
            ...Foo
            ...Bar
          }
        }

        fragment Foo on Post {
          comments(limit: $limit) {
            id
          }
        }

        fragment Bar on Post {
          comments(limit: $limit) {
            id
          }
        }
        "#
    );
}

#[tokio::test]
async fn flattening_cannot_merge_fields_with_different_type_conditions() {
    assert_solving_snapshots!(
        "flattening_cannot_merge_fields_with_different_type_conditions",
        SCHEMA,
        r#"
        query {
          node {
            ... on Post {
              ... on Node {
                id
              }
            }
            ... on Comment {
              ... on Node {
                id
              }
            }
          }
        }
        "#
    );
}
