use std::{borrow::Cow, fmt};

use crate::{common::Identifier, expression::Expression};

#[derive(Debug)]
pub struct Object {
    entries: Vec<Entry>,
}

impl Object {
    #[must_use]
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn entry(&mut self, key: impl Into<Cow<'static, str>>, value: impl Into<Expression>) {
        self.entries.push(Entry::new(Identifier::new(key), value))
    }
}

impl fmt::Display for Object {
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

#[derive(Debug)]
pub struct Entry {
    key: Identifier,
    value: Expression,
}

impl Entry {
    pub fn new(key: Identifier, value: impl Into<Expression>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}
