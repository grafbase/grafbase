use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::{introspect_namespaced_postgres, introspect_postgres};

#[test]
fn table_with_serial_primary_key() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserBatchMutation
          """
            Create a User
          """
          userCreate(input: UserInput!): UserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): UserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
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
          user(by: UserByInput!): User
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: UserCollection, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput]): UserConnection
        }

        type User {
          id: Int!
        }

        type UserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserByInput {
          id: Int
        }

        input UserCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserCollection]
          """
            None of the filters must match
          """ NONE: [UserCollection]
          """
            At least one of the filters must match
          """ ANY: [UserCollection]
        }

        type UserConnection {
          edges: [UserEdge]!
          pageInfo: PageInfo!
        }

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int
        }

        type UserMutation {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserMutationCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserMutationCollection]
          """
            None of the filters must match
          """ NONE: [UserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [UserMutationCollection]
        }

        input UserOrderByInput {
          id: OrderByDirection
        }

        type UserReturning {
          id: Int!
        }

        input UserUpdateInput {
          id: IntUpdateInput
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_enum_field() {
    let response = introspect_postgres(|api| async move {
        let r#type = indoc! {r"
            CREATE TYPE street_light AS ENUM ('red', 'yellow', 'green');
        "};

        api.execute_sql(r#type).await;

        let table = indoc! {r#"
            CREATE TABLE "A" (
              id INT PRIMARY KEY,
              val street_light NOT NULL
            );
        "#};

        api.execute_sql(table).await;
    });

    let expected = expect![[r#"
        type A {
          id: Int!
          val: StreetLight!
        }

        type ABatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [AReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input AByInput {
          id: Int
        }

        input ACollection {
          id: IntSearchFilterInput
          val: StreetLightSearchFilterInput
          """
            All of the filters must match
          """ ALL: [ACollection]
          """
            None of the filters must match
          """ NONE: [ACollection]
          """
            At least one of the filters must match
          """ ANY: [ACollection]
        }

        type AConnection {
          edges: [AEdge]!
          pageInfo: PageInfo!
        }

        type AEdge {
          node: A!
          cursor: String!
        }

        input AInput {
          id: Int!
          val: StreetLight!
        }

        type AMutation {
          """
            Returned item from the mutation.
          """
          returning: AReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input AMutationCollection {
          id: IntSearchFilterInput
          val: StreetLightSearchFilterInput
          """
            All of the filters must match
          """ ALL: [AMutationCollection]
          """
            None of the filters must match
          """ NONE: [AMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [AMutationCollection]
        }

        input AOrderByInput {
          id: OrderByDirection
          val: OrderByDirection
        }

        type AReturning {
          id: Int!
          val: StreetLight!
        }

        input AUpdateInput {
          id: IntUpdateInput
          val: StreetLightUpdateInput
        }

        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type Mutation {
          """
            Delete a unique A by a field or combination of fields
          """
          aDelete(by: AByInput!): AMutation
          """
            Delete multiple rows of A by a filter
          """
          aDeleteMany(filter: AMutationCollection!): ABatchMutation
          """
            Create a A
          """
          aCreate(input: AInput!): AMutation
          """
            Create multiple As
          """
          aCreateMany(input: [AInput!]!): ABatchMutation
          """
            Update a unique A
          """
          aUpdate(by: AByInput!, input: AUpdateInput!): AMutation
          """
            Update multiple As
          """
          aUpdateMany(filter: AMutationCollection!, input: AUpdateInput!): ABatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
        }

        type PageInfo {
          hasPreviousPage: Boolean!
          hasNextPage: Boolean!
          startCursor: String
          endCursor: String
        }

        type Query {
          """
            Query a single A by a field
          """
          a(by: AByInput!): A
          """
            Paginated query to fetch the whole list of A
          """
          aCollection(filter: ACollection, first: Int, last: Int, before: String, after: String, orderBy: [AOrderByInput]): AConnection
        }

        enum StreetLight {
          RED
          YELLOW
          GREEN
        }

        """
          Search filter input for StreetLight type.
        """
        input StreetLightSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: StreetLight
          """
            The value is not the one given
          """ ne: StreetLight
          """
            The value is greater than the one given
          """ gt: StreetLight
          """
            The value is less than the one given
          """ lt: StreetLight
          """
            The value is greater than, or equal to the one given
          """ gte: StreetLight
          """
            The value is less than, or equal to the one given
          """ lte: StreetLight
          """
            The value is in the given array of values
          """ in: [StreetLight]
          """
            The value is not in the given array of values
          """ nin: [StreetLight]
          not: StreetLightSearchFilterInput
        }

        """
          Update input for StreetLight type.
        """
        input StreetLightUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: StreetLight
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_int_primary_key() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserBatchMutation
          """
            Create a User
          """
          userCreate(input: UserInput!): UserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): UserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
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
          user(by: UserByInput!): User
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: UserCollection, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput]): UserConnection
        }

        type User {
          id: Int!
        }

        type UserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserByInput {
          id: Int
        }

        input UserCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserCollection]
          """
            None of the filters must match
          """ NONE: [UserCollection]
          """
            At least one of the filters must match
          """ ANY: [UserCollection]
        }

        type UserConnection {
          edges: [UserEdge]!
          pageInfo: PageInfo!
        }

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int!
        }

        type UserMutation {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserMutationCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserMutationCollection]
          """
            None of the filters must match
          """ NONE: [UserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [UserMutationCollection]
        }

        input UserOrderByInput {
          id: OrderByDirection
        }

        type UserReturning {
          id: Int!
        }

        input UserUpdateInput {
          id: IntUpdateInput
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_int_unique() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserBatchMutation
          """
            Create a User
          """
          userCreate(input: UserInput!): UserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): UserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
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
          user(by: UserByInput!): User
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: UserCollection, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput]): UserConnection
        }

        type User {
          id: Int!
        }

        type UserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserByInput {
          id: Int
        }

        input UserCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserCollection]
          """
            None of the filters must match
          """ NONE: [UserCollection]
          """
            At least one of the filters must match
          """ ANY: [UserCollection]
        }

        type UserConnection {
          edges: [UserEdge]!
          pageInfo: PageInfo!
        }

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int!
        }

        type UserMutation {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserMutationCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserMutationCollection]
          """
            None of the filters must match
          """ NONE: [UserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [UserMutationCollection]
        }

        input UserOrderByInput {
          id: OrderByDirection
        }

        type UserReturning {
          id: Int!
        }

        input UserUpdateInput {
          id: IntUpdateInput
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_serial_primary_key_string_unique() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserBatchMutation
          """
            Create a User
          """
          userCreate(input: UserInput!): UserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): UserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
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
          user(by: UserByInput!): User
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: UserCollection, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput]): UserConnection
        }

        """
          Search filter input for String type.
        """
        input StringSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: String
          """
            The value is not the one given
          """ ne: String
          """
            The value is greater than the one given
          """ gt: String
          """
            The value is less than the one given
          """ lt: String
          """
            The value is greater than, or equal to the one given
          """ gte: String
          """
            The value is less than, or equal to the one given
          """ lte: String
          """
            The value is in the given array of values
          """ in: [String]
          """
            The value is not in the given array of values
          """ nin: [String]
          not: StringSearchFilterInput
        }

        """
          Update input for String type.
        """
        input StringUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: String
        }

        type User {
          id: Int!
          email: String!
        }

        type UserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserByInput {
          email: String
          id: Int
        }

        input UserCollection {
          id: IntSearchFilterInput
          email: StringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserCollection]
          """
            None of the filters must match
          """ NONE: [UserCollection]
          """
            At least one of the filters must match
          """ ANY: [UserCollection]
        }

        type UserConnection {
          edges: [UserEdge]!
          pageInfo: PageInfo!
        }

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int
          email: String!
        }

        type UserMutation {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserMutationCollection {
          id: IntSearchFilterInput
          email: StringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserMutationCollection]
          """
            None of the filters must match
          """ NONE: [UserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [UserMutationCollection]
        }

        input UserOrderByInput {
          id: OrderByDirection
          email: OrderByDirection
        }

        type UserReturning {
          id: Int!
          email: String!
        }

        input UserUpdateInput {
          id: IntUpdateInput
          email: StringUpdateInput
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_composite_primary_key() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                CONSTRAINT "User_pkey" PRIMARY KEY (name, email)
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserBatchMutation
          """
            Create a User
          """
          userCreate(input: UserInput!): UserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): UserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
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
          user(by: UserByInput!): User
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: UserCollection, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput]): UserConnection
        }

        """
          Search filter input for String type.
        """
        input StringSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: String
          """
            The value is not the one given
          """ ne: String
          """
            The value is greater than the one given
          """ gt: String
          """
            The value is less than the one given
          """ lt: String
          """
            The value is greater than, or equal to the one given
          """ gte: String
          """
            The value is less than, or equal to the one given
          """ lte: String
          """
            The value is in the given array of values
          """ in: [String]
          """
            The value is not in the given array of values
          """ nin: [String]
          not: StringSearchFilterInput
        }

        """
          Update input for String type.
        """
        input StringUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: String
        }

        type User {
          name: String!
          email: String!
        }

        type UserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserByInput {
          nameEmail: UserNameEmailInput
        }

        input UserCollection {
          name: StringSearchFilterInput
          email: StringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserCollection]
          """
            None of the filters must match
          """ NONE: [UserCollection]
          """
            At least one of the filters must match
          """ ANY: [UserCollection]
        }

        type UserConnection {
          edges: [UserEdge]!
          pageInfo: PageInfo!
        }

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          name: String!
          email: String!
        }

        type UserMutation {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserMutationCollection {
          name: StringSearchFilterInput
          email: StringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserMutationCollection]
          """
            None of the filters must match
          """ NONE: [UserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [UserMutationCollection]
        }

        input UserNameEmailInput {
          name: String!
          email: String!
        }

        input UserOrderByInput {
          name: OrderByDirection
          email: OrderByDirection
        }

        type UserReturning {
          name: String!
          email: String!
        }

        input UserUpdateInput {
          name: StringUpdateInput
          email: StringUpdateInput
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn two_schemas_same_table_name() {
    let response = introspect_postgres(|api| async move {
        api.execute_sql(r"CREATE SCHEMA private").await;

        let schema = indoc! {r#"
            CREATE TABLE private."User" (
                id SERIAL PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;

        let schema = indoc! {r#"
            CREATE TABLE public."User" (
                id SERIAL PRIMARY KEY
            )    
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type Mutation {
          """
            Delete a unique PrivateUser by a field or combination of fields
          """
          privateUserDelete(by: PrivateUserByInput!): PrivateUserMutation
          """
            Delete multiple rows of PrivateUser by a filter
          """
          privateUserDeleteMany(filter: PrivateUserMutationCollection!): PrivateUserBatchMutation
          """
            Create a PrivateUser
          """
          privateUserCreate(input: PrivateUserInput!): PrivateUserMutation
          """
            Create multiple PrivateUsers
          """
          privateUserCreateMany(input: [PrivateUserInput!]!): PrivateUserBatchMutation
          """
            Update a unique PrivateUser
          """
          privateUserUpdate(by: PrivateUserByInput!, input: PrivateUserUpdateInput!): PrivateUserMutation
          """
            Update multiple PrivateUsers
          """
          privateUserUpdateMany(filter: PrivateUserMutationCollection!, input: PrivateUserUpdateInput!): PrivateUserBatchMutation
          """
            Delete a unique PublicUser by a field or combination of fields
          """
          publicUserDelete(by: PublicUserByInput!): PublicUserMutation
          """
            Delete multiple rows of PublicUser by a filter
          """
          publicUserDeleteMany(filter: PublicUserMutationCollection!): PublicUserBatchMutation
          """
            Create a PublicUser
          """
          publicUserCreate(input: PublicUserInput!): PublicUserMutation
          """
            Create multiple PublicUsers
          """
          publicUserCreateMany(input: [PublicUserInput!]!): PublicUserBatchMutation
          """
            Update a unique PublicUser
          """
          publicUserUpdate(by: PublicUserByInput!, input: PublicUserUpdateInput!): PublicUserMutation
          """
            Update multiple PublicUsers
          """
          publicUserUpdateMany(filter: PublicUserMutationCollection!, input: PublicUserUpdateInput!): PublicUserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
        }

        type PageInfo {
          hasPreviousPage: Boolean!
          hasNextPage: Boolean!
          startCursor: String
          endCursor: String
        }

        type PrivateUser {
          id: Int!
        }

        type PrivateUserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [PrivateUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input PrivateUserByInput {
          id: Int
        }

        input PrivateUserCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PrivateUserCollection]
          """
            None of the filters must match
          """ NONE: [PrivateUserCollection]
          """
            At least one of the filters must match
          """ ANY: [PrivateUserCollection]
        }

        type PrivateUserConnection {
          edges: [PrivateUserEdge]!
          pageInfo: PageInfo!
        }

        type PrivateUserEdge {
          node: PrivateUser!
          cursor: String!
        }

        input PrivateUserInput {
          id: Int
        }

        type PrivateUserMutation {
          """
            Returned item from the mutation.
          """
          returning: PrivateUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input PrivateUserMutationCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PrivateUserMutationCollection]
          """
            None of the filters must match
          """ NONE: [PrivateUserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [PrivateUserMutationCollection]
        }

        input PrivateUserOrderByInput {
          id: OrderByDirection
        }

        type PrivateUserReturning {
          id: Int!
        }

        input PrivateUserUpdateInput {
          id: IntUpdateInput
        }

        type PublicUser {
          id: Int!
        }

        type PublicUserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [PublicUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input PublicUserByInput {
          id: Int
        }

        input PublicUserCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PublicUserCollection]
          """
            None of the filters must match
          """ NONE: [PublicUserCollection]
          """
            At least one of the filters must match
          """ ANY: [PublicUserCollection]
        }

        type PublicUserConnection {
          edges: [PublicUserEdge]!
          pageInfo: PageInfo!
        }

        type PublicUserEdge {
          node: PublicUser!
          cursor: String!
        }

        input PublicUserInput {
          id: Int
        }

        type PublicUserMutation {
          """
            Returned item from the mutation.
          """
          returning: PublicUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input PublicUserMutationCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PublicUserMutationCollection]
          """
            None of the filters must match
          """ NONE: [PublicUserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [PublicUserMutationCollection]
        }

        input PublicUserOrderByInput {
          id: OrderByDirection
        }

        type PublicUserReturning {
          id: Int!
        }

        input PublicUserUpdateInput {
          id: IntUpdateInput
        }

        type Query {
          """
            Query a single PrivateUser by a field
          """
          privateUser(by: PrivateUserByInput!): PrivateUser
          """
            Paginated query to fetch the whole list of PrivateUser
          """
          privateUserCollection(filter: PrivateUserCollection, first: Int, last: Int, before: String, after: String, orderBy: [PrivateUserOrderByInput]): PrivateUserConnection
          """
            Query a single PublicUser by a field
          """
          publicUser(by: PublicUserByInput!): PublicUser
          """
            Paginated query to fetch the whole list of PublicUser
          """
          publicUserCollection(filter: PublicUserCollection, first: Int, last: Int, before: String, after: String, orderBy: [PublicUserOrderByInput]): PublicUserConnection
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_serial_primary_key_namespaced() {
    let response = introspect_namespaced_postgres("Neon", |api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id SERIAL PRIMARY KEY
            )
        "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        type Mutation {
          neon: NeonMutation
        }

        """
          Search filter input for Int type.
        """
        input NeonIntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: NeonIntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input NeonIntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type NeonMutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: NeonUserByInput!): NeonUserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: NeonUserMutationCollection!): NeonUserBatchMutation
          """
            Create a User
          """
          userCreate(input: NeonUserInput!): NeonUserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [NeonUserInput!]!): NeonUserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: NeonUserByInput!, input: NeonUserUpdateInput!): NeonUserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: NeonUserMutationCollection!, input: NeonUserUpdateInput!): NeonUserBatchMutation
        }

        enum NeonOrderByDirection {
          ASC
          DESC
        }

        type NeonPageInfo {
          hasPreviousPage: Boolean!
          hasNextPage: Boolean!
          startCursor: String
          endCursor: String
        }

        type NeonQuery {
          """
            Query a single NeonUser by a field
          """
          user(by: NeonUserByInput!): NeonUser
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: NeonUserCollection, first: Int, last: Int, before: String, after: String, orderBy: [NeonUserOrderByInput]): NeonUserConnection
        }

        type NeonUser {
          id: Int!
        }

        type NeonUserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [NeonUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input NeonUserByInput {
          id: Int
        }

        input NeonUserCollection {
          id: NeonIntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [NeonUserCollection]
          """
            None of the filters must match
          """ NONE: [NeonUserCollection]
          """
            At least one of the filters must match
          """ ANY: [NeonUserCollection]
        }

        type NeonUserConnection {
          edges: [NeonUserEdge]!
          pageInfo: NeonPageInfo!
        }

        type NeonUserEdge {
          node: NeonUser!
          cursor: String!
        }

        input NeonUserInput {
          id: Int
        }

        type NeonUserMutation {
          """
            Returned item from the mutation.
          """
          returning: NeonUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input NeonUserMutationCollection {
          id: NeonIntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [NeonUserMutationCollection]
          """
            None of the filters must match
          """ NONE: [NeonUserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [NeonUserMutationCollection]
        }

        input NeonUserOrderByInput {
          id: NeonOrderByDirection
        }

        type NeonUserReturning {
          id: Int!
        }

        input NeonUserUpdateInput {
          id: NeonIntUpdateInput
        }

        type Query {
          neon: NeonQuery
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_an_array_column() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
           CREATE TABLE "User" (
               id SERIAL PRIMARY KEY,
               name INT[] NOT NULL 
           );
       "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        """
          Search filter input for Int type.
        """
        input IntArraySearchFilterInput {
          """
            The value is exactly the one given
          """ eq: [Int]
          """
            The value is not the one given
          """ ne: [Int]
          """
            The value is greater than the one given
          """ gt: [Int]
          """
            The value is less than the one given
          """ lt: [Int]
          """
            The value is greater than, or equal to the one given
          """ gte: [Int]
          """
            The value is less than, or equal to the one given
          """ lte: [Int]
          """
            The value is in the given array of values
          """ in: [[Int]]
          """
            The value is not in the given array of values
          """ nin: [[Int]]
          """
            The column contains all elements from the given array.
          """ contains: [Int]
          """
            The given array contains all elements from the column.
          """ contained: [Int]
          """
            Do the arrays have any elements in common.
          """ overlaps: [Int]
          not: IntArraySearchFilterInput
        }

        """
          Update input for Int array type.
        """
        input IntArrayUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: [Int]
          """
            Append an array value to the column.
          """ append: [Int]
          """
            Prepend an array value to the column.
          """ prepend: [Int]
        }

        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserBatchMutation
          """
            Create a User
          """
          userCreate(input: UserInput!): UserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): UserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
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
          user(by: UserByInput!): User
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: UserCollection, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput]): UserConnection
        }

        type User {
          id: Int!
          name: [Int]!
        }

        type UserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserByInput {
          id: Int
        }

        input UserCollection {
          id: IntSearchFilterInput
          name: IntArraySearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserCollection]
          """
            None of the filters must match
          """ NONE: [UserCollection]
          """
            At least one of the filters must match
          """ ANY: [UserCollection]
        }

        type UserConnection {
          edges: [UserEdge]!
          pageInfo: PageInfo!
        }

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int
          name: [Int]!
        }

        type UserMutation {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserMutationCollection {
          id: IntSearchFilterInput
          name: IntArraySearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserMutationCollection]
          """
            None of the filters must match
          """ NONE: [UserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [UserMutationCollection]
        }

        input UserOrderByInput {
          id: OrderByDirection
          name: OrderByDirection
        }

        type UserReturning {
          id: Int!
          name: [Int]!
        }

        input UserUpdateInput {
          id: IntUpdateInput
          name: IntArrayUpdateInput
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_jsonb_column() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
           CREATE TABLE "User" (
               id SERIAL PRIMARY KEY,
               name JSONB NOT NULL 
           );
       "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        """
          A JSON Value
        """
        scalar JSON

        """
          Search filter input for JSON type.
        """
        input JsonSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: JSON
          """
            The value is not the one given
          """ ne: JSON
          """
            The value is greater than the one given
          """ gt: JSON
          """
            The value is less than the one given
          """ lt: JSON
          """
            The value is greater than, or equal to the one given
          """ gte: JSON
          """
            The value is less than, or equal to the one given
          """ lte: JSON
          """
            The value is in the given array of values
          """ in: [JSON]
          """
            The value is not in the given array of values
          """ nin: [JSON]
          not: JsonSearchFilterInput
        }

        """
          Update input for JSON type.
        """
        input JsonUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: JSON
          """
            Append JSON value to the column.
          """ append: JSON
          """
            Prepend JSON value to the column.
          """ prepend: JSON
          """
            Deletes a key (and its value) from a JSON object, or matching string value(s) from a JSON array.
          """ deleteKey: String
          """
            Deletes the array element with specified index (negative integers count from the end). Throws an error if JSON value is not an array.
          """ deleteElem: Int
          """
            Deletes the field or array element at the specified path, where path elements can be either field keys or array indexes.
          """ deleteAtPath: [String!]
        }

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserBatchMutation
          """
            Create a User
          """
          userCreate(input: UserInput!): UserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): UserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
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
          user(by: UserByInput!): User
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: UserCollection, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput]): UserConnection
        }

        type User {
          id: Int!
          name: JSON!
        }

        type UserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserByInput {
          id: Int
        }

        input UserCollection {
          id: IntSearchFilterInput
          name: JsonSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserCollection]
          """
            None of the filters must match
          """ NONE: [UserCollection]
          """
            At least one of the filters must match
          """ ANY: [UserCollection]
        }

        type UserConnection {
          edges: [UserEdge]!
          pageInfo: PageInfo!
        }

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int
          name: JSON!
        }

        type UserMutation {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserMutationCollection {
          id: IntSearchFilterInput
          name: JsonSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserMutationCollection]
          """
            None of the filters must match
          """ NONE: [UserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [UserMutationCollection]
        }

        input UserOrderByInput {
          id: OrderByDirection
          name: OrderByDirection
        }

        type UserReturning {
          id: Int!
          name: JSON!
        }

        input UserUpdateInput {
          id: IntUpdateInput
          name: JsonUpdateInput
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_json_column() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
           CREATE TABLE "User" (
               id SERIAL PRIMARY KEY,
               name JSON NOT NULL 
           );
       "#};

        api.execute_sql(schema).await;
    });

    let expected = expect![[r#"
        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        """
          A JSON Value
        """
        scalar JSON

        """
          Search filter input for JSON type.
        """
        input JsonSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: JSON
          """
            The value is not the one given
          """ ne: JSON
          """
            The value is greater than the one given
          """ gt: JSON
          """
            The value is less than the one given
          """ lt: JSON
          """
            The value is greater than, or equal to the one given
          """ gte: JSON
          """
            The value is less than, or equal to the one given
          """ lte: JSON
          """
            The value is in the given array of values
          """ in: [JSON]
          """
            The value is not in the given array of values
          """ nin: [JSON]
          not: JsonSearchFilterInput
        }

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserBatchMutation
          """
            Create a User
          """
          userCreate(input: UserInput!): UserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): UserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
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
          user(by: UserByInput!): User
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: UserCollection, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput]): UserConnection
        }

        """
          Update input for SimpleJSON type.
        """
        input SimpleJSONUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: SimpleJSON
        }

        type User {
          id: Int!
          name: JSON!
        }

        type UserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserByInput {
          id: Int
        }

        input UserCollection {
          id: IntSearchFilterInput
          name: JsonSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserCollection]
          """
            None of the filters must match
          """ NONE: [UserCollection]
          """
            At least one of the filters must match
          """ ANY: [UserCollection]
        }

        type UserConnection {
          edges: [UserEdge]!
          pageInfo: PageInfo!
        }

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int
          name: JSON!
        }

        type UserMutation {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserMutationCollection {
          id: IntSearchFilterInput
          name: JsonSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserMutationCollection]
          """
            None of the filters must match
          """ NONE: [UserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [UserMutationCollection]
        }

        input UserOrderByInput {
          id: OrderByDirection
          name: OrderByDirection
        }

        type UserReturning {
          id: Int!
          name: JSON!
        }

        input UserUpdateInput {
          id: IntUpdateInput
          name: SimpleJSONUpdateInput
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn two_tables_with_single_column_foreign_key() {
    let response = introspect_postgres(|api| async move {
        let create_user = indoc! {r#"
           CREATE TABLE "User" (
               id SERIAL PRIMARY KEY,
               name VARCHAR(255) NOT NULL 
           );
       "#};

        api.execute_sql(create_user).await;

        let create_blog = indoc! {r#"
            CREATE TABLE "Blog" (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT,
                user_id INT NOT NULL,
                CONSTRAINT "Blog_User" FOREIGN KEY (user_id) REFERENCES "User"(id)
            )    
        "#};

        api.execute_sql(create_blog).await;
    });

    let expected = expect![[r#"
        type Blog {
          id: Int!
          title: String!
          content: String
          userId: Int!
          user: User!
        }

        type BlogBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [BlogReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input BlogByInput {
          id: Int
        }

        input BlogCollection {
          id: IntSearchFilterInput
          title: StringSearchFilterInput
          content: StringSearchFilterInput
          userId: IntSearchFilterInput
          user: UserCollection
          """
            All of the filters must match
          """ ALL: [BlogCollection]
          """
            None of the filters must match
          """ NONE: [BlogCollection]
          """
            At least one of the filters must match
          """ ANY: [BlogCollection]
        }

        input BlogCollectionContains {
          contains: BlogCollection
        }

        type BlogConnection {
          edges: [BlogEdge]!
          pageInfo: PageInfo!
        }

        type BlogEdge {
          node: Blog!
          cursor: String!
        }

        input BlogInput {
          id: Int
          title: String!
          content: String
          userId: Int!
        }

        type BlogMutation {
          """
            Returned item from the mutation.
          """
          returning: BlogReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input BlogMutationCollection {
          id: IntSearchFilterInput
          title: StringSearchFilterInput
          content: StringSearchFilterInput
          userId: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [BlogMutationCollection]
          """
            None of the filters must match
          """ NONE: [BlogMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [BlogMutationCollection]
        }

        input BlogOrderByInput {
          id: OrderByDirection
          title: OrderByDirection
          content: OrderByDirection
          userId: OrderByDirection
        }

        type BlogReturning {
          id: Int!
          title: String!
          content: String
          userId: Int!
        }

        input BlogUpdateInput {
          id: IntUpdateInput
          title: StringUpdateInput
          content: StringUpdateInput
          userId: IntUpdateInput
        }

        """
          Search filter input for Int type.
        """
        input IntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: IntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input IntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type Mutation {
          """
            Delete a unique Blog by a field or combination of fields
          """
          blogDelete(by: BlogByInput!): BlogMutation
          """
            Delete multiple rows of Blog by a filter
          """
          blogDeleteMany(filter: BlogMutationCollection!): BlogBatchMutation
          """
            Create a Blog
          """
          blogCreate(input: BlogInput!): BlogMutation
          """
            Create multiple Blogs
          """
          blogCreateMany(input: [BlogInput!]!): BlogBatchMutation
          """
            Update a unique Blog
          """
          blogUpdate(by: BlogByInput!, input: BlogUpdateInput!): BlogMutation
          """
            Update multiple Blogs
          """
          blogUpdateMany(filter: BlogMutationCollection!, input: BlogUpdateInput!): BlogBatchMutation
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserMutation
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserBatchMutation
          """
            Create a User
          """
          userCreate(input: UserInput!): UserMutation
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): UserBatchMutation
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserMutation
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserBatchMutation
        }

        enum OrderByDirection {
          ASC
          DESC
        }

        type PageInfo {
          hasPreviousPage: Boolean!
          hasNextPage: Boolean!
          startCursor: String
          endCursor: String
        }

        type Query {
          """
            Query a single Blog by a field
          """
          blog(by: BlogByInput!): Blog
          """
            Paginated query to fetch the whole list of Blog
          """
          blogCollection(filter: BlogCollection, first: Int, last: Int, before: String, after: String, orderBy: [BlogOrderByInput]): BlogConnection
          """
            Query a single User by a field
          """
          user(by: UserByInput!): User
          """
            Paginated query to fetch the whole list of User
          """
          userCollection(filter: UserCollection, first: Int, last: Int, before: String, after: String, orderBy: [UserOrderByInput]): UserConnection
        }

        """
          Search filter input for String type.
        """
        input StringSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: String
          """
            The value is not the one given
          """ ne: String
          """
            The value is greater than the one given
          """ gt: String
          """
            The value is less than the one given
          """ lt: String
          """
            The value is greater than, or equal to the one given
          """ gte: String
          """
            The value is less than, or equal to the one given
          """ lte: String
          """
            The value is in the given array of values
          """ in: [String]
          """
            The value is not in the given array of values
          """ nin: [String]
          not: StringSearchFilterInput
        }

        """
          Update input for String type.
        """
        input StringUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: String
        }

        type User {
          id: Int!
          name: String!
          blogs(first: Int, last: Int, before: String, after: String, orderBy: [BlogOrderByInput!]): BlogConnection
        }

        type UserBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserByInput {
          id: Int
        }

        input UserCollection {
          id: IntSearchFilterInput
          name: StringSearchFilterInput
          blogs: BlogCollectionContains
          """
            All of the filters must match
          """ ALL: [UserCollection]
          """
            None of the filters must match
          """ NONE: [UserCollection]
          """
            At least one of the filters must match
          """ ANY: [UserCollection]
        }

        type UserConnection {
          edges: [UserEdge]!
          pageInfo: PageInfo!
        }

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int
          name: String!
        }

        type UserMutation {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input UserMutationCollection {
          id: IntSearchFilterInput
          name: StringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserMutationCollection]
          """
            None of the filters must match
          """ NONE: [UserMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [UserMutationCollection]
        }

        input UserOrderByInput {
          id: OrderByDirection
          name: OrderByDirection
        }

        type UserReturning {
          id: Int!
          name: String!
        }

        input UserUpdateInput {
          id: IntUpdateInput
          name: StringUpdateInput
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn cedalio_issue_november_2023() {
    let response = introspect_namespaced_postgres("pg", |api| async move {
        let create = indoc! {r"
            CREATE TYPE access_mode AS ENUM ('PUBLIC', 'PUBLIC_READ', 'PRIVATE');
        "};

        api.execute_sql(create).await;

        let create = indoc! {r"
            CREATE TYPE project_status AS ENUM ('CREATED', 'READY', 'FAILED');
        "};

        api.execute_sql(create).await;

        let create = indoc! {r"
            CREATE TABLE networks (
                id SERIAL PRIMARY KEY
            );
        "};

        api.execute_sql(create).await;

        let create = indoc! {r"
            CREATE TABLE projects (
                id SERIAL PRIMARY KEY,
                access_mode access_mode NOT NULL,
                status project_status DEFAULT 'CREATED' NOT NULL,
                network_id INT REFERENCES networks(id)
            );
        "};

        api.execute_sql(create).await;
    });

    let expected = expect![[r#"
        type Mutation {
          pg: PgMutation
        }

        enum PgAccessMode {
          PUBLIC
          PUBLIC_READ
          PRIVATE
        }

        """
          Search filter input for PgAccessMode type.
        """
        input PgAccessModeSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: PgAccessMode
          """
            The value is not the one given
          """ ne: PgAccessMode
          """
            The value is greater than the one given
          """ gt: PgAccessMode
          """
            The value is less than the one given
          """ lt: PgAccessMode
          """
            The value is greater than, or equal to the one given
          """ gte: PgAccessMode
          """
            The value is less than, or equal to the one given
          """ lte: PgAccessMode
          """
            The value is in the given array of values
          """ in: [PgAccessMode]
          """
            The value is not in the given array of values
          """ nin: [PgAccessMode]
          not: PgAccessModeSearchFilterInput
        }

        """
          Update input for PgAccessMode type.
        """
        input PgAccessModeUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: PgAccessMode
        }

        """
          Search filter input for Int type.
        """
        input PgIntSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: Int
          """
            The value is not the one given
          """ ne: Int
          """
            The value is greater than the one given
          """ gt: Int
          """
            The value is less than the one given
          """ lt: Int
          """
            The value is greater than, or equal to the one given
          """ gte: Int
          """
            The value is less than, or equal to the one given
          """ lte: Int
          """
            The value is in the given array of values
          """ in: [Int]
          """
            The value is not in the given array of values
          """ nin: [Int]
          not: PgIntSearchFilterInput
        }

        """
          Update input for Int type.
        """
        input PgIntUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: Int
          """
            Increments the value of the field by the specified amount.
          """ increment: Int
          """
            Decrements the value of the field by the specified amount.
          """ decrement: Int
          """
            Multiplies the value of the field by the specified amount.
          """ multiply: Int
          """
            Divides the value of the field with the given value.
          """ divide: Int
        }

        type PgMutation {
          """
            Delete a unique Networks by a field or combination of fields
          """
          networksDelete(by: PgNetworksByInput!): PgNetworksMutation
          """
            Delete multiple rows of Networks by a filter
          """
          networksDeleteMany(filter: PgNetworksMutationCollection!): PgNetworksBatchMutation
          """
            Create a Networks
          """
          networksCreate(input: PgNetworksInput!): PgNetworksMutation
          """
            Create multiple Networkss
          """
          networksCreateMany(input: [PgNetworksInput!]!): PgNetworksBatchMutation
          """
            Update a unique Networks
          """
          networksUpdate(by: PgNetworksByInput!, input: PgNetworksUpdateInput!): PgNetworksMutation
          """
            Update multiple Networkss
          """
          networksUpdateMany(filter: PgNetworksMutationCollection!, input: PgNetworksUpdateInput!): PgNetworksBatchMutation
          """
            Delete a unique Projects by a field or combination of fields
          """
          projectsDelete(by: PgProjectsByInput!): PgProjectsMutation
          """
            Delete multiple rows of Projects by a filter
          """
          projectsDeleteMany(filter: PgProjectsMutationCollection!): PgProjectsBatchMutation
          """
            Create a Projects
          """
          projectsCreate(input: PgProjectsInput!): PgProjectsMutation
          """
            Create multiple Projectss
          """
          projectsCreateMany(input: [PgProjectsInput!]!): PgProjectsBatchMutation
          """
            Update a unique Projects
          """
          projectsUpdate(by: PgProjectsByInput!, input: PgProjectsUpdateInput!): PgProjectsMutation
          """
            Update multiple Projectss
          """
          projectsUpdateMany(filter: PgProjectsMutationCollection!, input: PgProjectsUpdateInput!): PgProjectsBatchMutation
        }

        type PgNetworks {
          id: Int!
          projects(first: Int, last: Int, before: String, after: String, orderBy: [PgProjectsOrderByInput!]): PgProjectsConnection
        }

        type PgNetworksBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [PgNetworksReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input PgNetworksByInput {
          id: Int
        }

        input PgNetworksCollection {
          id: PgIntSearchFilterInput
          projects: PgProjectsCollectionContains
          """
            All of the filters must match
          """ ALL: [PgNetworksCollection]
          """
            None of the filters must match
          """ NONE: [PgNetworksCollection]
          """
            At least one of the filters must match
          """ ANY: [PgNetworksCollection]
        }

        type PgNetworksConnection {
          edges: [PgNetworksEdge]!
          pageInfo: PgPageInfo!
        }

        type PgNetworksEdge {
          node: PgNetworks!
          cursor: String!
        }

        input PgNetworksInput {
          id: Int
        }

        type PgNetworksMutation {
          """
            Returned item from the mutation.
          """
          returning: PgNetworksReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input PgNetworksMutationCollection {
          id: PgIntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PgNetworksMutationCollection]
          """
            None of the filters must match
          """ NONE: [PgNetworksMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [PgNetworksMutationCollection]
        }

        input PgNetworksOrderByInput {
          id: PgOrderByDirection
        }

        type PgNetworksReturning {
          id: Int!
        }

        input PgNetworksUpdateInput {
          id: PgIntUpdateInput
        }

        enum PgOrderByDirection {
          ASC
          DESC
        }

        type PgPageInfo {
          hasPreviousPage: Boolean!
          hasNextPage: Boolean!
          startCursor: String
          endCursor: String
        }

        enum PgProjectStatus {
          CREATED
          READY
          FAILED
        }

        """
          Search filter input for PgProjectStatus type.
        """
        input PgProjectStatusSearchFilterInput {
          """
            The value is exactly the one given
          """ eq: PgProjectStatus
          """
            The value is not the one given
          """ ne: PgProjectStatus
          """
            The value is greater than the one given
          """ gt: PgProjectStatus
          """
            The value is less than the one given
          """ lt: PgProjectStatus
          """
            The value is greater than, or equal to the one given
          """ gte: PgProjectStatus
          """
            The value is less than, or equal to the one given
          """ lte: PgProjectStatus
          """
            The value is in the given array of values
          """ in: [PgProjectStatus]
          """
            The value is not in the given array of values
          """ nin: [PgProjectStatus]
          not: PgProjectStatusSearchFilterInput
        }

        """
          Update input for PgProjectStatus type.
        """
        input PgProjectStatusUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: PgProjectStatus
        }

        type PgProjects {
          id: Int!
          accessMode: PgAccessMode!
          status: PgProjectStatus!
          networkId: Int
          networks: PgNetworks
        }

        type PgProjectsBatchMutation {
          """
            Returned items from the mutation.
          """
          returning: [PgProjectsReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input PgProjectsByInput {
          id: Int
        }

        input PgProjectsCollection {
          id: PgIntSearchFilterInput
          accessMode: PgAccessModeSearchFilterInput
          status: PgProjectStatusSearchFilterInput
          networkId: PgIntSearchFilterInput
          networks: PgNetworksCollection
          """
            All of the filters must match
          """ ALL: [PgProjectsCollection]
          """
            None of the filters must match
          """ NONE: [PgProjectsCollection]
          """
            At least one of the filters must match
          """ ANY: [PgProjectsCollection]
        }

        input PgProjectsCollectionContains {
          contains: PgProjectsCollection
        }

        type PgProjectsConnection {
          edges: [PgProjectsEdge]!
          pageInfo: PgPageInfo!
        }

        type PgProjectsEdge {
          node: PgProjects!
          cursor: String!
        }

        input PgProjectsInput {
          id: Int
          accessMode: PgAccessMode!
          status: PgProjectStatus
          networkId: Int
        }

        type PgProjectsMutation {
          """
            Returned item from the mutation.
          """
          returning: PgProjectsReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        input PgProjectsMutationCollection {
          id: PgIntSearchFilterInput
          accessMode: PgAccessModeSearchFilterInput
          status: PgProjectStatusSearchFilterInput
          networkId: PgIntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PgProjectsMutationCollection]
          """
            None of the filters must match
          """ NONE: [PgProjectsMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [PgProjectsMutationCollection]
        }

        input PgProjectsOrderByInput {
          id: PgOrderByDirection
          accessMode: PgOrderByDirection
          status: PgOrderByDirection
          networkId: PgOrderByDirection
        }

        type PgProjectsReturning {
          id: Int!
          accessMode: PgAccessMode!
          status: PgProjectStatus!
          networkId: Int
        }

        input PgProjectsUpdateInput {
          id: PgIntUpdateInput
          accessMode: PgAccessModeUpdateInput
          status: PgProjectStatusUpdateInput
          networkId: PgIntUpdateInput
        }

        type PgQuery {
          """
            Query a single PgNetworks by a field
          """
          networks(by: PgNetworksByInput!): PgNetworks
          """
            Paginated query to fetch the whole list of Networks
          """
          networksCollection(filter: PgNetworksCollection, first: Int, last: Int, before: String, after: String, orderBy: [PgNetworksOrderByInput]): PgNetworksConnection
          """
            Query a single PgProjects by a field
          """
          projects(by: PgProjectsByInput!): PgProjects
          """
            Paginated query to fetch the whole list of Projects
          """
          projectsCollection(filter: PgProjectsCollection, first: Int, last: Int, before: String, after: String, orderBy: [PgProjectsOrderByInput]): PgProjectsConnection
        }

        type Query {
          pg: PgQuery
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}
