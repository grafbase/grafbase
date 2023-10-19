use std::{collections::BTreeSet, fmt};

use colored::Colorize;

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

    #[cfg(test)]
    pub fn uses_deprecated_models(&self) -> bool {
        self.warnings.contains(&Warning::DeprecatedModelDefinition)
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub(crate) enum Warning {
    DeprecatedModelDefinition,
}

impl AsRef<str> for Warning {
    fn as_ref(&self) -> &str {
        match self {
            Warning::DeprecatedModelDefinition => "The Grafbase database is deprecated and will be sunset soon. Use connectors like Postgres or MongoDB instead.",
        }
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
