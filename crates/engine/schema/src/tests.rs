use crate::Schema;

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

    let config = std::sync::Arc::new(config);
    let schema1 = Schema::builder(SCHEMA).config(config.clone()).build().await.unwrap();
    let schema1bis = Schema::builder(SCHEMA).config(config.clone()).build().await.unwrap();
    let schema2 = Schema::builder(SCHEMA_WITH_INACCESSIBLES)
        .config(config)
        .build()
        .await
        .unwrap();

    assert_eq!(schema1.hash, schema1bis.hash);
    assert_ne!(schema1.hash, schema2.hash);
}

const PG_SCHEMA: &str = r#"
directive @join__unionMember(graph: join__Graph!, member: String!) on UNION

directive @join__implements(graph: join__Graph!, interface: String!) on OBJECT | INTERFACE

directive @join__graph(name: String!, url: String) on ENUM_VALUE

directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet, type: String, external: Boolean, override: String, overrideLabel: String) on FIELD_DEFINITION

directive @join__type(graph: join__Graph, key: join__FieldSet, resolvable: Boolean = true) on OBJECT | INTERFACE

directive @join__owner(graph: join__Graph!) on OBJECT

scalar JSON

scalar Bytes

scalar BigInt

scalar Decimal

scalar join__FieldSet

type PageInfo
  @join__type(graph: POSTGRES)
{
  endCursor: String!
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String!
}

type ProfileReturning
  @join__type(graph: POSTGRES)
{
  address: String
  avatarUrl: String
  bio: String
  birthday: String
  city: String
  country: String
  firstName: String
  gender: String
  id: BigInt!
  interests: [String]
  lastName: String
  phone: String
  postalCode: String
  socialMedia: JSON
  updatedAt: String!
  userId: BigInt!
  visibility: String
  websiteUrl: String
}

type Profile
  @join__type(graph: POSTGRES)
{
  address: String
  avatarUrl: String
  bio: String
  birthday: String
  city: String
  country: String
  firstName: String
  gender: String
  id: BigInt!
  interests: [String]
  lastName: String
  phone: String
  postalCode: String
  socialMedia: JSON
  updatedAt: String!
  user: User
  userId: BigInt!
  visibility: String
  websiteUrl: String
}

type ProfileCreatePayload
  @join__type(graph: POSTGRES)
{
  returning: ProfileReturning
  rowCount: Int!
}

type ProfileCreateManyPayload
  @join__type(graph: POSTGRES)
{
  returning: [ProfileReturning]!
  rowCount: Int!
}

type ProfileUpdatePayload
  @join__type(graph: POSTGRES)
{
  returning: ProfileReturning
  rowCount: Int!
}

type ProfileUpdateManyPayload
  @join__type(graph: POSTGRES)
{
  returning: [ProfileReturning]!
  rowCount: Int!
}

type ProfileDeletePayload
  @join__type(graph: POSTGRES)
{
  returning: ProfileReturning
  rowCount: Int!
}

type ProfileDeleteManyPayload
  @join__type(graph: POSTGRES)
{
  returning: [ProfileReturning]!
  rowCount: Int!
}

type ProfileEdge
  @join__type(graph: POSTGRES)
{
  cursor: String!
  node: Profile!
}

type ProfileConnection
  @join__type(graph: POSTGRES)
{
  edges: [ProfileEdge!]!
  pageInfo: PageInfo!
}

type UserReturning
  @join__type(graph: POSTGRES)
{
  createdAt: String!
  email: String!
  id: BigInt!
  isActive: Boolean!
  lastLogin: String
  loginCount: Int!
  metadata: JSON
  passwordHash: String!
  preferences: JSON
  role: String
  status: UserStatus
  username: String!
}

type User
  @join__type(graph: POSTGRES)
{
  createdAt: String!
  email: String!
  id: BigInt!
  isActive: Boolean!
  lastLogin: String
  loginCount: Int!
  metadata: JSON
  passwordHash: String!
  preferences: JSON
  profile: Profile!
  role: String
  status: UserStatus
  username: String!
}

