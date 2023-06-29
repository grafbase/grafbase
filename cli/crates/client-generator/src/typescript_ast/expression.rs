mod closure;
mod equals;
mod r#typeof;
mod value;

pub use closure::Closure;
pub use equals::Equals;
pub use r#typeof::TypeOf;
pub use value::{Object, Value};

use std::fmt;

use super::{Identifier, Template};

pub struct Expression<'a> {
    kind: ExpressionKind<'a>,
}

impl<'a> fmt::Display for Expression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ExpressionKind::Variable(ref v) => v.fmt(f),
            ExpressionKind::Value(ref v) => v.fmt(f),
            ExpressionKind::TypeOf(ref v) => v.fmt(f),
            ExpressionKind::Equals(ref v) => v.fmt(f),
            ExpressionKind::Closure(ref v) => v.fmt(f),
        }
    }
}

impl<'a, T> From<T> for Expression<'a>
where
    T: Into<ExpressionKind<'a>>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

pub enum ExpressionKind<'a> {
    Variable(Identifier<'a>),
    Value(Value<'a>),
    TypeOf(Box<TypeOf<'a>>),
    Equals(Box<Equals<'a>>),
    Closure(Closure<'a>),
}

impl<'a> From<Value<'a>> for ExpressionKind<'a> {
    fn from(value: Value<'a>) -> Self {
        Self::Value(value)
    }
}

impl<'a> From<Identifier<'a>> for ExpressionKind<'a> {
    fn from(value: Identifier<'a>) -> Self {
        Self::Variable(value)
    }
}

impl<'a> From<TypeOf<'a>> for ExpressionKind<'a> {
    fn from(value: TypeOf<'a>) -> Self {
        Self::TypeOf(Box::new(value))
    }
}

impl<'a> From<Equals<'a>> for ExpressionKind<'a> {
    fn from(value: Equals<'a>) -> Self {
        Self::Equals(Box::new(value))
    }
}

impl<'a> From<Object<'a>> for ExpressionKind<'a> {
    fn from(value: Object<'a>) -> Self {
        Self::Value(Value::from(value))
    }
}

impl<'a> From<Template<'a>> for ExpressionKind<'a> {
    fn from(value: Template<'a>) -> Self {
        Self::Value(Value::from(value))
    }
}

impl<'a> From<Closure<'a>> for ExpressionKind<'a> {
    fn from(value: Closure<'a>) -> Self {
        Self::Closure(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        test_helpers::{expect, expect_ts, indoc},
        typescript_ast::{Assignment, Block, Identifier, Return, StaticType, Template},
    };

    use super::{Closure, Equals, TypeOf, Value};

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
}
