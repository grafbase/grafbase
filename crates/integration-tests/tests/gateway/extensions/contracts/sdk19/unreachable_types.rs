use serde_json::json;

#[test]
fn scalar_types() {
    let sdl = r#"
        type Query {
            timestamp: DateTime
            id: UUID @tag(name: "internal")
        }

        scalar DateTime
        scalar UUID
        scalar CustomId
        "#;
    let key = &json!({
        "excludedTags": ["internal"]
    });

    let contract = super::run(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    scalar CustomId

    scalar DateTime

    type Query {
      timestamp: DateTime
    }

    scalar UUID
    "#);

    let contract = super::run_hide_unreachable_types(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    scalar DateTime

    type Query {
      timestamp: DateTime
    }
    "#);
}

#[test]
fn object_types() {
    let sdl = r#"
        type Query {
            user: User
        }

        type User {
            id: ID!
            profile: Profile
        }

        type Profile {
            name: String!
            settings: Settings @tag(name: "internal")
        }

        type Settings {
            theme: String!
        }

        type UnreachableType {
            data: String!
        }
        "#;
    let key = &json!({
        "excludedTags": ["internal"]
    });

    let contract = super::run(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    type Profile {
      name: String!
    }

    type Query {
      user: User
    }

    type Settings {
      theme: String!
    }

    type UnreachableType {
      data: String!
    }

    type User {
      id: ID!
      profile: Profile
    }
    "#);

    let contract = super::run_hide_unreachable_types(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    type Profile {
      name: String!
    }

    type Query {
      user: User
    }

    type User {
      id: ID!
      profile: Profile
    }
    "#);
}

#[test]
fn interface_types() {
    let sdl = r#"
        type Query {
            node: Node @tag(name: "internal")
            product: Product
        }

        # unreachable
        interface Node {
            id: ID!
        }

        # has no implementation
        interface Entity {
            name: String!
        }

        # Not directly reachable, but should be kept
        interface HiddenInterface {
            data: String!
        }

        type User implements Node & Entity @tag(name: "internal") {
            id: ID!
            name: String!
        }

        type Product implements Node & HiddenInterface {
            id: ID!
            data: String!
        }
        "#;
    let key = &json!({
        "excludedTags": ["internal"]
    });

    let contract = super::run(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    interface Entity {
      name: String!
    }

    interface HiddenInterface {
      data: String!
    }

    interface Node {
      id: ID!
    }

    type Product implements Node & HiddenInterface {
      data: String!
      id: ID!
    }

    type Query {
      product: Product
    }
    "#);

    let contract = super::run_hide_unreachable_types(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    interface HiddenInterface {
      data: String!
    }

    interface Node {
      id: ID!
    }

    type Product implements Node & HiddenInterface {
      data: String!
      id: ID!
    }

    type Query {
      product: Product
    }
    "#);
}

#[test]
fn union_types() {
    let sdl = r#"
        type Query {
            search: SearchResult
            blocked: BlockedUnion @tag(name: "internal")
            empty: EmptyUnion
        }

        union SearchResult = User | Product

        union UnreachableUnion = Article | Comment

        union BlockedUnion = Article | Comment

        union EmptyUnion = Article


        type User {
            id: ID!
        }

        type Product {
            name: String!
        }

        type Article @tag(name: "internal") {
            title: String!
        }

        type Comment {
            text: String!
        }
        "#;
    let key = &json!({
        "excludedTags": ["internal"]
    });

    let contract = super::run(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    union BlockedUnion = Comment

    type Comment {
      text: String!
    }

    type Product {
      name: String!
    }

    type Query {
      search: SearchResult
    }

    union SearchResult = User | Product

    union UnreachableUnion = Comment

    type User {
      id: ID!
    }
    "#);

    let contract = super::run_hide_unreachable_types(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    type Product {
      name: String!
    }

    type Query {
      search: SearchResult
    }

    union SearchResult = User | Product

    type User {
      id: ID!
    }
    "#);
}

#[test]
fn enum_types() {
    let sdl = r#"
        type Query {
            userRole: Role
            status: Status @tag(name: "internal")
            category: Category
        }

        enum Role {
            ADMIN
            USER
        }

        enum UnreachableStatus {
            ACTIVE
            INACTIVE
        }

        enum Status {
            PENDING
            COMPLETED
        }

        enum Category {
            ELECTRONICS @tag(name: "internal")
            CLOTHING @tag(name: "internal")
        }
        "#;
    let key = &json!({
        "excludedTags": ["internal"]
    });

    let contract = super::run(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    type Query {
      userRole: Role
    }

    enum Role {
      ADMIN
      USER
    }

    enum Status {
      PENDING
      COMPLETED
    }

    enum UnreachableStatus {
      ACTIVE
      INACTIVE
    }
    "#);

    let contract = super::run_hide_unreachable_types(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    type Query {
      userRole: Role
    }

    enum Role {
      ADMIN
      USER
    }
    "#);
}

#[test]
fn input_types() {
    let sdl = r#"
        type Query {
            searchUsers(filter: UserFilter, second: SecondFilter): [User]
            searchProducts(filter: ProductFilter @tag(name: "internal")): [Product]
        }

        input SecondFilter {
            id: ID! @tag(name: "internal")
        }

        input UserFilter {
            name: String
        }

        input ProductFilter {
            category: String
        }

        input UnreachableInput {
            data: String
        }

        type User {
            id: ID!
        }

        type Product {
            name: String!
        }
        "#;
    let key = &json!({
        "excludedTags": ["internal"]
    });

    let contract = super::run(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    type Product {
      name: String!
    }

    input ProductFilter {
      category: String
    }

    type Query {
      searchProducts: [Product]
      searchUsers(filter: UserFilter): [User]
    }

    input UnreachableInput {
      data: String
    }

    type User {
      id: ID!
    }

    input UserFilter {
      name: String
    }
    "#);

    let contract = super::run_hide_unreachable_types(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    type Product {
      name: String!
    }

    type Query {
      searchProducts: [Product]
      searchUsers(filter: UserFilter): [User]
    }

    type User {
      id: ID!
    }

    input UserFilter {
      name: String
    }
    "#);
}

#[test]
fn complex_unreachable_scenario() {
    let sdl = r#"
        type Query {
            publicData: PublicType
            blockedData: BlockedType @tag(name: "internal")
        }

        type PublicType {
            id: ID!
            reachableField: ReachableType
        }

        type BlockedType {
            id: ID!
            data: String!
        }

        type ReachableType {
            value: String!
            nested: NestedReachable
        }

        type NestedReachable {
            data: String!
        }

        type UnreachableType {
            field: String!
            connection: AnotherUnreachable
        }

        type AnotherUnreachable {
            value: Int!
        }

        interface UnreachableInterface {
            id: ID!
        }

        union UnreachableUnion = UnreachableType | AnotherUnreachable

        enum UnreachableEnum {
            VALUE1
            VALUE2
        }

        input UnreachableInput {
            field: String
        }

        scalar UnreachableScalar
        "#;
    let key = &json!({
        "excludedTags": ["internal"]
    });

    let contract = super::run(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    type AnotherUnreachable {
      value: Int!
    }

    type BlockedType {
      data: String!
      id: ID!
    }

    type NestedReachable {
      data: String!
    }

    type PublicType {
      id: ID!
      reachableField: ReachableType
    }

    type Query {
      publicData: PublicType
    }

    type ReachableType {
      nested: NestedReachable
      value: String!
    }

    enum UnreachableEnum {
      VALUE1
      VALUE2
    }

    input UnreachableInput {
      field: String
    }

    interface UnreachableInterface {
      id: ID!
    }

    scalar UnreachableScalar

    type UnreachableType {
      connection: AnotherUnreachable
      field: String!
    }

    union UnreachableUnion = UnreachableType | AnotherUnreachable
    "#);

    let contract = super::run_hide_unreachable_types(sdl, key);
    insta::assert_snapshot!(contract, @r#"
    type NestedReachable {
      data: String!
    }

    type PublicType {
      id: ID!
      reachableField: ReachableType
    }

    type Query {
      publicData: PublicType
    }

    type ReachableType {
      nested: NestedReachable
      value: String!
    }
    "#);
}
