use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
enum join__Graph {
    CATEGORY @join__graph(name: "category", url: "http://example.com/category")
    PRODUCTS @join__graph(name: "products", url: "http://example.com/products")
    SUBCATEGORIES @join__graph(name: "subcategories", url: "http://example.com/subcategories")
}

type Product
    @join__type(graph: CATEGORY, key: "id")
    @join__type(graph: PRODUCTS, key: "id")
{
    categories: [Category] @join__field(graph: CATEGORY)
    id: ID!
}

type Category
    @join__type(graph: CATEGORY, key: "id")
    @join__type(graph: SUBCATEGORIES, key: "id")
{
    id: ID!
    """
    This field is provided by Query.products
    and we deliberately don't resolve it.
    The test suite is about checking if @provides is used correctly.
    """
    name: String @join__field(graph: CATEGORY)
    kind: String @join__field(graph: CATEGORY)
    subCategories: [Category] @join__field(graph: SUBCATEGORIES, provides: "kind")
}

type Query {
    products: [Product] @join__field(graph: PRODUCTS, provides: "categories { id name subCategories { id name } }")
}
"#;

#[test]
fn provides_full() {
    assert_solving_snapshots!(
        "provides_full",
        SCHEMA,
        r#"
        query {
          products {
            id
            categories {
              id
              name
              subCategories {
                id
                name
              }
            }
          }
        }
        "#
    );
}

#[test]
fn provides_with_entity_join() {
    assert_solving_snapshots!(
        "provide_with_entity_join",
        SCHEMA,
        r#"
        query {
          products {
            id
            categories {
              id
              name
              kind
              subCategories {
                id
                name
                kind
              }
            }
          }
        }
        "#
    );
}
