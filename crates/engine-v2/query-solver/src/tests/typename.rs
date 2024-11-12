use crate::assert_solving_snapshots;

const SCHEMA: &str = r###"
enum join__Graph {
  PRODUCTS @join__graph(name: "products", url: "http://products:4003/graphql")
}

type Product
{
  upc: String!
}

type Query
  @join__type(graph: PRODUCTS)
{
  topProducts(first: Int = 5): [Product] @join__field(graph: PRODUCTS)
}
"###;

#[test]
fn top_level_typename() {
    assert_solving_snapshots!(
        "top_level_typename",
        SCHEMA,
        r#"
        query {
            __typename
        }
        "#
    );
}

#[test]
fn only_typename() {
    assert_solving_snapshots!(
        "only_typename",
        SCHEMA,
        r#"
        {
          topProducts {
            __typename
          }
        }
        "#
    );
}
