use std::{
    borrow::Cow,
    fmt::{self, Write},
};

use super::{comment::CommentBlock, Identifier};

pub struct EnumVariant<'a> {
    identifier: Identifier<'a>,
    description: Option<CommentBlock<'a>>,
}

impl<'a> EnumVariant<'a> {
    pub fn new(identifier: impl Into<Cow<'a, str>>) -> Self {
        Self {
            identifier: Identifier::new(identifier),
            description: None,
        }
    }

    pub fn description(&mut self, comment: impl Into<CommentBlock<'a>>) {
        self.description = Some(comment.into())
    }
}

impl<'a> fmt::Display for EnumVariant<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref description) = self.description {
            writeln!(f, "{description}")?;
        }

        self.identifier.fmt(f)?;

        Ok(())
    }
}

pub struct Enum<'a> {
    name: Identifier<'a>,
    variants: Vec<EnumVariant<'a>>,
}

impl<'a> Enum<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: Identifier::new(name),
            variants: Vec::new(),
        }
    }

    pub fn push_variant(&mut self, variant: EnumVariant<'a>) {
        self.variants.push(variant);
    }
}

impl<'a> fmt::Display for Enum<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "enum {} {{", self.name)?;

        for (i, variant) in self.variants.iter().enumerate() {
            write!(f, "{variant}")?;

            if i < self.variants.len() - 1 {
                f.write_char(',')?;
            }

            writeln!(f)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}
