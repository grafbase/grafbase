use std::{collections::BTreeSet, fmt};

use colored::Colorize;

#[derive(Clone, Debug, Default)]
pub(crate) struct Warnings {
    warnings: BTreeSet<Warning>,
}

impl Warnings {
    #[allow(dead_code)]
    pub fn push(&mut self, warning: Warning) {
        self.warnings.insert(warning);
    }

    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub(crate) enum Warning {}

impl AsRef<str> for Warning {
    fn as_ref(&self) -> &str {
        unreachable!()
    }
}

impl fmt::Display for Warnings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", "Warnings:".bold().yellow())?;

        for warning in &self.warnings {
            writeln!(f, "  - {}", warning.as_ref().yellow())?;
        }

        Ok(())
    }
}