type UserCreatePayload
  @join__type(graph: POSTGRES)
{
  returning: UserReturning
  rowCount: Int!
}

type UserCreateManyPayload
  @join__type(graph: POSTGRES)
{
  returning: [UserReturning]!
  rowCount: Int!
}

type UserUpdatePayload
  @join__type(graph: POSTGRES)
{
  returning: UserReturning
  rowCount: Int!
}

type UserUpdateManyPayload
  @join__type(graph: POSTGRES)
{
  returning: [UserReturning]!
  rowCount: Int!
}

type UserDeletePayload
  @join__type(graph: POSTGRES)
{
  returning: UserReturning
  rowCount: Int!
}

type UserDeleteManyPayload
  @join__type(graph: POSTGRES)
{
  returning: [UserReturning]!
  rowCount: Int!
}

type UserEdge
  @join__type(graph: POSTGRES)
{
  cursor: String!
  node: User!
}

type UserConnection
  @join__type(graph: POSTGRES)
{
  edges: [UserEdge!]!
  pageInfo: PageInfo!
}

type Query
{
  profile(lookup: ProfileLookupInput!): Profile @join__field(graph: POSTGRES)
  profiles(filter: ProfileFilterInput, first: Int, last: Int, before: String, after: String, orderBy: [ProfileOrderByInput!]): ProfileConnection! @join__field(graph: POSTGRES)
  user(lookup: UserLookupInput!): User @join__field(graph: POSTGRES)
  users(filter: UserFilterInput, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput!]): UserConnection! @join__field(graph: POSTGRES)
}

type Mutation
{
  profileCreate(input: ProfileCreateInput!): ProfileCreatePayload! @join__field(graph: POSTGRES)
  profileCreateMany(input: [ProfileCreateInput!]!): ProfileCreateManyPayload! @join__field(graph: POSTGRES)
  profileDelete(lookup: ProfileLookupInput!): ProfileDeletePayload! @join__field(graph: POSTGRES)
  profileDeleteMany(filter: ProfileFilterInput): ProfileDeleteManyPayload! @join__field(graph: POSTGRES)
  profileUpdate(lookup: ProfileLookupInput!, input: ProfileUpdateInput!): ProfileUpdatePayload! @join__field(graph: POSTGRES)
  profileUpdateMany(filter: ProfileFilterInput, input: ProfileUpdateInput!): ProfileUpdateManyPayload! @join__field(graph: POSTGRES)
  userCreate(input: UserCreateInput!): UserCreatePayload! @join__field(graph: POSTGRES)
  userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload! @join__field(graph: POSTGRES)
  userDelete(lookup: UserLookupInput!): UserDeletePayload! @join__field(graph: POSTGRES)
  userDeleteMany(filter: UserFilterInput): UserDeleteManyPayload! @join__field(graph: POSTGRES)
  userUpdate(lookup: UserLookupInput!, input: UserUpdateInput!): UserUpdatePayload! @join__field(graph: POSTGRES)
  userUpdateMany(filter: UserFilterInput, input: UserUpdateInput!): UserUpdateManyPayload! @join__field(graph: POSTGRES)
}

enum OrderDirection
  @join__type(graph: POSTGRES)
{
  ASC
  DESC
}

enum UserStatus
  @join__type(graph: POSTGRES)
{
  ACTIVE
  INACTIVE
  SUSPENDED
  PENDING
}

enum join__Graph
{
  POSTGRES @join__graph(name: "postgres")
}

input StringFilterInput
  @join__type(graph: POSTGRES)
{
  eq: String
  ne: String
  gt: String
  lt: String
  gte: String
  lte: String
  like: String
  in: [String]
  nin: [String]
  not: StringFilterInput
}

input StringUpdateInput
  @join__type(graph: POSTGRES)
{
  set: String
}

input StringArrayUpdateInput
  @join__type(graph: POSTGRES)
{
  set: [String]
  append: [String]
  prepend: [String]
}

