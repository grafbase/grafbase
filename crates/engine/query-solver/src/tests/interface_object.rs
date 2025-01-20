use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
enum join__Graph {
    A @join__graph(name: "a", url: "http://localhost:4200/simple-interface-object/a")
    B @join__graph(name: "b", url: "http://localhost:4200/simple-interface-object/b")
    C @join__graph(name: "c", url: "http://localhost:4200/simple-interface-object/c")
}

type Admin implements Account
    @join__implements(graph: A, interface: "Account")
    @join__type(graph: A, key: "id")
{
    id: ID!
    isMain: Boolean!
    isActive: Boolean!
    name: String! @join__field
}

type Query
    @join__type(graph: A)
    @join__type(graph: B)
    @join__type(graph: C)
{
    users: [NodeWithName!]! @join__field(graph: A)
    anotherUsers: [NodeWithName] @join__field(graph: B)
    accounts: [Account] @join__field(graph: B)
}

type Regular implements Account
    @join__implements(graph: A, interface: "Account")
    @join__type(graph: A, key: "id")
{
    id: ID!
    isMain: Boolean!
    name: String! @join__field
    isActive: Boolean! @join__field
}

type User implements NodeWithName
    @join__implements(graph: A, interface: "NodeWithName")
    @join__type(graph: A, key: "id")
{
    id: ID!
    name: String
    age: Int
    username: String @join__field
}

interface Account
    @join__type(graph: A, key: "id")
    @join__type(graph: B, key: "id", isInterfaceObject: true)
    @join__type(graph: C, key: "id", isInterfaceObject: true)
{
    id: ID!
    name: String! @join__field(graph: B)
    isActive: Boolean! @join__field(graph: C)
}

interface NodeWithName
    @join__type(graph: A, key: "id")
    @join__type(graph: B, key: "id", isInterfaceObject: true)
{
    id: ID!
    name: String @join__field(graph: A)
    username: String @join__field(graph: B)
}
"#;

#[test]
fn interface_object_cannot_provide_typename() {
    assert_solving_snapshots!(
        "interface_object_cannot_provide_typename",
        SCHEMA,
        r#"
        query {
          accounts {
            ... on Admin {
              id
            }
          }
        }
        "#
    );
}
