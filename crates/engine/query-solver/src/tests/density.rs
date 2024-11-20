use crate::assert_solving_snapshots;

const SCHEMA: &str = r###"
enum join__Graph {
    A @join__graph(name: "a", url: "http://localhost:4200/provides-on-interface/a")
    B @join__graph(name: "b", url: "http://localhost:4200/provides-on-interface/b")
    C @join__graph(name: "c", url: "http://localhost:4200/provides-on-interface/c")
}

type Animal
    @join__type(graph: A)
    @join__type(graph: B)
    @join__type(graph: C)
{
    id: ID! @join__field(graph: C)
    name: String @join__field(graph: C)
    age: Int @join__field(graph: C)
}

type Query
    @join__type(graph: A)
    @join__type(graph: B)
    @join__type(graph: C)
{
    media: Media @join__field(graph: A) @join__field(graph: B, provides: "animals { id name }")
}

type Media
    @join__type(graph: A)
    @join__type(graph: B)
    @join__type(graph: C)
{
    id: ID!
    animals: [Animal] @join__field(graph: B) @join__field(graph: C)
}
"###;

#[test]
fn should_have_a_single_query_partition() {
    assert_solving_snapshots!(
        "basic",
        SCHEMA,
        r#"
        query {
          media {
            id
            animals {
              id
              name
            }
          }
        }
        "#
    );
}
