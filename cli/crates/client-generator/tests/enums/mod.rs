use crate::common::expect_generated;
use expect_test::expect;
use indoc::indoc;

#[test]
fn single_enum() {
    let graphql = indoc! {r#"
        enum A {
            B
            C
            D
        }
    "#};

    let expected = expect![[r#"
        export enum A {
          B,
          C,
          D
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn commented_enum() {
    let graphql = indoc! {r#"
        """
        Comment #1
        Comment #2  
        """
        enum A {
            B
            C
            D
        }
    "#};

    let expected = expect![[r#"
        /**
         * Comment #1
         * Comment #2
         */
        export enum A {
          B,
          C,
          D
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn commented_enum_variant() {
    let graphql = indoc! {r#"
        enum A {
            """
            Comment #1
            Comment #2  
            """
            B
            C
            D
        }
    "#};

    let expected = expect![[r#"
        export enum A {
          /**
           * Comment #1
           * Comment #2
           */
          B,
          C,
          D
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn input_type_using_enum() {
    let graphql = indoc! {r#"
        enum A {
            B
            C
            D
        }

        input B {
            f: A
        }
    "#};

    let expected = expect![[r#"
        export enum A {
          B,
          C,
          D
        }

        export interface B {
          __typename?: 'B'
          f?: A | null
        }
    "#]];

    expect_generated(graphql, &expected);
}

#[test]
fn type_using_enum() {
    let graphql = indoc! {r#"
        enum A {
            B
            C
            D
        }

        type B {
            f: A
        }
    "#};

    let expected = expect![[r#"
        export enum A {
          B,
          C,
          D
        }

        export interface B {
          __typename?: 'B'
          f: A | null
        }
    "#]];

    expect_generated(graphql, &expected);
}
