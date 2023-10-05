use expect_test::expect;
use indoc::indoc;
use integration_tests::postgresql::{introspect_namespaced_postgresql, introspect_postgresql};

#[test]
fn table_with_serial_primary_key() {
    let response = introspect_postgresql(|api| async move {
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

        input UserOrderByInput {
          id: OrderByDirection
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_enum_field() {
    let response = introspect_postgresql(|api| async move {
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

        input AOrderByInput {
          id: OrderByDirection
          val: OrderByDirection
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
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_int_primary_key() {
    let response = introspect_postgresql(|api| async move {
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

        input UserOrderByInput {
          id: OrderByDirection
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_int_unique() {
    let response = introspect_postgresql(|api| async move {
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

        input UserOrderByInput {
          id: OrderByDirection
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_serial_primary_key_string_unique() {
    let response = introspect_postgresql(|api| async move {
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

        input UserOrderByInput {
          id: OrderByDirection
          email: OrderByDirection
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_composite_primary_key() {
    let response = introspect_postgresql(|api| async move {
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

        input UserNameEmailInput {
          name: String!
          email: String!
        }

        input UserOrderByInput {
          name: OrderByDirection
          email: OrderByDirection
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn two_schemas_same_table_name() {
    let response = introspect_postgresql(|api| async move {
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

        input PrivateUserOrderByInput {
          id: OrderByDirection
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

        input PublicUserOrderByInput {
          id: OrderByDirection
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
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn table_with_serial_primary_key_namespaced() {
    let response = introspect_namespaced_postgresql("Neon", |api| async move {
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

        input NeonUserOrderByInput {
          id: NeonOrderByDirection
        }

        type Query {
          neon: NeonQuery
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}

#[test]
fn two_tables_with_single_column_foreign_key() {
    let response = introspect_postgresql(|api| async move {
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

        input BlogOrderByInput {
          id: OrderByDirection
          title: OrderByDirection
          content: OrderByDirection
          userId: OrderByDirection
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

        input UserOrderByInput {
          id: OrderByDirection
          name: OrderByDirection
        }

        schema {
          query: Query
        }
    "#]];

    expected.assert_eq(&response);
}
