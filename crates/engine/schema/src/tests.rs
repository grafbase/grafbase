use crate::Schema;
use postcard::ser_flavors::Flavor;

const SCHEMA: &str = r#"
schema
  @link(url: "https://specs.apollo.dev/link/v1.0")
  @link(url: "https://specs.apollo.dev/join/v0.3", for: EXECUTION)
{
  query: Query
}

directive @join__enumValue(graph: join__Graph!) repeatable on ENUM_VALUE

directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet, type: String, external: Boolean, override: String, usedOverridden: Boolean) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

directive @join__graph(name: String!, url: String!) on ENUM_VALUE

directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

directive @join__type(graph: join__Graph!, key: join__FieldSet, extension: Boolean! = false, resolvable: Boolean! = true, isInterfaceObject: Boolean! = false) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR

directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

directive @link(url: String, as: String, for: link__Purpose, import: [link__Import]) repeatable on SCHEMA

scalar join__FieldSet

enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts", url: "http://accounts:4001/graphql")
  INVENTORY @join__graph(name: "inventory", url: "http://inventory:4002/graphql")
  PRODUCTS @join__graph(name: "products", url: "http://products:4003/graphql")
  REVIEWS @join__graph(name: "reviews", url: "http://reviews:4004/graphql")
}

scalar link__Import

enum link__Purpose {
  """
  `SECURITY` features provide metadata necessary to securely resolve fields.
  """
  SECURITY

  """
  `EXECUTION` features provide metadata necessary for operation execution.
  """
  EXECUTION
}

type Product
  @join__type(graph: INVENTORY, key: "upc")
  @join__type(graph: PRODUCTS, key: "upc")
  @join__type(graph: REVIEWS, key: "upc")
{
  upc: String!
  weight: Int @join__field(graph: INVENTORY, external: true) @join__field(graph: PRODUCTS)
  price: Int @join__field(graph: INVENTORY, external: true) @join__field(graph: PRODUCTS)
  inStock: Boolean @join__field(graph: INVENTORY)
  shippingEstimate: Int @join__field(graph: INVENTORY, requires: "price weight")
  name: String @join__field(graph: PRODUCTS)
  reviews: [Review] @join__field(graph: REVIEWS)
}

type Query
  @join__type(graph: ACCOUNTS)
  @join__type(graph: INVENTORY)
  @join__type(graph: PRODUCTS)
  @join__type(graph: REVIEWS)
{
  me: User @join__field(graph: ACCOUNTS)
  user(id: ID!): User @join__field(graph: ACCOUNTS)
  users: [User] @join__field(graph: ACCOUNTS)
  topProducts(first: Int = 5): [Product] @join__field(graph: PRODUCTS)
}

type Review
  @join__type(graph: REVIEWS, key: "id")
{
  id: ID!
  body: String
  product: Product
  author: User @join__field(graph: REVIEWS, provides: "username")
}

type User
  @join__type(graph: ACCOUNTS, key: "id")
  @join__type(graph: REVIEWS, key: "id")
{
  id: ID!
  name: String @join__field(graph: ACCOUNTS)
  username: String @join__field(graph: ACCOUNTS) @join__field(graph: REVIEWS, external: true)
  birthday: Int @join__field(graph: ACCOUNTS)
  reviews: [Review] @join__field(graph: REVIEWS)
}
"#;

#[tokio::test]
async fn should_not_fail() {
    let _schema = Schema::from_sdl_or_panic(SCHEMA).await;
}

const SCHEMA_WITH_INACCESSIBLES: &str = r#"
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

enum join__Graph {
    FIVE_WITH_ENUM @join__graph(name: "five_with_enum", url: "http://example.com/five_with_enum")
    FOUR_WITH_UNION @join__graph(name: "four_with_union", url: "http://example.com/four_with_union")
    ONE @join__graph(name: "one", url: "http://example.com/one")
    SIX_WITH_INPUT_OBJECT @join__graph(name: "six_with_input_object", url: "http://example.com/six_with_input_object")
    THREE_WITH_INTERFACE @join__graph(name: "three_with_interface", url: "http://example.com/three_with_interface")
    TWO @join__graph(name: "two", url: "http://example.com/two")
}

scalar Time @inaccessible

type Ungulate {
    id: ID! @join__field(graph: FIVE_WITH_ENUM)
    name: String! @join__field(graph: FIVE_WITH_ENUM)
    type: UngulateType! @join__field(graph: FIVE_WITH_ENUM) @inaccessible
}

type Movie {
    id: ID! @join__field(graph: FOUR_WITH_UNION)
    title: String! @join__field(graph: FOUR_WITH_UNION)
    director: String! @join__field(graph: FOUR_WITH_UNION)
    releaseYear: Int @join__field(graph: FOUR_WITH_UNION)
}

