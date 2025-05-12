use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
    type Product
      @join__type(graph: EXT)
    {
      authorId: ID!
      code: String!
      id: ID!
      author: User! @composite__derive(graph: EXT)
    }

    type User
        @join__type(graph: EXT, key: "id", resolvable: false)
    {
      id: ID!
    }

    type Query
    {
      products: [Product!]! @join__field(graph: EXT)
    }

    enum join__Graph
    {
    EXT @join__graph(name: "ext", url: "http://localhost:8080")
    }
"#;

#[tokio::test]
async fn simple_derive_field() {
    assert_solving_snapshots!(
        "simple_derive_field",
        SCHEMA,
        r#"
        query {
            products {
                author {
                    id
                }
            }
        }
        "#
    );
}
