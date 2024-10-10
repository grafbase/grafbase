use crate::{
    tests::{read_schema, TestOperation},
    OperationGraph,
};

const SCHEMA: &str = r###"
directive @core(feature: String!) repeatable on SCHEMA

directive @join__owner(graph: join__Graph!) on OBJECT

directive @join__type(
    graph: join__Graph!
    key: String!
    resolvable: Boolean = true
) repeatable on OBJECT | INTERFACE

directive @join__field(
    graph: join__Graph
    requires: String
    provides: String
) on FIELD_DEFINITION

directive @join__graph(name: String!, url: String!) on ENUM_VALUE

directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

enum join__Graph {
    ACCOUNTS @join__graph(name: "accounts", url: "http://127.0.0.1:39203/")
    INVENTORY @join__graph(name: "inventory", url: "http://127.0.0.1:33259/")
    PRODUCTS @join__graph(name: "products", url: "http://127.0.0.1:42663/")
    REVIEWS @join__graph(name: "reviews", url: "http://127.0.0.1:38447/")
    SHIPPING @join__graph(name: "shipping", url: "http://127.0.0.1:38213/")
}

type BusinessAccount implements Account
    @join__implements(graph: SHIPPING, interface: "Account")
    @join__type(graph: ACCOUNTS, key: "id")
    @join__type(graph: SHIPPING, key: "id email joinedTimestamp", resolvable: false)
{
    businessName: String! @join__field(graph: ACCOUNTS)
    email: String! @join__field(graph: ACCOUNTS)
    id: ID!
    joinedTimestamp: Int! @join__field(graph: ACCOUNTS)
}

type Cart {
    products: [Product!]! @join__field(graph: ACCOUNTS)
}

type Picture {
    altText: String! @inaccessible
    height: Int!
    url: String!
    width: Int!
}

type Product
    @join__type(graph: ACCOUNTS, key: "name", resolvable: false)
    @join__type(graph: INVENTORY, key: "upc")
    @join__type(graph: PRODUCTS, key: "upc")
    @join__type(graph: PRODUCTS, key: "name")
    @join__type(graph: REVIEWS, key: "upc")
{
    availableShippingService: [ShippingService!]! @join__field(graph: INVENTORY, requires: "weight(unit: KILOGRAM)")
    name: String!
    price: Int! @join__field(graph: PRODUCTS)
    reviews: [Review!]! @join__field(graph: REVIEWS)
    shippingEstimate: Int! @join__field(graph: INVENTORY, requires: "weight(unit: KILOGRAM)")
    upc: String!
    weight(unit: WeightUnit!): Float! @join__field(graph: PRODUCTS)
}

type User implements Account
    @join__implements(graph: SHIPPING, interface: "Account")
    @join__type(graph: ACCOUNTS, key: "id")
    @join__type(graph: REVIEWS, key: "id")
    @join__type(graph: SHIPPING, key: "id username reviewCount joinedTimestamp", resolvable: false)
{
    cart: Cart! @join__field(graph: ACCOUNTS)
    id: ID!
    joinedTimestamp: Int! @join__field(graph: ACCOUNTS)
    profilePicture: Picture @join__field(graph: ACCOUNTS)
    """
    This used to be part of this subgraph, but is now being overridden from
    `reviews`
    """
    reviewCount: Int! @join__field(graph: REVIEWS, override: "accounts")
    reviews: [Review!]! @join__field(graph: REVIEWS)
    trustworthiness: Trustworthiness! @join__field(graph: REVIEWS, requires: "joinedTimestamp")
    username: String! @join__field(graph: ACCOUNTS)
}

type DeliveryCompany implements ShippingService & ShippingModality
    @join__implements(graph: INVENTORY, interface: "ShippingService")
    @join__implements(graph: SHIPPING, interface: "ShippingModality")
    @join__type(graph: INVENTORY, key: "id")
    @join__type(graph: SHIPPING, key: "id")
{
    companyType: String! @join__field(graph: INVENTORY)
    id: String!
    name: String! @join__field(graph: INVENTORY)
    qualifiedName: String! @join__field(graph: SHIPPING, requires: "companyType")
    reviews: [ShippingServiceReview!]! @join__field(graph: REVIEWS, requires: "name")
}

type HomingPigeon implements ShippingService & ShippingModality
    @join__implements(graph: INVENTORY, interface: "ShippingService")
    @join__implements(graph: SHIPPING, interface: "ShippingModality")
    @join__type(graph: INVENTORY, key: "id")
    @join__type(graph: SHIPPING, key: "id")
{
    id: String!
    name: String! @join__field(graph: INVENTORY)
    nickname: String! @join__field(graph: INVENTORY)
    qualifiedName: String! @join__field(graph: SHIPPING, requires: "nickname")
    reviews: [ShippingServiceReview!]! @join__field(graph: REVIEWS, requires: "name")
}

type Review {
    author: User @join__field(graph: REVIEWS)
    body: String! @join__field(graph: REVIEWS)
    id: ID! @join__field(graph: REVIEWS)
    pictures: [Picture!]! @join__field(graph: REVIEWS)
    product: Product! @join__field(graph: REVIEWS, provides: "price")
}

type ShippingServiceReview {
    body: String! @join__field(graph: REVIEWS)
}

type ShippingOptions {
    defaultCompanySummary: String! @join__field(graph: SHIPPING, requires: "defaultDeliveryCompany { companyType }")
    defaultDeliveryCompany: DeliveryCompany! @join__field(graph: SHIPPING)
    modalities: [ShippingModality!]! @join__field(graph: SHIPPING)
    seller: Account! @join__field(graph: SHIPPING, provides: "... on BusinessAccount { email }... on User { reviewCount }")
    summary: String! @join__field(graph: SHIPPING, requires: "modalities { ... on HomingPigeon { nickname }... on DeliveryCompany { companyType } }")
}

type Query {
    me: User! @join__field(graph: ACCOUNTS)
    product(upc: String!): Product @join__field(graph: PRODUCTS)
    shippingOptions: ShippingOptions! @join__field(graph: SHIPPING)
    topProducts: [Product!]! @join__field(graph: PRODUCTS)
}

type Subscription {
    newProducts: Product! @join__field(graph: PRODUCTS)
}

interface ShippingService
    @join__type(graph: INVENTORY, key: "id")
    @join__type(graph: REVIEWS, key: "id", isInterfaceObject: true)
{
    id: String!
    name: String! @join__field(graph: INVENTORY)
    reviews: [ShippingServiceReview!]! @join__field(graph: REVIEWS, requires: "name")
}

interface Account {
    id: ID!
    joinedTimestamp: Int!
}

interface ShippingModality {
    id: String!
    name: String!
    qualifiedName: String!
}

enum WeightUnit {
    KILOGRAM
    GRAM
}

enum Trustworthiness {
    REALLY_TRUSTED
    KINDA_TRUSTED
    NOT_TRUSTED
}
"###;

#[test]
fn sibling_dependency() {
    let schema = read_schema(SCHEMA);
    let mut operation = TestOperation::bind(
        &schema,
        r#"
        query {
            me {
                id
                username
                cart {
                    products {
                        price
                        reviews {
                            author {
                                id
                                username
                            }
                            body
                        }
                    }
                }
            }
        }
    "#,
    );

    let mut graph = OperationGraph::new(&schema, &mut operation);
    insta::assert_snapshot!("graph", graph.to_dot_graph(), &graph.to_pretty_dot_graph());

    graph.estimate_resolver_costs();
    insta::assert_snapshot!("cost", graph.to_dot_graph(), &graph.to_pretty_dot_graph());
}
