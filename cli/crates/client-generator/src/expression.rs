mod closure;
mod equals;
mod r#typeof;
mod value;

pub use closure::Closure;
pub use equals::Equals;
pub use r#typeof::TypeOf;
pub use value::{Object, Value};

use crate::{common::Identifier, Template};
use std::fmt;

#[derive(Debug)]
pub struct Expression {
    kind: ExpressionKind,
}

impl fmt::Display for Expression {
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

impl<T> From<T> for Expression
where
    T: Into<ExpressionKind>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

#[derive(Debug)]
pub enum ExpressionKind {
    Variable(Identifier),
    Value(Value),
    TypeOf(Box<TypeOf>),
    Equals(Box<Equals>),
    Closure(Closure),
}

impl From<Value> for ExpressionKind {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

impl From<Identifier> for ExpressionKind {
    fn from(value: Identifier) -> Self {
        Self::Variable(value)
    }
}

impl From<TypeOf> for ExpressionKind {
    fn from(value: TypeOf) -> Self {
        Self::TypeOf(Box::new(value))
    }
}

impl From<Equals> for ExpressionKind {
    fn from(value: Equals) -> Self {
        Self::Equals(Box::new(value))
    }
}

impl From<Object> for ExpressionKind {
    fn from(value: Object) -> Self {
        Self::Value(Value::from(value))
    }
}

impl From<Template> for ExpressionKind {
    fn from(value: Template) -> Self {
        Self::Value(Value::from(value))
    }
}

impl From<Closure> for ExpressionKind {
    fn from(value: Closure) -> Self {
        Self::Closure(value)
    }
}
