use std::fmt;

use crate::{interface::Interface, r#type::Type, Function};

#[derive(Debug)]
pub struct Export {
    kind: ExportKind,
}

impl Export {
    pub fn new(kind: impl Into<ExportKind>) -> Self {
        Self { kind: kind.into() }
    }
}

impl fmt::Display for Export {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "export {}", self.kind)
    }
}

#[derive(Debug)]
pub enum ExportKind {
    Interface(Interface),
    Type(Type),
    Function(Function),
}

impl From<Interface> for ExportKind {
    fn from(value: Interface) -> Self {
        Self::Interface(value)
    }
}

impl From<Type> for ExportKind {
    fn from(value: Type) -> Self {
        Self::Type(value)
    }
}

impl From<Function> for ExportKind {
    fn from(value: Function) -> Self {
        Self::Function(value)
    }
}

impl fmt::Display for ExportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportKind::Interface(i) => i.fmt(f),
            ExportKind::Type(t) => t.fmt(f),
            ExportKind::Function(fun) => fun.fmt(f),
        }
    }
}
