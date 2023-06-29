use std::fmt;

use crate::typescript_ast::{Block, Expression};

pub struct Conditional<'a> {
    first_branch: (Expression<'a>, Block<'a>),
    branches: Vec<(Option<Expression<'a>>, Block<'a>)>,
}

#[allow(dead_code)]
impl<'a> Conditional<'a> {
    pub fn new(expr: impl Into<Expression<'a>>, block: impl Into<Block<'a>>) -> Self {
        Self {
            branches: Vec::new(),
            first_branch: (expr.into(), block.into()),
        }
    }

    pub fn else_if(&mut self, expr: impl Into<Expression<'a>>, block: impl Into<Block<'a>>) {
        self.branches.push((Some(expr.into()), block.into()));
    }

    pub fn r#else(&mut self, block: impl Into<Block<'a>>) {
        self.branches.push((None, block.into()));
    }
}

impl<'a> fmt::Display for Conditional<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if ({}) {}", self.first_branch.0, self.first_branch.1)?;

        for branch in &self.branches {
            f.write_str(" else")?;

            if let Some(ref condition) = branch.0 {
                write!(f, " if ({}) {}", condition, branch.1)?;
            } else {
                write!(f, " {}", branch.1)?;
            }
        }

        Ok(())
    }
}
