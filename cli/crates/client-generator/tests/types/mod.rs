use crate::common::expect_generated;
use expect_test::expect;
use indoc::indoc;

#[test]
fn required_id() {
    let graphql = indoc! {r#"
        type a {
            b: ID!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn optional_id() {
    let graphql = indoc! {r#"
        type a {
            b: ID
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string | null
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_string() {
    let graphql = indoc! {r#"
        type a {
            b: String!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_int() {
    let graphql = indoc! {r#"
        type a {
            b: Int!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: number
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_float() {
    let graphql = indoc! {r#"
        type a {
            b: Float!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: number
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_boolean() {
    let graphql = indoc! {r#"
        type a {
            b: Boolean!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: boolean
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_date() {
    let graphql = indoc! {r#"
        type a {
            b: Date!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: Date
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_datetime() {
    let graphql = indoc! {r#"
        type a {
            b: DateTime!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: Date
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_email() {
    let graphql = indoc! {r#"
        type a {
            b: Email!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_ip() {
    let graphql = indoc! {r#"
        type a {
            b: IPAddress!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_timestamp() {
    let graphql = indoc! {r#"
        type a {
            b: Timestamp!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: Date
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_url() {
    let graphql = indoc! {r#"
        type a {
            b: URL!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn optional_json() {
    let graphql = indoc! {r#"
        type a {
            b: JSON
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: object | null
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_phone_number() {
    let graphql = indoc! {r#"
        type a {
            b: PhoneNumber!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn dependent_types() {
    let graphql = indoc! {r#"
        type a {
            b: String!
        }

        type a {
            b: Address!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string
        }

        export interface a {
          __typename?: 'a'
          b: Address
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn description() {
    let graphql = indoc! {r#"
        """
        Hey we have this cool type.
        Why don't you give it a try! 
        """
        type a {
            b: String!
        }
    "#};

    let expected = expect![[r#"
        /**
         * Hey we have this cool type.
         * Why don't you give it a try!
         */
        export interface a {
          __typename?: 'a'
          b: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn field_description() {
    let graphql = indoc! {r#"
        type a {
            """
            Hey we have this cool field.
            Why don't you give it a try! 
            """
            b: String!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          /**
           * Hey we have this cool field.
           * Why don't you give it a try!
           */
          b: string
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_array_with_required_elements() {
    let graphql = indoc! {r#"
        type a {
            b: [String!]!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string[]
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_array_with_optional_elements() {
    let graphql = indoc! {r#"
        type a {
            b: [String]!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: (string | null)[]
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn optional_array_with_required_elements() {
    let graphql = indoc! {r#"
        type a {
            b: [String!]
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string[] | null
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn optional_array_with_optional_elements() {
    let graphql = indoc! {r#"
        type a {
            b: [String]
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: (string | null)[] | null
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_matrix_with_required_elements() {
    let graphql = indoc! {r#"
        type a {
            b: [[String!]!]!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string[][]
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_matrix_with_optional_elements() {
    let graphql = indoc! {r#"
        type a {
            b: [[String]!]!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: ((string | null)[])[]
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_matrix_with_optional_arrays() {
    let graphql = indoc! {r#"
        type a {
            b: [[String!]]!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: (string[] | null)[]
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn required_matrix_with_optional_arrays_and_elements() {
    let graphql = indoc! {r#"
        type a {
            b: [[String]]!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: ((string | null)[] | null)[]
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn optional_matrix_with_optional_arrays_and_elements() {
    let graphql = indoc! {r#"
        type a {
            b: [[String]]
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: ((string | null)[] | null)[] | null
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn with_resolvers() {
    let graphql = indoc! {r#"
        type a {
            b(c: String!): String!
        }
    "#};

    let expected = expect![[r#"
        export interface a {
          __typename?: 'a'
          b: string
        }
    "#]];

    expect_generated(graphql, &expected);
}
