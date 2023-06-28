use std::fmt;

use crate::{comment::CommentBlock, interface::Interface, r#type::Type, Function};

pub struct Export<'a> {
    kind: ExportKind<'a>,
    description: Option<CommentBlock<'a>>,
}

impl<'a> Export<'a> {
    pub fn new(kind: impl Into<ExportKind<'a>>) -> Self {
        Self {
            kind: kind.into(),
            description: None,
        }
    }

    pub fn description(&mut self, comment: impl Into<CommentBlock<'a>>) {
        self.description = Some(comment.into());
    }
}

impl<'a> fmt::Display for Export<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref comment) = self.description {
            writeln!(f, "{comment}")?;
        }

        write!(f, "export {}", self.kind)
    }
}

pub enum ExportKind<'a> {
    Interface(Interface<'a>),
    Type(Type<'a>),
    Function(Function<'a>),
}

impl<'a> From<Interface<'a>> for ExportKind<'a> {
    fn from(value: Interface<'a>) -> Self {
        Self::Interface(value)
    }
}

impl<'a> From<Type<'a>> for ExportKind<'a> {
    fn from(value: Type<'a>) -> Self {
        Self::Type(value)
    }
}

impl<'a> From<Function<'a>> for ExportKind<'a> {
    fn from(value: Function<'a>) -> Self {
        Self::Function(value)
    }
}

impl<'a> fmt::Display for ExportKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportKind::Interface(i) => i.fmt(f),
            ExportKind::Type(t) => t.fmt(f),
            ExportKind::Function(fun) => fun.fmt(f),
        }
    }
}
