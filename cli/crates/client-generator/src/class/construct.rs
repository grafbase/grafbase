use core::fmt;
use std::fmt::Write;

use crate::{Expression, Identifier};

pub struct Construct<'a> {
    identifier: Identifier<'a>,
    params: Vec<Expression<'a>>,
}

impl<'a> Construct<'a> {
    pub fn new(identifier: impl Into<Identifier<'a>>) -> Self {
        Self {
            identifier: identifier.into(),
            params: Vec::new(),
        }
    }

    pub fn push_param(&mut self, param: impl Into<Expression<'a>>) {
        self.params.push(param.into());
    }
}

impl<'a> fmt::Display for Construct<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "new {}(", self.identifier)?;

        for (i, param) in self.params.iter().enumerate() {
            param.fmt(f)?;

            if i < self.params.len() - 1 {
                f.write_str(", ")?;
            }
        }

        f.write_char(')')?;

        Ok(())
    }
}
