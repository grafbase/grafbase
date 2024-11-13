use crate::assert_solving_snapshots;

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
fn nested_join() {
    assert_solving_snapshots!(
        "nested_join",
        SCHEMA,
        r#"
        query {
          products {
            reviews {
              stars
            }
          }
        }
        "#
    );
}

#[test]
fn nested_join_with_name() {
    assert_solving_snapshots!(
        "nested_join_with_name",
        SCHEMA,
        r#"
        query {
          products {
            name
            reviews {
              stars
            }
          }
        }
        "#
    );
}
