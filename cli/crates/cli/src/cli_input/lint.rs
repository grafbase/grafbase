use clap::Parser;
use std::path::PathBuf;

/// Lint a GraphQL schema
#[derive(Debug, Parser)]
pub struct LintCommand {
    /// The path of the schema to lint
    pub schema: Option<PathBuf>,
}
