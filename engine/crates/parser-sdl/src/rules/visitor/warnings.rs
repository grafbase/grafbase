use std::{collections::BTreeSet, fmt};

use colored::Colorize;

use crate::schema_coord::OwnedSchemaCoord;

#[derive(Clone, Debug, Default)]
pub(crate) struct Warnings {
    warnings: BTreeSet<Warning>,
}

impl Warnings {
    pub fn push(&mut self, warning: Warning) {
        self.warnings.insert(warning);
    }

    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
    }
}

impl Extend<Warning> for Warnings {
    fn extend<T: IntoIterator<Item = Warning>>(&mut self, iter: T) {
        for warning in iter {
            self.push(warning)
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub(crate) enum Warning {
    ArgumentNotUsedByJoin(String, OwnedSchemaCoord),
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Warning::ArgumentNotUsedByJoin(argument, field_coord) => {
                write!(
                    f,
                    "The argument {argument} of {field_coord} is unused by the join on that field"
                )
            }
        }
    }
}

impl fmt::Display for Warnings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", "Warnings:".bold().yellow())?;

        for warning in &self.warnings {
            writeln!(f, "  - {}", warning.to_string().yellow())?;
        }

        Ok(())
    }
}