input BigIntFilterInput
  @join__type(graph: POSTGRES)
{
  eq: BigInt
  ne: BigInt
  gt: BigInt
  lt: BigInt
  gte: BigInt
  lte: BigInt
  in: [BigInt]
  nin: [BigInt]
  not: BigIntFilterInput
}

input BigIntUpdateInput
  @join__type(graph: POSTGRES)
{
  set: BigInt
  increment: BigInt
  decrement: BigInt
  multiply: BigInt
  divide: BigInt
}

input BigIntArrayUpdateInput
  @join__type(graph: POSTGRES)
{
  set: [BigInt]
  append: [BigInt]
  prepend: [BigInt]
}

input IntFilterInput
  @join__type(graph: POSTGRES)
{
  eq: Int
  ne: Int
  gt: Int
  lt: Int
  gte: Int
  lte: Int
  in: [Int]
  nin: [Int]
  not: IntFilterInput
}

input IntUpdateInput
  @join__type(graph: POSTGRES)
{
  set: Int
  increment: Int
  decrement: Int
  multiply: Int
  divide: Int
}

input IntArrayUpdateInput
  @join__type(graph: POSTGRES)
{
  set: [Int]
  append: [Int]
  prepend: [Int]
}

input FloatFilterInput
  @join__type(graph: POSTGRES)
{
  eq: Float
  ne: Float
  gt: Float
  lt: Float
  gte: Float
  lte: Float
  in: [Float]
  nin: [Float]
  not: FloatFilterInput
}

input FloatUpdateInput
  @join__type(graph: POSTGRES)
{
  set: Float
  increment: Float
  decrement: Float
  multiply: Float
  divide: Float
}

input FloatArrayUpdateInput
  @join__type(graph: POSTGRES)
{
  set: [Float]
  append: [Float]
  prepend: [Float]
}

input BooleanFilterInput
  @join__type(graph: POSTGRES)
{
  eq: Boolean
  ne: Boolean
  gt: Boolean
  lt: Boolean
  gte: Boolean
  lte: Boolean
  in: [Boolean]
  nin: [Boolean]
  not: BooleanFilterInput
}

input BooleanUpdateInput
  @join__type(graph: POSTGRES)
{
  set: Boolean
}

input BooleanArrayUpdateInput
  @join__type(graph: POSTGRES)
{
  set: [Boolean]
  append: [Boolean]
  prepend: [Boolean]
}

input DecimalFilterInput
  @join__type(graph: POSTGRES)
{
  eq: Decimal
  ne: Decimal
  gt: Decimal
  lt: Decimal
  gte: Decimal
  lte: Decimal
  in: [Decimal]
  nin: [Decimal]
  not: DecimalFilterInput
}

input DecimalUpdateInput
  @join__type(graph: POSTGRES)
{
  set: Decimal
  increment: Decimal
  decrement: Decimal
  multiply: Decimal
  divide: Decimal
}

input DecimalArrayUpdateInput
  @join__type(graph: POSTGRES)
{
  set: [Decimal]
  append: [Decimal]
  prepend: [Decimal]
}

input BytesFilterInput
  @join__type(graph: POSTGRES)
{
  eq: Bytes
  ne: Bytes
  gt: Bytes
  lt: Bytes
  gte: Bytes
  lte: Bytes
  in: [Bytes]
  nin: [Bytes]
  not: BytesFilterInput
}

input BytesUpdateInput
  @join__type(graph: POSTGRES)
{
  set: Bytes
}

input BytesArrayUpdateInput
  @join__type(graph: POSTGRES)
{
  set: [Bytes]
  append: [Bytes]
  prepend: [Bytes]
}

input JSONFilterInput
  @join__type(graph: POSTGRES)
{
  eq: JSON
  ne: JSON
  gt: JSON
  lt: JSON
  gte: JSON
  lte: JSON
  in: [JSON]
  nin: [JSON]
  not: JSONFilterInput
}

