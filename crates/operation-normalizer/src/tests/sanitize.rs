use expect_test::expect;

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
    let expected = expect!["query { user { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query($id: ID!, $name: String, $ages: [Int!]!, $other: [String]) { user(id: $id, name: $name, ages: $ages, other: $other) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query GetUser { user { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query GetUser($id: ID!) { user(id: $id) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect![[r#"query { user(id: "") { id } }"#]];

    expected.assert_eq(&output);
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
    let expected = expect!["query { user(id: 0) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query { user(id: 0) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query { user(id: null) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query { user(role: ADMIN) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query { user(id: []) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query { user(id: {}) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query { user(visible: true) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect!["query { user(visible: false) { id } }"];

    expected.assert_eq(&output);
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
    let expected = expect![[
        r#"query($id: ID!) { currentUser: user(id: $userId) { id name } postsThisWeek: posts(week: "") { id title } }"#
    ]];

    expected.assert_eq(&output);
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
    let expected =
        expect!["query($includeDetails: Boolean!) { id details @include(if: $includeDetails) { id title } }"];

    expected.assert_eq(&output);
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

    let expected = expect!["query { user { ...UserFields  } } fragment UserFields on User { id name }"];

    expected.assert_eq(&output);
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

    let expected = expect!["query { user { ... on User { id name } ... { id role } } }"];

    expected.assert_eq(&output);
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

    let expected = expect!["query { user { ...UserFields  } } fragment UserFields on User { id name ...UserDetails  } fragment UserDetails on User { email phone }"];

    expected.assert_eq(&output);
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

    let expected = expect![[r#"mutation { createUser(name: "", age: 0) { id name } }"#]];

    expected.assert_eq(&output);
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

    let expected = expect![[r#"subscription { newUser { id name } }"#]];

    expected.assert_eq(&output);
}
