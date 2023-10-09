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

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserReduced
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserReducedCollection!): [UserReduced]!
          """
            Create a User
          """
          userCreate(input: UserInput!): UserReduced
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): [UserReduced!]!
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

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int
        }

        input UserOrderByInput {
          id: OrderByDirection
        }

        type UserReduced {
          id: Int!
        }

        input UserReducedCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserReducedCollection]
          """
            None of the filters must match
          """ NONE: [UserReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [UserReducedCollection]
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
        let r#type = indoc! {r#"
            CREATE TYPE street_light AS ENUM ('red', 'yellow', 'green');
        "#};

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

        type AEdge {
          node: A!
          cursor: String!
        }

        input AInput {
          id: Int!
          val: StreetLight!
        }

        input AOrderByInput {
          id: OrderByDirection
          val: OrderByDirection
        }

        type AReduced {
          id: Int!
          val: StreetLight!
        }

        input AReducedCollection {
          id: IntSearchFilterInput
          val: StreetLightSearchFilterInput
          """
            All of the filters must match
          """ ALL: [AReducedCollection]
          """
            None of the filters must match
          """ NONE: [AReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [AReducedCollection]
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

        type Mutation {
          """
            Delete a unique A by a field or combination of fields
          """
          aDelete(by: AByInput!): AReduced
          """
            Delete multiple rows of A by a filter
          """
          aDeleteMany(filter: AReducedCollection!): [AReduced]!
          """
            Create a A
          """
          aCreate(input: AInput!): AReduced
          """
            Create multiple As
          """
          aCreateMany(input: [AInput!]!): [AReduced!]!
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

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserReduced
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserReducedCollection!): [UserReduced]!
          """
            Create a User
          """
          userCreate(input: UserInput!): UserReduced
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): [UserReduced!]!
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

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int!
        }

        input UserOrderByInput {
          id: OrderByDirection
        }

        type UserReduced {
          id: Int!
        }

        input UserReducedCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserReducedCollection]
          """
            None of the filters must match
          """ NONE: [UserReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [UserReducedCollection]
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

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserReduced
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserReducedCollection!): [UserReduced]!
          """
            Create a User
          """
          userCreate(input: UserInput!): UserReduced
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): [UserReduced!]!
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

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int!
        }

        input UserOrderByInput {
          id: OrderByDirection
        }

        type UserReduced {
          id: Int!
        }

        input UserReducedCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserReducedCollection]
          """
            None of the filters must match
          """ NONE: [UserReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [UserReducedCollection]
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

        type Mutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserReduced
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserReducedCollection!): [UserReduced]!
          """
            Create a User
          """
          userCreate(input: UserInput!): UserReduced
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): [UserReduced!]!
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

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int
          email: String!
        }

        input UserOrderByInput {
          id: OrderByDirection
          email: OrderByDirection
        }

        type UserReduced {
          id: Int!
          email: String!
        }

        input UserReducedCollection {
          id: IntSearchFilterInput
          email: StringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserReducedCollection]
          """
            None of the filters must match
          """ NONE: [UserReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [UserReducedCollection]
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
          userDelete(by: UserByInput!): UserReduced
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserReducedCollection!): [UserReduced]!
          """
            Create a User
          """
          userCreate(input: UserInput!): UserReduced
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): [UserReduced!]!
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

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          name: String!
          email: String!
        }

        input UserNameEmailInput {
          name: String!
          email: String!
        }

        input UserOrderByInput {
          name: OrderByDirection
          email: OrderByDirection
        }

        type UserReduced {
          name: String!
          email: String!
        }

        input UserReducedCollection {
          name: StringSearchFilterInput
          email: StringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserReducedCollection]
          """
            None of the filters must match
          """ NONE: [UserReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [UserReducedCollection]
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
        api.execute_sql(r#"CREATE SCHEMA private"#).await;

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

        type Mutation {
          """
            Delete a unique PrivateUser by a field or combination of fields
          """
          privateUserDelete(by: PrivateUserByInput!): PrivateUserReduced
          """
            Delete multiple rows of PrivateUser by a filter
          """
          privateUserDeleteMany(filter: PrivateUserReducedCollection!): [PrivateUserReduced]!
          """
            Create a PrivateUser
          """
          privateUserCreate(input: PrivateUserInput!): PrivateUserReduced
          """
            Create multiple PrivateUsers
          """
          privateUserCreateMany(input: [PrivateUserInput!]!): [PrivateUserReduced!]!
          """
            Delete a unique PublicUser by a field or combination of fields
          """
          publicUserDelete(by: PublicUserByInput!): PublicUserReduced
          """
            Delete multiple rows of PublicUser by a filter
          """
          publicUserDeleteMany(filter: PublicUserReducedCollection!): [PublicUserReduced]!
          """
            Create a PublicUser
          """
          publicUserCreate(input: PublicUserInput!): PublicUserReduced
          """
            Create multiple PublicUsers
          """
          publicUserCreateMany(input: [PublicUserInput!]!): [PublicUserReduced!]!
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

        type PrivateUserEdge {
          node: PrivateUser!
          cursor: String!
        }

        input PrivateUserInput {
          id: Int
        }

        input PrivateUserOrderByInput {
          id: OrderByDirection
        }

        type PrivateUserReduced {
          id: Int!
        }

        input PrivateUserReducedCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PrivateUserReducedCollection]
          """
            None of the filters must match
          """ NONE: [PrivateUserReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [PrivateUserReducedCollection]
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

        type PublicUserEdge {
          node: PublicUser!
          cursor: String!
        }

        input PublicUserInput {
          id: Int
        }

        input PublicUserOrderByInput {
          id: OrderByDirection
        }

        type PublicUserReduced {
          id: Int!
        }

        input PublicUserReducedCollection {
          id: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [PublicUserReducedCollection]
          """
            None of the filters must match
          """ NONE: [PublicUserReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [PublicUserReducedCollection]
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

        type NeonMutation {
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: NeonUserByInput!): NeonUserReduced
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: NeonUserReducedCollection!): [NeonUserReduced]!
          """
            Create a User
          """
          userCreate(input: NeonUserInput!): NeonUserReduced
          """
            Create multiple Users
          """
          userCreateMany(input: [NeonUserInput!]!): [NeonUserReduced!]!
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

        type NeonUserEdge {
          node: NeonUser!
          cursor: String!
        }

        input NeonUserInput {
          id: Int
        }

        input NeonUserOrderByInput {
          id: NeonOrderByDirection
        }

        type NeonUserReduced {
          id: Int!
        }

        input NeonUserReducedCollection {
          id: NeonIntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [NeonUserReducedCollection]
          """
            None of the filters must match
          """ NONE: [NeonUserReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [NeonUserReducedCollection]
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

        input BlogOrderByInput {
          id: OrderByDirection
          title: OrderByDirection
          content: OrderByDirection
          userId: OrderByDirection
        }

        type BlogReduced {
          id: Int!
          title: String!
          content: String
          userId: Int!
        }

        input BlogReducedCollection {
          id: IntSearchFilterInput
          title: StringSearchFilterInput
          content: StringSearchFilterInput
          userId: IntSearchFilterInput
          """
            All of the filters must match
          """ ALL: [BlogReducedCollection]
          """
            None of the filters must match
          """ NONE: [BlogReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [BlogReducedCollection]
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

        type Mutation {
          """
            Delete a unique Blog by a field or combination of fields
          """
          blogDelete(by: BlogByInput!): BlogReduced
          """
            Delete multiple rows of Blog by a filter
          """
          blogDeleteMany(filter: BlogReducedCollection!): [BlogReduced]!
          """
            Create a Blog
          """
          blogCreate(input: BlogInput!): BlogReduced
          """
            Create multiple Blogs
          """
          blogCreateMany(input: [BlogInput!]!): [BlogReduced!]!
          """
            Delete a unique User by a field or combination of fields
          """
          userDelete(by: UserByInput!): UserReduced
          """
            Delete multiple rows of User by a filter
          """
          userDeleteMany(filter: UserReducedCollection!): [UserReduced]!
          """
            Create a User
          """
          userCreate(input: UserInput!): UserReduced
          """
            Create multiple Users
          """
          userCreateMany(input: [UserInput!]!): [UserReduced!]!
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

        type UserEdge {
          node: User!
          cursor: String!
        }

        input UserInput {
          id: Int
          name: String!
        }

        input UserOrderByInput {
          id: OrderByDirection
          name: OrderByDirection
        }

        type UserReduced {
          id: Int!
          name: String!
        }

        input UserReducedCollection {
          id: IntSearchFilterInput
          name: StringSearchFilterInput
          """
            All of the filters must match
          """ ALL: [UserReducedCollection]
          """
            None of the filters must match
          """ NONE: [UserReducedCollection]
          """
            At least one of the filters must match
          """ ANY: [UserReducedCollection]
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#]];

    expected.assert_eq(&response);
}
