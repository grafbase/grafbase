//! Tests that the generated schema for mongo makes sense

use cynic::QueryBuilder;
use cynic_introspection::IntrospectionQuery;
use expect_test::expect;
use indoc::indoc;
use integration_tests::with_mongodb;

#[test]
fn nested_sort_schema() {
    // Some change I made broke the schema shape for this specific case, so
    // here's a test to help me figure it out
    let schema = indoc! {r#"
        type Age {
          number: Int! @map(name: "real_number")
        }

        type User @model(connector: "test", collection: "users") {
          age: Age!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        api.engine().execute(IntrospectionQuery::build(())).await
    });

    let sdl = serde_json::from_str::<cynic::GraphQlResponse<IntrospectionQuery>>(&response)
        .unwrap()
        .data
        .unwrap()
        .into_schema()
        .unwrap()
        .to_sdl();

    let expected = expect![[r#"
        type Age {
          number: Int!
        }

        """
        Age input type.
        """
        input AgeInput {
          number: Int!
        }

        input AgeOrderByInput {
          number: MongoOrderByDirection
        }

        input AgeUpdateInput {
          number: MongoDBRequiredIntUpdateInput
        }

        input MongoDBAgeSearchFilterInput {
          number: MongoDBIntSearchFilterInput
        }

        """
        Search filter input for ID type.
        """
        input MongoDBIDSearchFilterInput {
          """
          The value is exactly the one given
          """
          eq: ID
          """
          The value exists in the document and is not null.
          """
          exists: Boolean
          """
          The value is greater than the one given
          """
          gt: ID
          """
          The value is greater than, or equal to the one given
          """
          gte: ID
          """
          The value is in the given array of values
          """
          in: [ID]
          """
          The value is less than the one given
          """
          lt: ID
          """
          The value is less than, or equal to the one given
          """
          lte: ID
          """
          The value is not the one given
          """
          ne: ID
          """
          The value is not in the given array of values
          """
          nin: [ID]
          """
          The value does not match the filters.
          """
          not: MongoDBIDSearchFilterInput
        }

        """
        Search filter input for Int type.
        """
        input MongoDBIntSearchFilterInput {
          """
          The value is exactly the one given
          """
          eq: Int
          """
          The value exists in the document and is not null.
          """
          exists: Boolean
          """
          The value is greater than the one given
          """
          gt: Int
          """
          The value is greater than, or equal to the one given
          """
          gte: Int
          """
          The value is in the given array of values
          """
          in: [Int]
          """
          The value is less than the one given
          """
          lt: Int
          """
          The value is less than, or equal to the one given
          """
          lte: Int
          """
          The value is not the one given
          """
          ne: Int
          """
          The value is not in the given array of values
          """
          nin: [Int]
          """
          The value does not match the filters.
          """
          not: MongoDBIntSearchFilterInput
        }

        """
        Update input for Int type.
        """
        input MongoDBRequiredIntUpdateInput {
          """
          Increments the value of the field by the specified amount.
          """
          increment: Int
          """
          Only updates the field if the specified value is greater than the existing field value.
          """
          maximum: Int
          """
          Only updates the field if the specified value is less than the existing field value.
          """
          minimum: Int
          """
          Multiplies the value of the field by the specified amount.
          """
          multiply: Int
          """
          Replaces the value of a field with the specified value.
          """
          set: Int
        }

        enum MongoOrderByDirection {
          ASC
          DESC
        }

        type Mutation {
          """
          Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
          Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
          Delete a unique User
          """
          userDelete(by: UserByInput!): UserDeletePayload
          """
          Delete many Users
          """
          userDeleteMany(filter: UserCollection!): UserDeletePayload
          """
          Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
          Update many Users
          """
          userUpdateMany(filter: UserCollection!, input: UserUpdateInput!): UserUpdatePayload
        }

        type PageInfo {
          endCursor: String
          hasNextPage: Boolean!
          hasPreviousPage: Boolean!
          startCursor: String
        }

        type Query {
          """
          Query a single User by a field
          """
          user(
            """
            The field and value by which to query the User
            """
            by: UserByInput!
          ): User
          """
          Paginated query to fetch the whole list of User
          """
          userCollection(
            after: String
            before: String
            filter: UserCollection
            first: Int
            last: Int
            orderBy: [UserOrderByInput]
          ): UserConnection
        }

        type User {
          age: Age!
          """
          Unique identifier
          """
          id: ID!
        }

        input UserByInput {
          id: ID
        }

        input UserCollection {
          """
          All of the filters must match
          """
          ALL: [UserCollection]
          """
          At least one of the filters must match
          """
          ANY: [UserCollection]
          """
          None of the filters must match
          """
          NONE: [UserCollection]
          age: MongoDBAgeSearchFilterInput
          id: MongoDBIDSearchFilterInput
        }

        type UserConnection {
          edges: [UserEdge]
          """
          Information to aid in pagination
          """
          pageInfo: PageInfo!
        }

        """
        Input to create a User
        """
        input UserCreateInput {
          age: AgeInput!
          id: ID
        }

        type UserCreateManyPayload {
          insertedIds: [ID]
        }

        type UserCreatePayload {
          insertedId: ID
        }

        type UserDeletePayload {
          deletedCount: Int
        }

        type UserEdge {
          cursor: String!
          node: User!
        }

        input UserOrderByInput {
          age: AgeOrderByInput
          id: MongoOrderByDirection
        }

        input UserUpdateInput {
          age: AgeUpdateInput
        }

        type UserUpdatePayload {
          matchedCount: Int
          modifiedCount: Int
        }

    "#]];

    expected.assert_eq(&sdl);
}
