use std::{borrow::Cow, fmt};

use crate::typescript_ast::CommentBlock;

use super::{ObjectTypeDef, StaticType};

#[derive(Clone, Debug)]
pub struct Property<'a> {
    key: Cow<'a, str>,
    value: PropertyValue<'a>,
    optional: bool,
    description: Option<CommentBlock<'a>>,
}

impl<'a> Property<'a> {
    pub fn new(key: impl Into<Cow<'a, str>>, value: impl Into<PropertyValue<'a>>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            optional: false,
            description: None,
        }
    }

    pub fn optional(&mut self) {
        self.optional = true;
    }

    pub fn description(&mut self, comment: impl Into<CommentBlock<'a>>) {
        self.description = Some(comment.into());
    }
}

impl<'a> fmt::Display for Property<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref comment) = self.description {
            writeln!(f, "{comment}")?;
        }

        let optional = if self.optional { "?" } else { "" };
        write!(f, "{}{optional}: {}", self.key, self.value)
    }
}

#[derive(Clone, Debug)]
pub enum PropertyValue<'a> {
    Type(StaticType<'a>),
    Object(ObjectTypeDef<'a>),
}

impl<'a> From<StaticType<'a>> for PropertyValue<'a> {
    fn from(value: StaticType<'a>) -> Self {
        Self::Type(value)
    }
}

impl<'a> From<ObjectTypeDef<'a>> for PropertyValue<'a> {
    fn from(value: ObjectTypeDef<'a>) -> Self {
        Self::Object(value)
    }
}

impl<'a> fmt::Display for PropertyValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Type(ident) => ident.fmt(f),
            PropertyValue::Object(obj) => obj.fmt(f),
        }
    }
}