type Series {
    id: ID! @join__field(graph: FOUR_WITH_UNION)
    title: String! @join__field(graph: FOUR_WITH_UNION)
    seasons: Int @join__field(graph: FOUR_WITH_UNION)
}

type New {
    name: String! @inaccessible
    other: String!
    message: String! @inaccessible
    old: Old! @inaccessible
}

type Old @inaccessible {
    name: String! @inaccessible
}

type Book {
    id: ID! @join__field(graph: SIX_WITH_INPUT_OBJECT)
    title: String! @join__field(graph: SIX_WITH_INPUT_OBJECT)
    author: String! @join__field(graph: SIX_WITH_INPUT_OBJECT)
    publishedYear: Int @join__field(graph: SIX_WITH_INPUT_OBJECT)
    genre: String @join__field(graph: SIX_WITH_INPUT_OBJECT)
}

type Quadratic implements Polynomial {
    degree: Int @join__field(graph: THREE_WITH_INTERFACE)
    coefficients: [Float] @join__field(graph: THREE_WITH_INTERFACE)
    discriminant: Float @join__field(graph: THREE_WITH_INTERFACE)
}

type Cubic implements Polynomial {
    degree: Int @join__field(graph: THREE_WITH_INTERFACE)
    coefficients: [Float] @join__field(graph: THREE_WITH_INTERFACE)
    inflectionPoint: Float @join__field(graph: THREE_WITH_INTERFACE)
}

type Query {
    getUngulate(id: ID!): Ungulate @join__field(graph: FIVE_WITH_ENUM)
    getTVContent(id: ID!): TVContent @join__field(graph: FOUR_WITH_UNION) @inaccessible
    getNew(name: String!): New @join__field(graph: ONE)
    getBook(id: ID!): Book @join__field(graph: SIX_WITH_INPUT_OBJECT)
    getPolynomial(id: ID!): Polynomial @join__field(graph: THREE_WITH_INTERFACE) @inaccessible
    currentTime: Time! @join__field(graph: TWO) @inaccessible
}

type Mutation {
    addBook(input: BookInput! @inaccessible): Book @join__field(graph: SIX_WITH_INPUT_OBJECT)
    updateBook(id: ID!, input: BookInput2! @inaccessible): Book @join__field(graph: SIX_WITH_INPUT_OBJECT)
}

interface Polynomial @inaccessible {
    degree: Int
    coefficients: [Float]
}

enum UngulateType @inaccessible {
    DEER
    HORSE @inaccessible
    CAMEL
    RHINOCEROS
    GIRAFFE
}

union TVContent @inaccessible = Movie | Series

union Continent = New | Old

input BookInput @inaccessible {
    title: String!
    author: String! @inaccessible
    publishedYear: Int
    genre: String
}

input BookInput2 {
    title: String!
    author: String! @inaccessible
    publishedYear: Int
    genre: String
}
"#;

#[rstest::rstest]
#[case(SCHEMA)]
#[case(SCHEMA_WITH_INACCESSIBLES)]
#[tokio::test]
async fn serde_roundtrip(#[case] sdl: &str) {
    let config: gateway_config::Config = toml::from_str(
        r#"
        [[headers]]
        rule = "insert"
        name = "x-foo"
        value = "BAR"

        [[headers]]
        rule = "forward"
        name = "x-source"
        rename = "x-forwarded"
        "#,
    )
    .unwrap();

    let schema = Schema::build(&config, sdl, &Default::default()).await.unwrap();

    let mut serializer = postcard::Serializer {
        output: postcard::ser_flavors::AllocVec::new(),
    };
    let result = serde_path_to_error::serialize(&schema, &mut serializer);
    if let Err(err) = result {
        let path = err.path().to_string();
        panic!("Failed serialization at {path}: {}", err.into_inner());
    }

    let bytes = serializer.output.finalize().unwrap();
    let result: Result<Schema, _> = serde_path_to_error::deserialize(&mut postcard::Deserializer::from_bytes(&bytes));
    if let Err(err) = result {
        let path = err.path().to_string();
        panic!("Failed deserialization at {path}: {}", err.into_inner());
    }
}

#[tokio::test]
async fn consistent_hash() {
    let config: gateway_config::Config = toml::from_str(
        r#"
        [[headers]]
        rule = "insert"
        name = "x-foo"
        value = "BAR"

        [[headers]]
        rule = "forward"
        name = "x-source"
        rename = "x-forwarded"
        "#,
    )
    .unwrap();

    let schema1 = Schema::build(&config, SCHEMA, &Default::default()).await.unwrap();
    let schema1bis = Schema::build(&config, SCHEMA, &Default::default()).await.unwrap();
    let schema2 = Schema::build(&config, SCHEMA_WITH_INACCESSIBLES, &Default::default())
        .await
        .unwrap();

    assert_eq!(schema1.hash, schema1bis.hash);
    assert_ne!(schema1.hash, schema2.hash);
}
