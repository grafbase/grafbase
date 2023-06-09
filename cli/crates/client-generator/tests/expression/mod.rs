mod value;

use crate::common::{expect, expect_ts, indoc};
use grafbase_client_generator::{
    expression::{Closure, Equals, TypeOf, Value},
    r#type::StaticType,
    statement::{Assignment, Return},
    Block, Identifier, Template,
};

#[test]
fn strict_equals() {
    let eq = Equals::new(TypeOf::new(Identifier::new("val")), Value::from("object"));

    let expected = expect![[r#"
        typeof val === 'object'
    "#]];

    expect_ts(&eq, &expected);
}

#[test]
fn non_strict_equals() {
    let eq = Equals::new(Identifier::new("val"), Value::from(true)).non_strict();

    let expected = expect![[r#"
        val == true
    "#]];

    expect_ts(&eq, &expected);
}

#[test]
fn template_string() {
    let template = Template::new(indoc! {r#"
        This here is a long template with ${variable} definition.

        We can add newlines, and they are indented nicely.
    "#});

    let assignment = Assignment::new("text", template).r#const();

    let expected = expect![[r#"
        const text = `This here is a long template with ${variable} definition.

        We can add newlines, and they are indented nicely.
        `
    "#]];

    expect_ts(&assignment, &expected);
}

#[test]
fn empty_closure() {
    let closure = Closure::new(Default::default());

    let expected = expect![[r#"
        const fun = () => {
        }
    "#]];

    let assignment = Assignment::new("fun", closure).r#const();

    expect_ts(&assignment, &expected);
}

#[test]
fn closure_with_param() {
    let identifier = Identifier::new("a");

    let mut body = Block::new();
    body.push(Return::new(identifier.clone()));

    let closure = Closure::new(body).params(vec![identifier]);

    let expected = expect![[r#"
            const fun = (a) => {
              return a
            }
        "#]];

    let assignment = Assignment::new("fun", closure).r#const();

    expect_ts(&assignment, &expected);
}

#[test]
fn closure_with_params() {
    let a = Identifier::new("a");
    let b = Identifier::new("b");

    let mut body = Block::new();
    body.push(Return::new(Equals::new(a.clone(), b.clone())));

    let closure = Closure::new(body)
        .params(vec![a, b])
        .returns(StaticType::ident("boolean"));

    let expected = expect![[r#"
            const fun = (a, b): boolean => {
              return a === b
            }
        "#]];

    let assignment = Assignment::new("fun", closure).r#const();

    expect_ts(&assignment, &expected);
}

#[test]
fn closure_with_typed_params() {
    let identifier = Identifier::new("a");

    let mut body = Block::new();
    body.push(Return::new(identifier.clone()));

    let closure = Closure::new(body)
        .typed_params(vec![(identifier, StaticType::ident("string"))])
        .returns(StaticType::ident("string"));

    let expected = expect![[r#"
            const fun = (a: string): string => {
              return a
            }
        "#]];

    let assignment = Assignment::new("fun", closure).r#const();

    expect_ts(&assignment, &expected);
}