input JSONUpdateInput
  @join__type(graph: POSTGRES)
{
  set: JSON
  append: JSON
  prepend: JSON
  deleteKey: String
  deleteElem: Int
  deleteAtPath: [String!]
}

input JSONArrayUpdateInput
  @join__type(graph: POSTGRES)
{
  set: [JSON]
  append: [JSON]
  prepend: [JSON]
}

input StringArrayFilterInput
  @join__type(graph: POSTGRES)
{
  eq: [String]
  ne: [String]
  gt: [String]
  lt: [String]
  gte: [String]
  lte: [String]
  in: [[String]]
  nin: [[String]]
  not: StringArrayFilterInput
  contains: [String]
  contained: [String]
  overlaps: [String]
}

input IntArrayFilterInput
  @join__type(graph: POSTGRES)
{
  eq: [Int]
  ne: [Int]
  gt: [Int]
  lt: [Int]
  gte: [Int]
  lte: [Int]
  in: [[Int]]
  nin: [[Int]]
  not: IntArrayFilterInput
  contains: [Int]
  contained: [Int]
  overlaps: [Int]
}

input BigIntArrayFilterInput
  @join__type(graph: POSTGRES)
{
  eq: [BigInt]
  ne: [BigInt]
  gt: [BigInt]
  lt: [BigInt]
  gte: [BigInt]
  lte: [BigInt]
  in: [[BigInt]]
  nin: [[BigInt]]
  not: BigIntArrayFilterInput
  contains: [BigInt]
  contained: [BigInt]
  overlaps: [BigInt]
}

input DecimalArrayFilterInput
  @join__type(graph: POSTGRES)
{
  eq: [Decimal]
  ne: [Decimal]
  gt: [Decimal]
  lt: [Decimal]
  gte: [Decimal]
  lte: [Decimal]
  in: [[Decimal]]
  nin: [[Decimal]]
  not: DecimalArrayFilterInput
  contains: [Decimal]
  contained: [Decimal]
  overlaps: [Decimal]
}

input FloatArrayFilterInput
  @join__type(graph: POSTGRES)
{
  eq: [Float]
  ne: [Float]
  gt: [Float]
  lt: [Float]
  gte: [Float]
  lte: [Float]
  in: [[Float]]
  nin: [[Float]]
  not: FloatArrayFilterInput
  contains: [Float]
  contained: [Float]
  overlaps: [Float]
}

input BooleanArrayFilterInput
  @join__type(graph: POSTGRES)
{
  eq: [Boolean]
  ne: [Boolean]
  gt: [Boolean]
  lt: [Boolean]
  gte: [Boolean]
  lte: [Boolean]
  in: [[Boolean]]
  nin: [[Boolean]]
  not: BooleanArrayFilterInput
  contains: [Boolean]
  contained: [Boolean]
  overlaps: [Boolean]
}

input BytesArrayFilterInput
  @join__type(graph: POSTGRES)
{
  eq: [Bytes]
  ne: [Bytes]
  gt: [Bytes]
  lt: [Bytes]
  gte: [Bytes]
  lte: [Bytes]
  in: [[Bytes]]
  nin: [[Bytes]]
  not: BytesArrayFilterInput
  contains: [Bytes]
  contained: [Bytes]
  overlaps: [Bytes]
}

input JSONArrayFilterInput
  @join__type(graph: POSTGRES)
{
  eq: [JSON]
  ne: [JSON]
  gt: [JSON]
  lt: [JSON]
  gte: [JSON]
  lte: [JSON]
  in: [[JSON]]
  nin: [[JSON]]
  not: JSONArrayFilterInput
  contains: [JSON]
  contained: [JSON]
  overlaps: [JSON]
}

input UserStatusFilterInput
  @join__type(graph: POSTGRES)
{
  eq: UserStatus
  ne: UserStatus
  gt: UserStatus
  lt: UserStatus
  gte: UserStatus
  lte: UserStatus
  in: [UserStatus]
  nin: [UserStatus]
  not: UserStatusFilterInput
}

