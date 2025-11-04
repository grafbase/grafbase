use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
    type Product
      @join__type(graph: A)
    {
      id: ID!
    }

    interface HasProducts @join__type(graph: A) {
        products: [Product!]!
    }

    type Query implements HasProducts
        @join__implements(graph: A, interface: "HasProducts")
        @join__type(graph: A)
    {
      products: [Product!]! @join__field(graph: A)
    }

    enum join__Graph
    {
        A @join__graph(name: "a", url: "http://localhost:8080")
    }
"#;

#[test]
fn interface_on_root_type() {
    assert_solving_snapshots!(
        "interface_on_root_type",
        SCHEMA,
        r#"
        query { ...F }
        fragment F on HasProducts { products { id } }
        "#
    );
}
