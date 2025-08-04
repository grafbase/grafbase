use crate::normalize;
use indoc::indoc;

#[test]
fn no_operation_name() {
    let input = indoc! {r#"
        query Employees { employeeCollection(first: 5) { edges { node { id firstName lastName } } } }
    "#};
    let output = normalize(input, None).unwrap();
    insta::assert_snapshot!(output, @r#"
        query Employees {
          employeeCollection(first: 0) {
            edges {
              node {
                firstName
                id
                lastName
              }
            }
          }
        }
    "#);
}

#[test]
fn apollo_example() {
    // https://www.apollographql.com/docs/graphos/metrics/operation-signatures/
    let input = indoc! {r#"
        # Operation definition needs to appear after all fragment definitions
        query GetUser {
          user(id: "hello") {
            # Replace string argument value with empty string
            ...NameParts # Spread fragment needs to appear after individual fields
            timezone # Needs to appear alphanumerically after `name`
            aliased: name # Need to remove alias
          }
        }

        # Excessive characters (including this comment!) need to be removed
        fragment NameParts on User {
          firstname
          lastname
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        fragment NameParts on User {
          firstname
          lastname
        }

        query GetUser {
          user(id: "") {
            name
            timezone
            ...NameParts
          }
        }
    "#);
}

#[test]
fn inline_strings() {
    let input = indoc! {r#"
        query GetUser {
          user(id: "foo") @include(by: "secret") {
            names(first: 10) { name }
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user(id: "") @include(by: "") {
            names(first: 0) {
              name
            }
          }
        }
    "#);
}

#[test]
fn inline_ints() {
    let input = indoc! {r#"
        query GetUser {
          user(id: 69420) {
            name
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user(id: 0) {
            name
          }
        }
    "#);
}

#[test]
fn inline_floats() {
    let input = indoc! {r#"
        query GetUser {
          user(id: 69.420) {
            name
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user(id: 0) {
            name
          }
        }
    "#);
}

#[test]
fn inline_lists() {
    let input = indoc! {r#"
        query GetUser {
          user(id: [1, 2, 3]) {
            name
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user(id: []) {
            name
          }
        }
    "#);
}

#[test]
fn inline_objects() {
    let input = indoc! {r#"
        query GetUser {
          user(id: { value: 420 }) {
            name
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user(id: {}) {
            name
          }
        }
    "#);
}

#[test]
fn inline_enums() {
    let input = indoc! {r#"
        query GetUser {
          user(id: ID) {
            name
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user(id: ID) {
            name
          }
        }
    "#);
}

#[test]
fn inline_booleans() {
    let input = indoc! {r#"
        query GetUser {
          user(id: true) {
            name
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user(id: true) {
            name
          }
        }
    "#);
}

#[test]
fn variables() {
    let input = indoc! {r#"
        query GetUser($id: Int) {
          user(id: $id) {
            name
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser($id: Int) {
          user(id: $id) {
            name
          }
        }
    "#);
}

#[test]
fn argument_order() {
    let input = indoc! {r#"
        query GetUser($foo: Int, $bar: Int, $limit: Int) {
          user(foo: $foo, bar: $bar) {
            names(limit: $limit, foo: $foo) {
              name
            }
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser($bar: Int, $foo: Int, $limit: Int) {
          user(bar: $bar, foo: $foo) {
            names(foo: $foo, limit: $limit) {
              name
            }
          }
        }
    "#);
}

#[test]
fn inline_fragment() {
    let input = indoc! {r#"
       query GetUser($zimit: String, $limit: Int) {
         user {
           ... on User {
             lastname
             firstname
             nicknames(zimit: $zimit, limit: $limit) {
               name
             }
           }
           age
         }
       }

    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser($limit: Int, $zimit: String) {
          user {
            age
            ... on User {
              firstname
              lastname
              nicknames(limit: $limit, zimit: $zimit) {
                name
              }
            }
          }
        }
    "#);
}

#[test]
fn used_fragment() {
    let input = indoc! {r#"
       query {
         user {
           ...NameParts
           name
         }
       }

       fragment NameParts on User {
         lastname
         firstname
       }
    "#};

    let output = normalize(input, None).unwrap();

    insta::assert_snapshot!(output, @r#"
        fragment NameParts on User {
          firstname
          lastname
        }

        query {
          user {
            name
            ...NameParts
          }
        }
    "#);
}

#[test]
fn used_fragment_in_fragment() {
    let input = indoc! {r#"
       query {
         user {
           ...NameParts
           name
         }
       }

       fragment NameParts on User {
         lastname
         firstname
         ...AgeParts
       }

       fragment AgeParts on User {
         age
       }
    "#};

    let output = normalize(input, None).unwrap();

    insta::assert_snapshot!(output, @r#"
        fragment AgeParts on User {
          age
        }

        fragment NameParts on User {
          firstname
          lastname
          ...AgeParts
        }

        query {
          user {
            name
            ...NameParts
          }
        }
    "#);
}

#[test]
fn unused_fragment() {
    let input = indoc! {r#"
       query {
         user {
           name
         }
       }

       fragment NameParts on User {
         firstname
         lastname
       }
    "#};

    let output = normalize(input, None).unwrap();

    insta::assert_snapshot!(output, @r#"
        query {
          user {
            name
          }
        }
    "#);
}

#[test]
fn unused_queries() {
    let input = indoc! {r#"
        query GetUser {
          user {
            name
          }
        }

        query GetLocation {
          location {
            address
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user {
            name
          }
        }
    "#);
}

#[test]
fn unused_mutations() {
    let input = indoc! {r#"
        mutation UpdateUser {
          updateUser {
            name
          }
        }

        mutation UpdateLocation {
          updateLocation {
            address
          }
        }
    "#};

    let output = normalize(input, Some("UpdateUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        mutation UpdateUser {
          updateUser {
            name
          }
        }
    "#);
}

#[test]
fn unused_subscriptions() {
    let input = indoc! {r#"
        subscription Users {
          users {
            name
          }
        }

        subscription Locations {
          locations {
            address
          }
        }
    "#};

    let output = normalize(input, Some("Locations")).unwrap();

    insta::assert_snapshot!(output, @r#"
        subscription Locations {
          locations {
            address
          }
        }
    "#);
}

#[test]
fn directives() {
    let input = indoc! {r#"
        query GetUser {
          user @include @exclude {
            name
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user @exclude @include {
            name
          }
        }
    "#);
}

#[test]
fn directive_arguments() {
    let input = indoc! {r#"
        query GetUser {
          user @include(zoo: false, goo: true) {
            name
          }
        }
    "#};

    let output = normalize(input, Some("GetUser")).unwrap();

    insta::assert_snapshot!(output, @r#"
        query GetUser {
          user @include(goo: true, zoo: false) {
            name
          }
        }
    "#);
}