input UserStatusArrayFilterInput
  @join__type(graph: POSTGRES)
{
  eq: [UserStatus]
  ne: [UserStatus]
  gt: [UserStatus]
  lt: [UserStatus]
  gte: [UserStatus]
  lte: [UserStatus]
  in: [[UserStatus]]
  nin: [[UserStatus]]
  not: UserStatusArrayFilterInput
  contains: [UserStatus]
  contained: [UserStatus]
  overlaps: [UserStatus]
}

input UserStatusUpdateInput
  @join__type(graph: POSTGRES)
{
  set: UserStatus
}

input UserStatusArrayUpdateInput
  @join__type(graph: POSTGRES)
{
  set: [UserStatus]
  append: [UserStatus]
  prepend: [UserStatus]
}

input ProfileOrderByInput
  @join__type(graph: POSTGRES)
{
  id: OrderDirection
  userId: OrderDirection
  firstName: OrderDirection
  lastName: OrderDirection
  bio: OrderDirection
  avatarUrl: OrderDirection
  birthday: OrderDirection
  gender: OrderDirection
  phone: OrderDirection
  address: OrderDirection
  city: OrderDirection
  country: OrderDirection
  postalCode: OrderDirection
  interests: OrderDirection
  websiteUrl: OrderDirection
  socialMedia: OrderDirection
  updatedAt: OrderDirection
  visibility: OrderDirection
  user: UserOrderByInput
}

input ProfileLookupInput
  @join__type(graph: POSTGRES)
{
  id: BigInt
  userId: BigInt
}

input ProfileCollectionFilterInput
  @join__type(graph: POSTGRES)
{
  contains: ProfileFilterInput
}

input ProfileFilterInput
  @join__type(graph: POSTGRES)
{
  id: BigIntFilterInput
  userId: BigIntFilterInput
  firstName: StringFilterInput
  lastName: StringFilterInput
  bio: StringFilterInput
  avatarUrl: StringFilterInput
  birthday: StringFilterInput
  gender: StringFilterInput
  phone: StringFilterInput
  address: StringFilterInput
  city: StringFilterInput
  country: StringFilterInput
  postalCode: StringFilterInput
  interests: StringArrayFilterInput
  websiteUrl: StringFilterInput
  socialMedia: JSONFilterInput
  updatedAt: StringFilterInput
  visibility: StringFilterInput
  user: UserFilterInput
  ALL: [ProfileFilterInput]
  NONE: [ProfileFilterInput]
  ANY: [ProfileFilterInput]
}

input ProfileCreateInput
  @join__type(graph: POSTGRES)
{
  userId: BigInt!
  firstName: String
  lastName: String
  bio: String
  avatarUrl: String
  birthday: String
  gender: String
  phone: String
  address: String
  city: String
  country: String
  postalCode: String
  interests: [String]
  websiteUrl: String
  socialMedia: JSON
  updatedAt: String
  visibility: String
}

input ProfileUpdateInput
  @join__type(graph: POSTGRES)
{
  userId: BigIntUpdateInput
  firstName: StringUpdateInput
  lastName: StringUpdateInput
  bio: StringUpdateInput
  avatarUrl: StringUpdateInput
  birthday: StringUpdateInput
  gender: StringUpdateInput
  phone: StringUpdateInput
  address: StringUpdateInput
  city: StringUpdateInput
  country: StringUpdateInput
  postalCode: StringUpdateInput
  interests: StringArrayUpdateInput
  websiteUrl: StringUpdateInput
  socialMedia: JSONUpdateInput
  updatedAt: StringUpdateInput
  visibility: StringUpdateInput
}

input UserOrderByInput
  @join__type(graph: POSTGRES)
{
  id: OrderDirection
  username: OrderDirection
  email: OrderDirection
  passwordHash: OrderDirection
  createdAt: OrderDirection
  lastLogin: OrderDirection
  loginCount: OrderDirection
  isActive: OrderDirection
  preferences: OrderDirection
  metadata: OrderDirection
  role: OrderDirection
  status: OrderDirection
  profile: ProfileOrderByInput
}

