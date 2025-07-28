use integration_tests::{gateway::Gateway, runtime};

#[test]
fn inaccessible_required_input_value() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User
                    @join__type(graph: FST, key: "id")
                {
                    id: ID!
                    name: String @join__field(graph: FST) @deprecated(reason: "we have no name")
                }

                type Query {
                    dummy: String @join__field(graph: FST)
                    user_with_opt(id: ID @inaccessible, x: Int!): User @join__field(graph: FST)
                    user_with_req(id: ID! @inaccessible): User @join__field(graph: FST)
                    user_with_nested_opt(input: UserInput, x: Int!): User @join__field(graph: FST)
                    user_with_nested_req(input: UserInput!): User @join__field(graph: FST)
                }

                input UserInput @join__field(graph: FST) {
                    id: ID! @inaccessible
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
          user_with_opt(x: Int!): User
          user_with_nested_opt(x: Int!): User
        }

        type User {
          id: ID!
          name: String @deprecated(reason: "we have no name")
        }
        "#);
    })
}

#[test]
fn inaccessible_required_input_value_should_not_impact_composite_require() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User
                    @join__type(graph: FST, key: "id")
                {
                    id: ID!
                    friends(userId: ID! @composite__require(graph: FST, field: "id")): [User!]!
                }

                type Query {
                    dummy: String @join__field(graph: FST)
                    user: User
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
          user: User
        }

        type User {
          id: ID!
          friends: [User!]!
        }
        "#);
    })
}

#[test]
fn inaccessible_interface_with_no_fields() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User implements Node
                    @join__type(graph: FST, key: "id")
                {
                    id: ID!
                    name: String @join__field(graph: FST) @deprecated(reason: "we have no name")
                }

                interface Node @join__type(graph: FST) {
                    id: ID! @inaccessible
                }

                type Query @join__type(graph: FST) {
                    dummy: String
                    node: Node
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
        }

        type User {
          id: ID!
          name: String @deprecated(reason: "we have no name")
        }
        "#);
    })
}

#[test]
fn inaccessible_object_with_no_fields() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User
                    @join__type(graph: FST, key: "id")
                {
                    id: ID! @inaccessible
                }

                type Query @join__type(graph: FST) {
                    dummy: String
                    user: User
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
        }
        "#);
    })
}

#[test]
fn inaccessible_input_object_with_no_fields() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User
                    @join__type(graph: FST, key: "id")
                {
                    id: ID!
                }

                input UserInput @join__type(graph: FST)  {
                    id: ID! @inaccessible

                }

                type Query @join__type(graph: FST) {
                    dummy: String
                    user(input: UserInput): User
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
          user: User
        }

        type User {
          id: ID!
        }
        "#);
    })
}

#[test]
fn inaccessible_union_with_no_members() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User
                    @join__type(graph: FST, key: "id")
                    @inaccessible
                {
                    id: ID!
                }

                type Account 
                    @join__type(graph: FST, key: "id")
                    @inaccessible
                {
                    id: ID!
                }

                union Any = User | Account

                type Query @join__type(graph: FST) {
                    dummy: String
                    any: Any
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
        }
        "#);
    })
}

#[test]
fn inaccessible_interface_with_no_fields_propagation() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User implements Node & SuperNode
                    @join__type(graph: FST, key: "id")
                {
                    id: ID!
                    name: String @join__field(graph: FST) @deprecated(reason: "we have no name")
                    node: Node
                }

                interface SuperNode @join__type(graph: FST) {
                    node: Node
                }

                interface Node @join__type(graph: FST) {
                    id: ID! @inaccessible
                }

                type Query @join__type(graph: FST) {
                    dummy: String
                    node: SuperNode 
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
        }

        type User {
          id: ID!
          name: String @deprecated(reason: "we have no name")
        }
        "#);
    })
}

#[test]
fn inaccessible_object_with_no_fields_propagation() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User
                    @join__type(graph: FST, key: "id")
                {
                    id: ID! @inaccessible
                    nested: Nested
                }

                type Nested @join__type(graph: FST) {
                    id: ID! @inaccessible
                }


                type Query @join__type(graph: FST) {
                    dummy: String
                    user: User
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
        }
        "#);
    })
}

#[test]
fn inaccessible_input_object_no_fields_propagation() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User
                    @join__type(graph: FST, key: "id")
                {
                    id: ID!
                }

                input UserInput @join__type(graph: FST) {
                    id: ID! @inaccessible
                    nested: NestedInput
                }

                input NestedInput @join__type(graph: FST) {
                    id: ID! @inaccessible
                }

                type Query @join__type(graph: FST) {
                    dummy: String
                    user(input: UserInput): User
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
          user: User
        }

        type User {
          id: ID!
        }
        "#);
    })
}

#[test]
fn inaccessible_complex_output_propagation() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_federated_sdl(
                r#"
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
                    FST @join__graph(name: "fst", url: "http://does.not.exist")
                }

                type User
                    @join__type(graph: FST, key: "id")
                {
                    id: ID! @inaccessible

                }

                type Account 
                    @join__type(graph: FST)
                {
                    user: User
                }

                union Any = User | Account

                interface Super @join__type(graph: FST) {
                    any: Any
                }

                type Random @join__type(graph: FST) {
                    super: Super
                }

                union Any2 = Random

                type Query @join__type(graph: FST) {
                    dummy: String
                    any: Any2
                }
                "#,
            )
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let sdl = gateway.introspect().await;
        insta::assert_snapshot!(sdl, @r#"
        type Query {
          dummy: String
        }
        "#);
    })
}
