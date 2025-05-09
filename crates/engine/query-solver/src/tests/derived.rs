use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
    type Product
      @join__type(graph: EXT)
    {
      author_id: ID!
      code: String!
      id: ID!
      user: User! @composite__is(graph: EXT, field: "{ id: author_id }")
    }

    type User
      @join__type(graph: EXT)
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
async fn derived_fields() {
    assert_solving_snapshots!(
        "comments_fields_should_be_flattened",
        SCHEMA,
        r#"
        query {
            products {
                user {
                    id
                }
            }
        }
        "#
    );
}
