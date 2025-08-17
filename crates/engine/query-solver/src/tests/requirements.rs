use crate::{assert_solving_snapshots, tests::WithExtensions};

const SCHEMA: &str = r#"
type Product
  @join__type(graph: EXT, key: "id", resolvable: false)
  @join__type(graph: EXT, key: "name", resolvable: false)
  @join__type(graph: GQL, key: "id")
  @join__type(graph: GQL2, key: "name")
{
  args: JSON @join__field(graph: EXT)
  id: ID! @join__field(graph: EXT) @join__field(graph: GQL)
  name: String! @join__field(graph: EXT) @join__field(graph: GQL2)
}

type Query
{
  productBatch(input: Lookup! @composite__is(graph: EXT, field: "{ ids: [id] } | { names: [name] }")): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
  products: [Product!]! @join__field(graph: GQL)
  products2: [Product!]! @join__field(graph: GQL2)
}

enum join__Graph
{
  EXT @join__graph(name: "ext")
  GQL @join__graph(name: "gql", url: "http://127.0.0.1:33029/")
  GQL2 @join__graph(name: "gql2", url: "http://127.0.0.1:42167/")
}

enum extension__Link
{
  ECHO @extension__link(url: "file:///echo")
}

input Lookup
  @oneOf
  @join__type(graph: EXT)
{
  ids: [ID!]
  names: [String!]
}

scalar JSON
"#;

#[test]
fn intermediate_resolver() {
    let schema = WithExtensions::new(SCHEMA).resolver("file:///echo", r#"directive @echo on FIELD_DEFINITION"#);
    assert_solving_snapshots!(
        "intermediate_resolver",
        schema,
        r#"
        query { products { id args } }
        "#
    );
}
