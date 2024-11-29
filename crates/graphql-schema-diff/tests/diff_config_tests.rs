#![allow(unused_crate_dependencies)]

use graphql_schema_diff::{diff_with_config, DiffConfig};

#[test]
fn added_fields_inside_added_types() {
    let source = r#"
        type Query {
            hello: String!
        }
    "#;

    let target = r#"
        type Query {
            hello: String!
        }

        type Mutation implements Greeter & RootType
            @myDirective
        {
            goodDaySir: String!
            goodbye: String!
        }
    "#;

    let diff = diff_with_config(
        source,
        target,
        DiffConfig::default().with_additions_inside_type_definitions(true),
    )
    .unwrap();

    insta::assert_debug_snapshot!(diff, @r#"
    [
        Change {
            path: "Mutation",
            kind: AddObjectType,
            span: Span {
                start: 68,
                end: 217,
            },
        },
        Change {
            path: "Mutation.&Greeter",
            kind: AddInterfaceImplementation,
            span: Span {
                start: 0,
                end: 0,
            },
        },
        Change {
            path: "Mutation.&RootType",
            kind: AddInterfaceImplementation,
            span: Span {
                start: 0,
                end: 0,
            },
        },
        Change {
            path: "Mutation.goodDaySir",
            kind: AddField,
            span: Span {
                start: 159,
                end: 191,
            },
        },
        Change {
            path: "Mutation.goodbye",
            kind: AddField,
            span: Span {
                start: 191,
                end: 216,
            },
        },
    ]
    "#);
}

#[test]
fn added_members_inside_added_unions() {
    let source = r#"
        type Member {
            id: ID!
            nickname: String!
        }

        type Admin {
            id: ID!
            fullName: String!
        }
    "#;

    let target = r#"
        type Member {
            id: ID!
            nickname: String!
        }

        type Admin {
            id: ID!
            fullName: String!
        }

        union MyUnion = Member | Admin
    "#;

    let diff = diff_with_config(
        source,
        target,
        DiffConfig::default().with_additions_inside_type_definitions(true),
    )
    .unwrap();

    insta::assert_debug_snapshot!(diff, @r#"
    [
        Change {
            path: "MyUnion",
            kind: AddUnion,
            span: Span {
                start: 174,
                end: 204,
            },
        },
        Change {
            path: "MyUnion.Admin",
            kind: AddUnionMember,
            span: Span {
                start: 199,
                end: 204,
            },
        },
        Change {
            path: "MyUnion.Member",
            kind: AddUnionMember,
            span: Span {
                start: 190,
                end: 196,
            },
        },
    ]
    "#);
}

#[test]
fn added_values_inside_added_enums() {
    let source = r#"
        type Query {
            hello: String!
        }
    "#;

    let target = r#"
        enum MyEnum {
            A
            B
            C
            D
        }
    "#;

    let diff = diff_with_config(
        source,
        target,
        DiffConfig::default().with_additions_inside_type_definitions(true),
    )
    .unwrap();

    insta::assert_debug_snapshot!(diff, @r#"
    [
        Change {
            path: "MyEnum",
            kind: AddEnum,
            span: Span {
                start: 9,
                end: 88,
            },
        },
        Change {
            path: "MyEnum.A",
            kind: AddEnumValue,
            span: Span {
                start: 35,
                end: 49,
            },
        },
        Change {
            path: "MyEnum.B",
            kind: AddEnumValue,
            span: Span {
                start: 49,
                end: 63,
            },
        },
        Change {
            path: "MyEnum.C",
            kind: AddEnumValue,
            span: Span {
                start: 63,
                end: 77,
            },
        },
        Change {
            path: "MyEnum.D",
            kind: AddEnumValue,
            span: Span {
                start: 77,
                end: 87,
            },
        },
        Change {
            path: "Query",
            kind: RemoveObjectType,
            span: Span {
                start: 9,
                end: 58,
            },
        },
    ]
    "#);
}

#[test]
fn added_fields_inside_input_objects() {
    let source = r#"
        type Query {
            hello: String!
        }
    "#;

    let target = r#"
        input MyInput {
            a: Int!
            b: String!
        }
    "#;

    let diff = diff_with_config(
        source,
        target,
        DiffConfig::default().with_additions_inside_type_definitions(true),
    )
    .unwrap();

    insta::assert_debug_snapshot!(diff, @r#"
    [
        Change {
            path: "MyInput",
            kind: AddInputObject,
            span: Span {
                start: 9,
                end: 77,
            },
        },
        Change {
            path: "MyInput.a",
            kind: AddField,
            span: Span {
                start: 37,
                end: 57,
            },
        },
        Change {
            path: "MyInput.b",
            kind: AddField,
            span: Span {
                start: 57,
                end: 76,
            },
        },
        Change {
            path: "Query",
            kind: RemoveObjectType,
            span: Span {
                start: 9,
                end: 58,
            },
        },
    ]
    "#);
}

#[test]
fn added_fields_inside_added_interfaces() {
    let source = r#"
        type Query {
            hello: String!
        }
    "#;

    let target = r#"
        interface MyInterface {
            a: Int!
            b: String!
        }
    "#;

    let diff = diff_with_config(
        source,
        target,
        DiffConfig::default().with_additions_inside_type_definitions(true),
    )
    .unwrap();

    insta::assert_debug_snapshot!(diff, @r#"
    [
        Change {
            path: "MyInterface",
            kind: AddInterface,
            span: Span {
                start: 9,
                end: 85,
            },
        },
        Change {
            path: "MyInterface.a",
            kind: AddField,
            span: Span {
                start: 45,
                end: 65,
            },
        },
        Change {
            path: "MyInterface.b",
            kind: AddField,
            span: Span {
                start: 65,
                end: 84,
            },
        },
        Change {
            path: "Query",
            kind: RemoveObjectType,
            span: Span {
                start: 9,
                end: 58,
            },
        },
    ]
    "#);
}
