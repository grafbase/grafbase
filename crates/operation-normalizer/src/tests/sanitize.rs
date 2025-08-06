#[test]
fn unnamed_query() {
    let input = indoc::indoc! {r#"
        query {
            user {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query { user { id } }");
}

#[test]
fn unnamed_query_with_variables() {
    let input = indoc::indoc! {r#"
        query($id: ID!, $name: String, $ages: [Int!]!, $other: [String]) {
            user(id: $id, name: $name, ages: $ages, other: $other) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query($id: ID!, $name: String, $ages: [Int!]!, $other: [String]) { user(id: $id, name: $name, ages: $ages, other: $other) { id } }");
}

#[test]
fn named_query() {
    let input = indoc::indoc! {r#"
        query GetUser {
            user {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query GetUser { user { id } }");
}

#[test]
fn named_query_with_variables() {
    let input = indoc::indoc! {r#"
        query GetUser($id: ID!) {
            user(id: $id) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query GetUser($id: ID!) { user(id: $id) { id } }");
}

#[test]
fn query_with_static_string_gets_sanitized() {
    let input = indoc::indoc! {r#"
        query {
            user(id: "secret-id") {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @r#"query { user(id: "") { id } }"#);
}

#[test]
fn query_with_static_int_gets_sanitized() {
    let input = indoc::indoc! {r#"
        query {
            user(id: 123) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query { user(id: 0) { id } }");
}

#[test]
fn query_with_static_float_gets_sanitized() {
    let input = indoc::indoc! {r#"
        query {
            user(id: 1.23) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query { user(id: 0) { id } }");
}

#[test]
fn query_with_static_null() {
    let input = indoc::indoc! {r#"
        query {
            user(id: null) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query { user(id: null) { id } }");
}

#[test]
fn query_with_enum() {
    let input = indoc::indoc! {r#"
        query {
            user(role: ADMIN) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query { user(role: ADMIN) { id } }");
}

#[test]
fn query_with_list() {
    let input = indoc::indoc! {r#"
        query {
            user(id: [1, 2, 3, 4]) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query { user(id: []) { id } }");
}

#[test]
fn query_with_object() {
    let input = indoc::indoc! {r#"
        query {
            user(id: { key: "value" }) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @r#"query { user(id: {key: ""}) { id } }"#);
}

#[test]
fn query_with_complex_object() {
    let input = indoc::indoc! {r#"
        query {
            createUser(input: { 
                name: "Alice", 
                age: 30,
                active: true,
                role: ADMIN,
                scores: [1, 2, 3],
                metadata: { key: "value", count: 10 }
            }) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @r#"query { createUser(input: {name: "", age: 0, active: true, role: ADMIN, scores: [], metadata: {key: "", count: 0}}) { id } }"#);
}

#[test]
fn query_with_object_containing_variable() {
    let input = indoc::indoc! {r#"
        query($userId: ID!) {
            updateUser(input: { id: $userId, name: "New Name" }) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @r#"query($userId: ID!) { updateUser(input: {id: $userId, name: ""}) { id } }"#);
}

#[test]
fn test_sanitization_requirements() {
    let input = indoc::indoc! {r#"
        query($var: String!, $enumVar: Role) {
            user(filter: {
                name: "John Doe",
                age: 42,
                height: 1.75,
                active: true,
                role: ADMIN,
                tags: ["tag1", "tag2"],
                nullField: null,
                varField: $var,
                enumField: $enumVar,
                nested: {
                    id: 123,
                    data: "secret"
                }
            }) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);

    // Verify all requirements:
    // - Int and Float replaced by 0
    // - Strings replaced with ""
    // - Lists replaced with []
    // - Objects preserve structure but values are sanitized
    // - Boolean preserved (true)
    // - Enum preserved (ADMIN)
    // - Variable references preserved ($var, $enumVar)
    // - null preserved
    insta::assert_snapshot!(output, @r#"query($var: String!, $enumVar: Role) { user(filter: {name: "", age: 0, height: 0, active: true, role: ADMIN, tags: [], nullField: null, varField: $var, enumField: $enumVar, nested: {id: 0, data: ""}}) { id } }"#);
}

#[test]
fn query_with_static_bool_true() {
    let input = indoc::indoc! {r#"
        query {
            user(visible: true) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query { user(visible: true) { id } }");
}

#[test]
fn query_with_static_bool_false() {
    let input = indoc::indoc! {r#"
        query {
            user(visible: false) {
                id
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query { user(visible: false) { id } }");
}

#[test]
fn aliases() {
    let input = indoc::indoc! {r#"
        query($id: ID!) {
            currentUser: user(id: $userId) {
                id
                name
            }

            postsThisWeek: posts(week: "2021-10-04") {
                id
                title
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @r#"query($id: ID!) { currentUser: user(id: $userId) { id name } postsThisWeek: posts(week: "") { id title } }"#);
}

#[test]
fn field_directive() {
    let input = indoc::indoc! {r#"
        query($includeDetails: Boolean!) {
            id
            details @include(if: $includeDetails) {
                id
                title
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);
    insta::assert_snapshot!(output, @"query($includeDetails: Boolean!) { id details @include(if: $includeDetails) { id title } }");
}

#[test]
fn fragment_spreads() {
    let input = indoc::indoc! {r#"
        query {
            user {
                ...UserFields
            }
        }

        fragment UserFields on User {
            id
            name
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);

    insta::assert_snapshot!(output, @"query { user { ...UserFields  } } fragment UserFields on User { id name }");
}

#[test]
fn inline_fragments() {
    let input = indoc::indoc! {r#"
        query {
            user {
                ... on User {
                    id
                    name
                }
                ... {
                    id
                    role
                }
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);

    insta::assert_snapshot!(output, @"query { user { ... on User { id name } ... { id role } } }");
}

#[test]
fn nested_fragments() {
    let input = indoc::indoc! {r#"
        query {
            user {
                ...UserFields
            }
        }

        fragment UserFields on User {
            id
            name
            ...UserDetails
        }

        fragment UserDetails on User {
            email
            phone
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);

    insta::assert_snapshot!(output, @"query { user { ...UserFields  } } fragment UserFields on User { id name ...UserDetails  } fragment UserDetails on User { email phone }");
}

#[test]
fn mutation() {
    let input = indoc::indoc! {r#"
        mutation {
            createUser(name: "Alice", age: 30) {
                id
                name
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);

    insta::assert_snapshot!(output, @r#"mutation { createUser(name: "", age: 0) { id name } }"#);
}

#[test]
fn subscription() {
    let input = indoc::indoc! {r#"
        subscription {
            newUser {
                id
                name
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);

    insta::assert_snapshot!(output, @r#"subscription { newUser { id name } }"#);
}

#[test]
fn query_with_list_of_objects() {
    let input = indoc::indoc! {r#"
        query {
            users(filters: [
                { name: "Alice", age: 30, active: true },
                { name: "Bob", age: 25, active: false },
                { name: "Charlie", age: 35, active: true }
            ]) {
                id
                name
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);

    insta::assert_snapshot!(output, @"query { users(filters: []) { id name } }");
}

#[test]
fn query_with_nested_list_of_objects() {
    let input = indoc::indoc! {r#"
        query {
            createTeam(input: {
                name: "Engineering",
                members: [
                    { 
                        name: "Alice", 
                        role: "Lead",
                        skills: ["Rust", "GraphQL"],
                        projects: [
                            { name: "Project A", status: "Active" },
                            { name: "Project B", status: "Completed" }
                        ]
                    },
                    { 
                        name: "Bob", 
                        role: "Developer",
                        skills: ["JavaScript", "React"],
                        projects: []
                    }
                ]
            }) {
                id
                name
            }
        }
    "#};

    let document = cynic_parser::parse_executable_document(input).unwrap();
    let output = crate::sanitize(&document);

    insta::assert_snapshot!(output, @r#"query { createTeam(input: {name: "", members: []}) { id name } }"#);
}
