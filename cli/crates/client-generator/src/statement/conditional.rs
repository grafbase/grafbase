use std::fmt;

use crate::{Block, Expression};

#[derive(Debug)]
pub struct Conditional {
    first_branch: (Expression, Block),
    branches: Vec<(Option<Expression>, Block)>,
}

impl Conditional {
    pub fn new(expr: impl Into<Expression>, block: impl Into<Block>) -> Self {
        Self {
            branches: Vec::new(),
            first_branch: (expr.into(), block.into()),
        }
    }

    pub fn else_if(&mut self, expr: impl Into<Expression>, block: impl Into<Block>) {
        self.branches.push((Some(expr.into()), block.into()));
    }

    pub fn r#else(&mut self, block: impl Into<Block>) {
        self.branches.push((None, block.into()));
    }
}

impl fmt::Display for Conditional {
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
