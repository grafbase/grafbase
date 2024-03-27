//! Tests that the generated schema for mongo makes sense

use cynic::QueryBuilder;
use cynic_introspection::IntrospectionQuery;
use expect_test::expect;
use indoc::indoc;
use integration_tests::{federation::GraphqlResponse, with_mongodb, with_namespaced_mongodb, ResponseExt};
use serde_json::json;

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
          The value is not the one given
          """
          ne: ID
          """
          The value is greater than the one given
          """
          gt: ID
          """
          The value is less than the one given
          """
          lt: ID
          """
          The value is greater than, or equal to the one given
          """
          gte: ID
          """
          The value is less than, or equal to the one given
          """
          lte: ID
          """
          The value does not match the filters.
          """
          not: MongoDBIDSearchFilterInput
          """
          The value is in the given array of values
          """
          in: [ID]
          """
          The value is not in the given array of values
          """
          nin: [ID]
          """
          The value exists in the document and is not null.
          """
          exists: Boolean
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
          The value is not the one given
          """
          ne: Int
          """
          The value is greater than the one given
          """
          gt: Int
          """
          The value is less than the one given
          """
          lt: Int
          """
          The value is greater than, or equal to the one given
          """
          gte: Int
          """
          The value is less than, or equal to the one given
          """
          lte: Int
          """
          The value does not match the filters.
          """
          not: MongoDBIntSearchFilterInput
          """
          The value is in the given array of values
          """
          in: [Int]
          """
          The value is not in the given array of values
          """
          nin: [Int]
          """
          The value exists in the document and is not null.
          """
          exists: Boolean
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
          Only updates the field if the specified value is less than the existing field value.
          """
          minimum: Int
          """
          Only updates the field if the specified value is greater than the existing field value.
          """
          maximum: Int
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
          Delete a unique User
          """
          userDelete(by: UserByInput!): UserDeletePayload
          """
          Delete many Users
          """
          userDeleteMany(filter: UserCollection!): UserDeletePayload
          """
          Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
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
          hasPreviousPage: Boolean!
          hasNextPage: Boolean!
          startCursor: String
          endCursor: String
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
            filter: UserCollection
            first: Int
            last: Int
            before: String
            after: String
            orderBy: [UserOrderByInput]
          ): UserConnection
        }

        type User {
          """
          Unique identifier
          """
          id: ID!
          age: Age!
        }

        input UserByInput {
          id: ID
        }

        input UserCollection {
          id: MongoDBIDSearchFilterInput
          """
          All of the filters must match
          """
          ALL: [UserCollection]
          """
          None of the filters must match
          """
          NONE: [UserCollection]
          """
          At least one of the filters must match
          """
          ANY: [UserCollection]
          age: MongoDBAgeSearchFilterInput
        }

        type UserConnection {
          """
          Information to aid in pagination
          """
          pageInfo: PageInfo!
          edges: [UserEdge]
        }

        """
        Input to create a User
        """
        input UserCreateInput {
          id: ID
          age: AgeInput!
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
          node: User!
          cursor: String!
        }

        input UserOrderByInput {
          id: MongoOrderByDirection
          age: AgeOrderByInput
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