input UserLookupInput
  @join__type(graph: POSTGRES)
{
  id: BigInt
}

input UserCollectionFilterInput
  @join__type(graph: POSTGRES)
{
  contains: UserFilterInput
}

input UserFilterInput
  @join__type(graph: POSTGRES)
{
  id: BigIntFilterInput
  ALL: [UserFilterInput]
  NONE: [UserFilterInput]
  ANY: [UserFilterInput]
  username: StringFilterInput
  email: StringFilterInput
  passwordHash: StringFilterInput
  createdAt: StringFilterInput
  lastLogin: StringFilterInput
  loginCount: IntFilterInput
  isActive: BooleanFilterInput
  preferences: JSONFilterInput
  metadata: JSONFilterInput
  role: StringFilterInput
  status: UserStatusFilterInput
  profile: ProfileFilterInput
}

input UserCreateInput
  @join__type(graph: POSTGRES)
{
  username: String!
  email: String!
  passwordHash: String!
  createdAt: String
  lastLogin: String
  loginCount: Int
  isActive: Boolean
  preferences: JSON
  metadata: JSON
  role: String
  status: UserStatus
}

input UserUpdateInput
  @join__type(graph: POSTGRES)
{
  username: StringUpdateInput
  email: StringUpdateInput
  passwordHash: StringUpdateInput
  createdAt: StringUpdateInput
  lastLogin: StringUpdateInput
  loginCount: IntUpdateInput
  isActive: BooleanUpdateInput
  preferences: JSONUpdateInput
  metadata: JSONUpdateInput
  role: StringUpdateInput
  status: UserStatusUpdateInput
}
"#;

#[tokio::test]
async fn debug() {
    use std::fmt::Write;

    let schema = Schema::from_sdl_or_panic(PG_SCHEMA).await;

    let mut s = String::with_capacity(1024);
    for def in schema.type_definitions() {
        println!("{}", def.name());
        write!(s, "{def:?}").unwrap();
        s.clear();
    }

    for def in schema.field_definitions() {
        println!("{}.{}", def.parent_entity().name(), def.name());
        write!(s, "{def:?}").unwrap();
        s.clear();
    }

    for def in schema.resolver_definitions() {
        println!("{}", def.name());
        write!(s, "{def:?}").unwrap();
        s.clear();
    }
}

const SCHEMA_WITH_EXTENSION: &str = r#"
directive @join__unionMember(graph: join__Graph!, member: String!) on UNION

directive @join__implements(graph: join__Graph!, interface: String!) on OBJECT | INTERFACE

directive @join__graph(name: String!, url: String) on ENUM_VALUE

directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet, type: String, external: Boolean, override: String, overrideLabel: String) on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

directive @join__type(graph: join__Graph, key: join__FieldSet, extension: Boolean = false, resolvable: Boolean = true, isInterfaceObject: Boolean = false) on SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT

directive @join__owner(graph: join__Graph!) on OBJECT

scalar JSON

scalar join__FieldSet

type Query
{
  echo(first: Int, limit: Int, after: String, filters: Filters): JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {input: "*"}) @join__field(graph: A)
}

enum join__Graph
{
  A @join__graph(name: "a")
}

enum extension__Link
{
  ECHO @extension__link(url: "file:///tmp/.tmpv8mlGN/extensions/echo-1.0.0")
}

input Filters
  @join__type(graph: A)
{
  latest: Boolean
  nested: Nested
}

input Nested
  @join__type(graph: A)
{
  id: ID
  name: String
}
"#;

#[tokio::test]
async fn for_operation_analytics_only() {
    assert!(Schema::builder(SCHEMA_WITH_EXTENSION).build().await.is_err());
    assert!(
        Schema::builder(SCHEMA_WITH_EXTENSION)
            .for_operation_analytics_only()
            .build()
            .await
            .is_ok()
    );
}
