use crate::{errors::CliError, output::report};
use graphql_lint::Severity;
use std::{borrow::Borrow, fs, path::PathBuf};

const ALLOWED_EXTENSIONS: [&str; 4] = ["gql", "graphql", "graphqls", "sdl"];

pub fn lint(schema_path: PathBuf) -> Result<(), CliError> {
    let extension = schema_path
        .extension()
        .ok_or(CliError::LintNoExtension)?
        .to_string_lossy();

    if !ALLOWED_EXTENSIONS.contains(&extension.borrow()) {
        return Err(CliError::LintUnsupportedFileExtension(extension.into_owned()));
    }

    let schema = fs::read_to_string(&schema_path).map_err(|error| CliError::ReadLintSchema(schema_path, error))?;
    let diagnostics = graphql_lint::lint(&schema)?;

    if diagnostics.is_empty() {
        report::lint_success();
        return Ok(());
    }

    for (message, severity) in diagnostics {
        match severity {
            Severity::Warning => report::lint_warning(message),
        }
    }

    Ok(())
}
