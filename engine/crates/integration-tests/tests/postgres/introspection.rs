use expect_test::expect;
use indoc::indoc;
use integration_tests::postgres::{introspect_namespaced_postgres, introspect_postgres};

#[test]
fn table_with_generated_always_identity_primary_key() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY
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

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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

        input UserCreateInput

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        input UserUpdateInput

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_generated_by_default_identity_primary_key() {
    let response = introspect_postgres(|api| async move {
        let schema = indoc! {r#"
            CREATE TABLE "User" (
                id INT PRIMARY KEY GENERATED BY DEFAULT AS IDENTITY
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
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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

        input UserCreateInput {
          id: Int
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];
    expected.assert_eq(&response);
}

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
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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

        input UserCreateInput {
          id: Int
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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

        input ACreateInput {
          id: Int!
          val: StreetLight!
        }

        type ACreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [AReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type ACreatePayload {
          """
            Returned item from the mutation.
          """
          returning: AReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type ADeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [AReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type ADeletePayload {
          """
            Returned item from the mutation.
          """
          returning: AReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type AEdge {
          node: A!
          cursor: String!
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

        type AUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [AReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type AUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: AReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          aDelete(by: AByInput!): ADeletePayload
          """
            Delete multiple rows of A by a filter
          """
          aDeleteMany(filter: AMutationCollection!): ADeleteManyPayload
          """
            Create a A
          """
          aCreate(input: ACreateInput!): ACreatePayload
          """
            Create multiple As
          """
          aCreateMany(input: [ACreateInput!]!): ACreateManyPayload
          """
            Update a unique A
          """
          aUpdate(by: AByInput!, input: AUpdateInput!): AUpdatePayload
          """
            Update multiple As
          """
          aUpdateMany(filter: AMutationCollection!, input: AUpdateInput!): AUpdateManyPayload
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
          acollection(filter: ACollection, first: Int, last: Int, before: String, after: String, orderBy: [AOrderByInput]): AConnection
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
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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

        input UserCreateInput {
          id: Int!
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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

        input UserCreateInput {
          id: Int!
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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
          """
            	The string matches the given pattern.

            Example: "%ear%" would match strings containing the substring "ear".

            See the reference at https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE

          """ like: String
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

        input UserCreateInput {
          id: Int
          email: String!
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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
          """
            	The string matches the given pattern.

            Example: "%ear%" would match strings containing the substring "ear".

            See the reference at https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE

          """ like: String
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

        input UserCreateInput {
          name: String!
          email: String!
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          privateUserDelete(by: PrivateUserByInput!): PrivateUserDeletePayload
          """
            Delete multiple rows of PrivateUser by a filter
          """
          privateUserDeleteMany(filter: PrivateUserMutationCollection!): PrivateUserDeleteManyPayload
          """
            Create a PrivateUser
          """
          privateUserCreate(input: PrivateUserCreateInput!): PrivateUserCreatePayload
          """
            Create multiple PrivateUsers
          """
          privateUserCreateMany(input: [PrivateUserCreateInput!]!): PrivateUserCreateManyPayload
          """
            Update a unique PrivateUser
          """
          privateUserUpdate(by: PrivateUserByInput!, input: PrivateUserUpdateInput!): PrivateUserUpdatePayload
          """
            Update multiple PrivateUsers
          """
          privateUserUpdateMany(filter: PrivateUserMutationCollection!, input: PrivateUserUpdateInput!): PrivateUserUpdateManyPayload
          """
            Delete a unique PublicUser by a field or combination of fields
          """
          publicUserDelete(by: PublicUserByInput!): PublicUserDeletePayload
          """
            Delete multiple rows of PublicUser by a filter
          """
          publicUserDeleteMany(filter: PublicUserMutationCollection!): PublicUserDeleteManyPayload
          """
            Create a PublicUser
          """
          publicUserCreate(input: PublicUserCreateInput!): PublicUserCreatePayload
          """
            Create multiple PublicUsers
          """
          publicUserCreateMany(input: [PublicUserCreateInput!]!): PublicUserCreateManyPayload
          """
            Update a unique PublicUser
          """
          publicUserUpdate(by: PublicUserByInput!, input: PublicUserUpdateInput!): PublicUserUpdatePayload
          """
            Update multiple PublicUsers
          """
          publicUserUpdateMany(filter: PublicUserMutationCollection!, input: PublicUserUpdateInput!): PublicUserUpdateManyPayload
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

        input PrivateUserCreateInput {
          id: Int
        }

        type PrivateUserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PrivateUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PrivateUserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: PrivateUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PrivateUserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PrivateUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PrivateUserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: PrivateUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PrivateUserEdge {
          node: PrivateUser!
          cursor: String!
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

        type PrivateUserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PrivateUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PrivateUserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: PrivateUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PublicUser {
          id: Int!
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

        input PublicUserCreateInput {
          id: Int
        }

        type PublicUserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PublicUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PublicUserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: PublicUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PublicUserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PublicUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PublicUserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: PublicUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PublicUserEdge {
          node: PublicUser!
          cursor: String!
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

        type PublicUserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PublicUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PublicUserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: PublicUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          userDelete(by: NeonUserByInput!): NeonUserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: NeonUserMutationCollection!): NeonUserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: NeonUserCreateInput!): NeonUserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [NeonUserCreateInput!]!): NeonUserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: NeonUserByInput!, input: NeonUserUpdateInput!): NeonUserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: NeonUserMutationCollection!, input: NeonUserUpdateInput!): NeonUserUpdateManyPayload
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

        input NeonUserCreateInput {
          id: Int
        }

        type NeonUserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [NeonUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type NeonUserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: NeonUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type NeonUserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [NeonUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type NeonUserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: NeonUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type NeonUserEdge {
          node: NeonUser!
          cursor: String!
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

        type NeonUserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [NeonUserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type NeonUserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: NeonUserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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

        input UserCreateInput {
          id: Int
          name: [Int]!
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          Update input for JSON type.
        """
        input JSONUpdateInput {
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
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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

        input UserCreateInput {
          id: Int
          name: JSON!
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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
          name: JSONUpdateInput
        }

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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
          virtual type for non-JSONB operations (only set)
        """
        scalar SimpleJSON

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

        input UserCreateInput {
          id: Int
          name: JSON!
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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

        input BlogCreateInput {
          id: Int
          title: String!
          content: String
          userId: Int!
        }

        type BlogCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [BlogReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type BlogCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: BlogReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type BlogDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [BlogReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type BlogDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: BlogReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type BlogEdge {
          node: Blog!
          cursor: String!
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

        type BlogUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [BlogReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type BlogUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: BlogReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          blogDelete(by: BlogByInput!): BlogDeletePayload
          """
            Delete multiple rows of Blog by a filter
          """
          blogDeleteMany(filter: BlogMutationCollection!): BlogDeleteManyPayload
          """
            Create a Blog
          """
          blogCreate(input: BlogCreateInput!): BlogCreatePayload
          """
            Create multiple Blogs
          """
          blogCreateMany(input: [BlogCreateInput!]!): BlogCreateManyPayload
          """
            Update a unique Blog
          """
          blogUpdate(by: BlogByInput!, input: BlogUpdateInput!): BlogUpdatePayload
          """
            Update multiple Blogs
          """
          blogUpdateMany(filter: BlogMutationCollection!, input: BlogUpdateInput!): BlogUpdateManyPayload
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserDeletePayload
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserMutationCollection!): UserDeleteManyPayload
          """
            Create a User
          """
          userCreate(input: UserCreateInput!): UserCreatePayload
          """
            Create multiple Users
          """
          userCreateMany(input: [UserCreateInput!]!): UserCreateManyPayload
          """
            Update a unique User
          """
          userUpdate(by: UserByInput!, input: UserUpdateInput!): UserUpdatePayload
          """
            Update multiple Users
          """
          userUpdateMany(filter: UserMutationCollection!, input: UserUpdateInput!): UserUpdateManyPayload
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
          """
            	The string matches the given pattern.

            Example: "%ear%" would match strings containing the substring "ear".

            See the reference at https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE

          """ like: String
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

        input UserCreateInput {
          id: Int
          name: String!
        }

        type UserCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserEdge {
          node: User!
          cursor: String!
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

        type UserUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [UserReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type UserUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: UserReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn foreign_key_to_a_table_without_a_key_should_not_create_a_relation() {
    let response = introspect_namespaced_postgres("pg", |api| async move {
        api.execute_sql(r#"CREATE TABLE visible_table (id TEXT PRIMARY KEY)"#)
            .await;

        api.execute_sql(r#"CREATE TABLE hidden_table (visible_table TEXT NOT NULL REFERENCES visible_table(id))"#)
            .await;
    });

    let expected = expect![[r#"
        type Mutation {
          pg: PgMutation
        }

        type PgMutation {
          """
            Delete a unique VisibleTable by a field or combination of fields
          """
          visibleTableDelete(by: PgVisibleTableByInput!): PgVisibleTableDeletePayload
          """
            Delete multiple rows of VisibleTable by a filter
          """
          visibleTableDeleteMany(filter: PgVisibleTableMutationCollection!): PgVisibleTableDeleteManyPayload
          """
            Create a VisibleTable
          """
          visibleTableCreate(input: PgVisibleTableCreateInput!): PgVisibleTableCreatePayload
          """
            Create multiple VisibleTables
          """
          visibleTableCreateMany(input: [PgVisibleTableCreateInput!]!): PgVisibleTableCreateManyPayload
          """
            Update a unique VisibleTable
          """
          visibleTableUpdate(by: PgVisibleTableByInput!, input: PgVisibleTableUpdateInput!): PgVisibleTableUpdatePayload
          """
            Update multiple VisibleTables
          """
          visibleTableUpdateMany(filter: PgVisibleTableMutationCollection!, input: PgVisibleTableUpdateInput!): PgVisibleTableUpdateManyPayload
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

        type PgQuery {
          """
            Query a single PgVisibleTable by a field
          """
          visibleTable(by: PgVisibleTableByInput!): PgVisibleTable
          """
            Paginated query to fetch the whole list of VisibleTable
          """
          visibleTableCollection(filter: PgVisibleTableCollection, first: Int, last: Int, before: String, after: String, orderBy: [PgVisibleTableOrderByInput]): PgVisibleTableConnection
        }

        """
          Search filter input for String type.
        """
        input PgStringSearchFilterInput {
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
          not: PgStringSearchFilterInput
          """
            	The string matches the given pattern.

            Example: "%ear%" would match strings containing the substring "ear".

            See the reference at https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE

          """ like: String
        }

        """
          Update input for String type.
        """
        input PgStringUpdateInput {
          """
            Replaces the value of a field with the specified value.
          """ set: String
        }

        type PgVisibleTable {
          id: String!
        }

        input PgVisibleTableByInput {
          id: String
        }

        input PgVisibleTableCollection {
          id: PgStringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PgVisibleTableCollection]
          """
            None of the filters must match
          """ NONE: [PgVisibleTableCollection]
          """
            At least one of the filters must match
          """ ANY: [PgVisibleTableCollection]
        }

        type PgVisibleTableConnection {
          edges: [PgVisibleTableEdge]!
          pageInfo: PgPageInfo!
        }

        input PgVisibleTableCreateInput {
          id: String!
        }

        type PgVisibleTableCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PgVisibleTableReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgVisibleTableCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: PgVisibleTableReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgVisibleTableDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PgVisibleTableReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgVisibleTableDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: PgVisibleTableReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgVisibleTableEdge {
          node: PgVisibleTable!
          cursor: String!
        }

        input PgVisibleTableMutationCollection {
          id: PgStringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PgVisibleTableMutationCollection]
          """
            None of the filters must match
          """ NONE: [PgVisibleTableMutationCollection]
          """
            At least one of the filters must match
          """ ANY: [PgVisibleTableMutationCollection]
        }

        input PgVisibleTableOrderByInput {
          id: PgOrderByDirection
        }

        type PgVisibleTableReturning {
          id: String!
        }

        input PgVisibleTableUpdateInput {
          id: PgStringUpdateInput
        }

        type PgVisibleTableUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PgVisibleTableReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgVisibleTableUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: PgVisibleTableReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
          networksDelete(by: PgNetworksByInput!): PgNetworksDeletePayload
          """
            Delete multiple rows of Networks by a filter
          """
          networksDeleteMany(filter: PgNetworksMutationCollection!): PgNetworksDeleteManyPayload
          """
            Create a Networks
          """
          networksCreate(input: PgNetworksCreateInput!): PgNetworksCreatePayload
          """
            Create multiple Networkss
          """
          networksCreateMany(input: [PgNetworksCreateInput!]!): PgNetworksCreateManyPayload
          """
            Update a unique Networks
          """
          networksUpdate(by: PgNetworksByInput!, input: PgNetworksUpdateInput!): PgNetworksUpdatePayload
          """
            Update multiple Networkss
          """
          networksUpdateMany(filter: PgNetworksMutationCollection!, input: PgNetworksUpdateInput!): PgNetworksUpdateManyPayload
          """
            Delete a unique Projects by a field or combination of fields
          """
          projectsDelete(by: PgProjectsByInput!): PgProjectsDeletePayload
          """
            Delete multiple rows of Projects by a filter
          """
          projectsDeleteMany(filter: PgProjectsMutationCollection!): PgProjectsDeleteManyPayload
          """
            Create a Projects
          """
          projectsCreate(input: PgProjectsCreateInput!): PgProjectsCreatePayload
          """
            Create multiple Projectss
          """
          projectsCreateMany(input: [PgProjectsCreateInput!]!): PgProjectsCreateManyPayload
          """
            Update a unique Projects
          """
          projectsUpdate(by: PgProjectsByInput!, input: PgProjectsUpdateInput!): PgProjectsUpdatePayload
          """
            Update multiple Projectss
          """
          projectsUpdateMany(filter: PgProjectsMutationCollection!, input: PgProjectsUpdateInput!): PgProjectsUpdateManyPayload
        }

        type PgNetworks {
          id: Int!
          projects(first: Int, last: Int, before: String, after: String, orderBy: [PgProjectsOrderByInput!]): PgProjectsConnection
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

        input PgNetworksCreateInput {
          id: Int
        }

        type PgNetworksCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PgNetworksReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgNetworksCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: PgNetworksReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgNetworksDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PgNetworksReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgNetworksDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: PgNetworksReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgNetworksEdge {
          node: PgNetworks!
          cursor: String!
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

        type PgNetworksUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PgNetworksReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgNetworksUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: PgNetworksReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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

        input PgProjectsCreateInput {
          id: Int
          accessMode: PgAccessMode!
          status: PgProjectStatus
          networkId: Int
        }

        type PgProjectsCreateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PgProjectsReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgProjectsCreatePayload {
          """
            Returned item from the mutation.
          """
          returning: PgProjectsReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgProjectsDeleteManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PgProjectsReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgProjectsDeletePayload {
          """
            Returned item from the mutation.
          """
          returning: PgProjectsReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgProjectsEdge {
          node: PgProjects!
          cursor: String!
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

        type PgProjectsUpdateManyPayload {
          """
            Returned items from the mutation.
          """
          returning: [PgProjectsReturning]!
          """
            The number of rows mutated.
          """
          rowCount: Int!
        }

        type PgProjectsUpdatePayload {
          """
            Returned item from the mutation.
          """
          returning: PgProjectsReturning
          """
            The number of rows mutated.
          """
          rowCount: Int!
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
