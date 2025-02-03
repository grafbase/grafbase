use crate::assert_solving_snapshots;

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

#[tokio::test]
async fn all_fields() {
    assert_solving_snapshots!(
        "all_fields",
        SCHEMA,
        r#"
        query {
          products {
            price
            category
            id
            name
          }
        }
        "#
    );
}

#[tokio::test]
async fn single_field() {
    assert_solving_snapshots!(
        "single_field",
        SCHEMA,
        r#"
        query {
          products {
            price
          }
        }
        "#
    );
}
