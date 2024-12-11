use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
enum join__Graph {
    AGENCY @join__graph(name: "agency", url: "http://localhost:4200/abstract-types/agency")
    BOOKS @join__graph(name: "books", url: "http://localhost:4200/abstract-types/books")
    INVENTORY @join__graph(name: "inventory", url: "http://localhost:4200/abstract-types/inventory")
    MAGAZINES @join__graph(name: "magazines", url: "http://localhost:4200/abstract-types/magazines")
    PRODUCTS @join__graph(name: "products", url: "http://localhost:4200/abstract-types/products")
    REVIEWS @join__graph(name: "reviews", url: "http://localhost:4200/abstract-types/reviews")
    USERS @join__graph(name: "users", url: "http://localhost:4200/abstract-types/users")
}

type Agency
    @join__type(graph: AGENCY, key: "id")
    @join__type(graph: PRODUCTS)
{
    id: ID!
    companyName: String @join__field(graph: AGENCY)
}

type Book implements Product & Similar
    @join__implements(graph: INVENTORY, interface: "Product")
    @join__implements(graph: PRODUCTS, interface: "Product")
    @join__implements(graph: PRODUCTS, interface: "Similar")
    @join__implements(graph: REVIEWS, interface: "Product")
    @join__implements(graph: REVIEWS, interface: "Similar")
    @join__type(graph: BOOKS, key: "id")
    @join__type(graph: INVENTORY, key: "id")
    @join__type(graph: PRODUCTS, key: "id")
    @join__type(graph: REVIEWS, key: "id")
{
    id: ID!
    title: String @join__field(graph: BOOKS)
    dimensions: ProductDimension @join__field(graph: PRODUCTS)
    delivery(zip: String): DeliveryEstimates @join__field(graph: INVENTORY, requires: "dimensions { size weight }")
    sku: String @join__field(graph: PRODUCTS)
    createdBy: User @join__field(graph: PRODUCTS)
    similar: [Book] @join__field(graph: PRODUCTS)
    hidden: Boolean @join__field(graph: PRODUCTS)
    publisherType: PublisherType @join__field(graph: PRODUCTS)
    reviewsCount: Int! @join__field(graph: REVIEWS)
    reviewsScore: Float! @join__field(graph: REVIEWS)
    reviews: [Review!]! @join__field(graph: REVIEWS)
    reviewsOfSimilar: [Review!]! @join__field(graph: REVIEWS, requires: "similar { id }")
}

type DeliveryEstimates
    @join__type(graph: INVENTORY)
{
    estimatedDelivery: String
    fastestDelivery: String
}

type Group
    @join__type(graph: AGENCY, key: "id")
{
    id: ID!
    name: String
}

type Magazine implements Product & Similar
    @join__implements(graph: INVENTORY, interface: "Product")
    @join__implements(graph: PRODUCTS, interface: "Product")
    @join__implements(graph: PRODUCTS, interface: "Similar")
    @join__implements(graph: REVIEWS, interface: "Product")
    @join__implements(graph: REVIEWS, interface: "Similar")
    @join__type(graph: INVENTORY, key: "id")
    @join__type(graph: MAGAZINES, key: "id")
    @join__type(graph: PRODUCTS, key: "id")
    @join__type(graph: REVIEWS, key: "id")
{
    id: ID!
    dimensions: ProductDimension @join__field(graph: PRODUCTS)
    delivery(zip: String): DeliveryEstimates @join__field(graph: INVENTORY, requires: "dimensions { size weight }")
    title: String @join__field(graph: MAGAZINES)
    sku: String @join__field(graph: PRODUCTS)
    createdBy: User @join__field(graph: PRODUCTS)
    similar: [Magazine] @join__field(graph: PRODUCTS)
    hidden: Boolean @join__field(graph: PRODUCTS)
    publisherType: PublisherType @join__field(graph: PRODUCTS)
    reviewsCount: Int! @join__field(graph: REVIEWS)
    reviewsScore: Float! @join__field(graph: REVIEWS)
    reviews: [Review!]! @join__field(graph: REVIEWS)
    reviewsOfSimilar: [Review!]! @join__field(graph: REVIEWS, requires: "similar { id }")
}

