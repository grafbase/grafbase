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
    ExperimentalFeatureRemoved(String),
    ExperimentalFeaturePromoted { feature: String, documentation: String },
    ExperimentalFeatureUnreleased { feature: String },
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
            Warning::ExperimentalFeatureRemoved(feature) => write!(
                f,
                "The experimental {feature} feature is not supported anymore, please remove it from your configuration"
            ),
            Warning::ExperimentalFeaturePromoted{
                feature,
                documentation
            } => write!(f, "The {feature} feature is not experimental anymore, please see {documentation} for details on how it is now configured"),
            Warning::ExperimentalFeatureUnreleased { feature } => write!(f, "The experimental {feature} feature is not releaseed yet, it may not work correctly or at all"),
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
