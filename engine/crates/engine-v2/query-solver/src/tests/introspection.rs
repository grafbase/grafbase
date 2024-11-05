use crate::assert_solving_snapshots;

const SCHEMA: &str = r###"
enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts", url: "http://accounts:4001/graphql")
}

type Query
  @join__type(graph: ACCOUNTS)
{
  me: String @join__field(graph: ACCOUNTS)
}

"###;

#[test]
fn schema() {
    assert_solving_snapshots!(
        "schema",
        SCHEMA,
        r#"
        query {
          __schema {
            queryType { name }
          }
        }
        "#
    );
}
