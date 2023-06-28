use std::{borrow::Cow, fmt};

use crate::{common::Identifier, expression::Expression};

pub struct Object<'a> {
    entries: Vec<Entry<'a>>,
}

impl<'a> Object<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn entry(&mut self, key: impl Into<Cow<'a, str>>, value: impl Into<Expression<'a>>) {
        self.entries.push(Entry::new(Identifier::new(key), value))
    }
}

impl<'a> fmt::Display for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{ ")?;

        for (i, entry) in self.entries.iter().enumerate() {
            entry.fmt(f)?;

            if i < self.entries.len() - 1 {
                f.write_str(", ")?;
            }
        }

        f.write_str(" }")?;

        Ok(())
    }
}

pub struct Entry<'a> {
    key: Identifier<'a>,
    value: Expression<'a>,
}

impl<'a> Entry<'a> {
    pub fn new(key: Identifier<'a>, value: impl Into<Expression<'a>>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }
}

impl<'a> fmt::Display for Entry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}