type ProductDimension
    @join__type(graph: INVENTORY)
    @join__type(graph: PRODUCTS)
{
    size: String
    weight: Float
}

type Query
    @join__type(graph: AGENCY)
    @join__type(graph: BOOKS)
    @join__type(graph: INVENTORY)
    @join__type(graph: MAGAZINES)
    @join__type(graph: PRODUCTS)
    @join__type(graph: REVIEWS)
    @join__type(graph: USERS)
{
    books: [Book] @join__field(graph: BOOKS)
    magazines: [Magazine] @join__field(graph: MAGAZINES)
    products: [Product] @join__field(graph: PRODUCTS)
    similar(id: ID!): [Product] @join__field(graph: PRODUCTS)
    review(id: Int!): Review @join__field(graph: REVIEWS)
}

type Review
    @join__type(graph: REVIEWS)
{
    id: Int!
    body: String!
    product: Product
}

type Self
    @join__type(graph: PRODUCTS)
{
    email: String
}

type User
    @join__type(graph: PRODUCTS, key: "email")
    @join__type(graph: USERS, key: "email")
{
    email: ID!
    totalProductsCreated: Int
    name: String @join__field(graph: USERS)
}

interface Product
    @join__type(graph: INVENTORY)
    @join__type(graph: PRODUCTS)
    @join__type(graph: REVIEWS)
{
    id: ID!
    dimensions: ProductDimension @join__field(graph: INVENTORY) @join__field(graph: PRODUCTS)
    delivery(zip: String): DeliveryEstimates @join__field(graph: INVENTORY)
    sku: String @join__field(graph: PRODUCTS)
    createdBy: User @join__field(graph: PRODUCTS)
    hidden: Boolean @inaccessible @join__field(graph: PRODUCTS)
    reviewsCount: Int! @join__field(graph: REVIEWS)
    reviewsScore: Float! @join__field(graph: REVIEWS)
    reviews: [Review!]! @join__field(graph: REVIEWS)
}

interface Similar
    @join__type(graph: PRODUCTS)
    @join__type(graph: REVIEWS)
{
    similar: [Product]
}

union PublisherType
    @join__type(graph: AGENCY)
    @join__type(graph: PRODUCTS)
    @join__unionMember(graph: AGENCY, member: "Agency")
    @join__unionMember(graph: PRODUCTS, member: "Agency")
    @join__unionMember(graph: AGENCY, member: "Group")
    @join__unionMember(graph: PRODUCTS, member: "Self")
 = Agency | Group | Self
"#;

#[test]
fn interface_field_provided_by_implementors() {
    // delivery is not accessible on the Similar interface, but it is on all of its implementors.
    assert_solving_snapshots!(
        "interface_field_provided_by_implementors",
        SCHEMA,
        r#"
        query {
          similar(id: "p1") {
            id
            sku
            delivery(zip: "1234") {
              fastestDelivery
              estimatedDelivery
            }
          }
        }
        "#
    );
}

#[test]
fn nested_interface_field_provided_by_implementors() {
    assert_solving_snapshots!(
        "nested_interface_field_provided_by_implementors",
        SCHEMA,
        r#"
        {
          products {
            id
            reviews {
              product {
                sku
                ... on Magazine {
                  title
                }
                ... on Book {
                  reviewsCount
                }
              }
            }
          }
        }
        "#
    );
}

#[test]
fn unreachable_object() {
    // Group is never reachable.
    assert_solving_snapshots!(
        "unreachable_object",
        SCHEMA,
        r#"
        query {
          products {
            id
            ... on Magazine {
              publisherType {
                ...Publisher
              }
            }
            ... on Book {
              publisherType {
                ...Publisher
              }
            }
          }
        }

        fragment Publisher on PublisherType {
          ... on Agency {
            id
            companyName
          }
          ... on Self {
            email
          }
          ... on Group {
            name
          }
        }
        "#
    );
}
