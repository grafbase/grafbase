use crate::common::expect_generated;
use expect_test::expect;
use indoc::indoc;

#[test]
fn required_id() {
    let graphql = indoc! {r#"
        input User {
            id: ID!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          id: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn optional_id() {
    let graphql = indoc! {r#"
        input User {
            id: ID
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          id?: string | null
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_string() {
    let graphql = indoc! {r#"
        input User {
            name: String!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          name: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_int() {
    let graphql = indoc! {r#"
        input User {
            age: Int!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          age: number
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_float() {
    let graphql = indoc! {r#"
        input User {
            weight: Float!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          weight: number
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_boolean() {
    let graphql = indoc! {r#"
        input User {
            registered: Boolean!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          registered: boolean
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_date() {
    let graphql = indoc! {r#"
        input User {
            registered: Date!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          registered: Date
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_datetime() {
    let graphql = indoc! {r#"
        input User {
            registered: DateTime!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          registered: Date
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_email() {
    let graphql = indoc! {r#"
        input User {
            email: Email!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          email: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_ip() {
    let graphql = indoc! {r#"
        input User {
            ip: IPAddress!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          ip: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_timestamp() {
    let graphql = indoc! {r#"
        input User {
            lastSeen: Timestamp!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          lastSeen: Date
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_url() {
    let graphql = indoc! {r#"
        input User {
            homepage: URL!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          homepage: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn optional_json() {
    let graphql = indoc! {r#"
        input User {
            homepage: JSON
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          homepage?: object | null
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_phone_number() {
    let graphql = indoc! {r#"
        input User {
            contactNo: PhoneNumber!
        }
    "#};

    let expected = expect![[r#"
        export interface User {
          __typename?: 'User'
          contactNo: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn dependent_types() {
    let graphql = indoc! {r#"
        input Address {
            street: String!
        }

        input User {
            address: Address!
        }
    "#};

    let expected = expect![[r#"
        export interface Address {
          __typename?: 'Address'
          street: string
        }

        export interface User {
          __typename?: 'User'
          address: Address
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn input_description() {
    let graphql = indoc! {r#"
        """
        Hey we have this cool type.
        Why don't you give it a try! 
        """
        input Cool {
            howCool: String!
        }
    "#};

    let expected = expect![[r#"
        /**
         * Hey we have this cool type.
         * Why don't you give it a try!
         */
        export interface Cool {
          __typename?: 'Cool'
          howCool: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn input_field_description() {
    let graphql = indoc! {r#"
        input Cool {
            """
            Hey we have this cool field.
            Why don't you give it a try! 
            """
            howCool: String!
        }
    "#};

    let expected = expect![[r#"
        export interface Cool {
          __typename?: 'Cool'
          /**
           * Hey we have this cool field.
           * Why don't you give it a try!
           */
          howCool: string
        }
    "#]];

    expect_generated(graphql, &expected);
}
