use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
enum join__Graph {
    TEA_SHOP @join__graph(name: "tea-shop", url: "http://127.0.0.1:35289/")
}

type Address {
    street: String! @join__field(graph: TEA_SHOP)
}

type Order {
    amount: Int! @join__field(graph: TEA_SHOP)
    tea: Tea! @join__field(graph: TEA_SHOP)
}

"""
A type with a _required_ style, to test nullability bubbling.
"""
type StyleContainer {
    name: String! @join__field(graph: TEA_SHOP)
    style: TeaStyle! @join__field(graph: TEA_SHOP)
}

type Tea {
    id: Int! @join__field(graph: TEA_SHOP)
    name: String! @join__field(graph: TEA_SHOP)
    style: TeaStyle @join__field(graph: TEA_SHOP)
}

type User {
    address: Address! @join__field(graph: TEA_SHOP)
    favoriteTea: Tea @join__field(graph: TEA_SHOP)
    id: Int! @join__field(graph: TEA_SHOP)
    name: String! @join__field(graph: TEA_SHOP)
    orders: [Order!]! @join__field(graph: TEA_SHOP)
}

type Query {
    node(id: String!): Node @join__field(graph: TEA_SHOP)
    recommendedTeas: [Tea!]! @join__field(graph: TEA_SHOP)
    teaWithInaccessibleStyle: StyleContainer @join__field(graph: TEA_SHOP)
    user(id: Int!): User @join__field(graph: TEA_SHOP)
    users: [User!]! @join__field(graph: TEA_SHOP)
}

enum TeaStyle {
    WHITE
    OOLONG
    GREEN
    YELLOW
    BLACK
    PUER @inaccessible
    POST_FERMENTED
}

union Node
    @join__unionMember(graph: TEA_SHOP, member: "Tea")
    @join__unionMember(graph: TEA_SHOP, member: "User")
 = Tea | User

"#;

#[tokio::test]
async fn missing_style_enum() {
    assert_solving_snapshots!(
        "missing_tyle_enum",
        SCHEMA,
        r#"
        query {
          recommendedTeas {
            id
            name
            style
          }
          teaWithInaccessibleStyle {
            name
            style
          }
        }
        "#
    );
}
