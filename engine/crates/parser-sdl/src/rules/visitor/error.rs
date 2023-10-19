use std::fmt;

use engine_parser::Pos;

#[derive(Debug, PartialEq, Eq)]
pub struct RuleError {
    pub(crate) locations: Vec<Pos>,
    pub(crate) message: String,
}

impl RuleError {
    pub(crate) fn new(locations: Vec<Pos>, msg: impl Into<String>) -> Self {
        Self {
            locations,
            message: msg.into(),
        }
    }
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, loc) in self.locations.iter().enumerate() {
            if idx == 0 {
                write!(f, "[")?;
            } else {
                write!(f, ", ")?;
            }

            write!(f, "{}:{}", loc.line, loc.column)?;

            if idx == self.locations.len() - 1 {
                write!(f, "] ")?;
            }
        }

        write!(f, "{}", self.message)?;
        Ok(())
    }
}
