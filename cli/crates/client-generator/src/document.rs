use std::fmt;

use crate::{
    r#type::Type,
    statement::{Assignment, Export},
    Class, Function, Import, Interface,
};

#[derive(Default)]
pub struct Document<'a> {
    items: Vec<DocumentItem<'a>>,
}

impl<'a> Document<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_item(&mut self, item: impl Into<DocumentItem<'a>>) {
        self.items.push(item.into());
    }
}

impl<'a> fmt::Display for Document<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            writeln!(f, "{item}")?;
            writeln!(f)?;
        }

        Ok(())
    }
}

#[allow(dead_code)]
enum DocumentItemKind<'a> {
    Import(Import<'a>),
    Type(Type<'a>),
    Interface(Interface<'a>),
    Assignment(Assignment<'a>),
    Class(Class<'a>),
    Function(Function<'a>),
    Export(Export<'a>),
    Newline,
}

impl<'a> fmt::Display for DocumentItemKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentItemKind::Import(ref value) => value.fmt(f),
            DocumentItemKind::Type(ref value) => value.fmt(f),
            DocumentItemKind::Interface(ref value) => value.fmt(f),
            DocumentItemKind::Assignment(ref value) => value.fmt(f),
            DocumentItemKind::Class(ref value) => value.fmt(f),
            DocumentItemKind::Function(ref value) => value.fmt(f),
            DocumentItemKind::Export(ref value) => value.fmt(f),
            DocumentItemKind::Newline => writeln!(f),
        }
    }
}

pub struct DocumentItem<'a> {
    kind: DocumentItemKind<'a>,
}

impl<'a> fmt::Display for DocumentItem<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[allow(dead_code)]
impl DocumentItem<'static> {
    pub fn newline() -> Self {
        Self {
            kind: DocumentItemKind::Newline,
        }
    }
}

impl<'a> From<Import<'a>> for DocumentItem<'a> {
    fn from(value: Import<'a>) -> Self {
        Self {
            kind: DocumentItemKind::Import(value),
        }
    }
}

impl<'a> From<Type<'a>> for DocumentItem<'a> {
    fn from(value: Type<'a>) -> Self {
        Self {
            kind: DocumentItemKind::Type(value),
        }
    }
}

impl<'a> From<Interface<'a>> for DocumentItem<'a> {
    fn from(value: Interface<'a>) -> Self {
        Self {
            kind: DocumentItemKind::Interface(value),
        }
    }
}

impl<'a> From<Assignment<'a>> for DocumentItem<'a> {
    fn from(value: Assignment<'a>) -> Self {
        Self {
            kind: DocumentItemKind::Assignment(value),
        }
    }
}

impl<'a> From<Class<'a>> for DocumentItem<'a> {
    fn from(value: Class<'a>) -> Self {
        Self {
            kind: DocumentItemKind::Class(value),
        }
    }
}

impl<'a> From<Function<'a>> for DocumentItem<'a> {
    fn from(value: Function<'a>) -> Self {
        Self {
            kind: DocumentItemKind::Function(value),
        }
    }
}

impl<'a> From<Export<'a>> for DocumentItem<'a> {
    fn from(value: Export<'a>) -> Self {
        Self {
            kind: DocumentItemKind::Export(value),
        }
    }
}
