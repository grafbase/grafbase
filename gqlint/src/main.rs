use clap::Parser;
use colored::Colorize;
use graphql_lint::{lint, LinterError, Severity};
use std::{fs, path::PathBuf, process};

#[derive(Debug, Parser)]
#[command(name = "gqlint", version)]
struct Interface {
    /// The GraphQL SDL file to lint
    schema: PathBuf,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Could not read the provided schema file\nCaused by: {0}")]
    ReadSchemaFile(#[from] std::io::Error),
    #[error(transparent)]
    Lint(#[from] LinterError),
}

fn report_error(error: Error) {
    eprintln!("{}", format!("Error: {error}").bright_red());
}

fn report_diagnostic(message: String, severity: Severity) {
    let label = match severity {
        Severity::Warning => "Warning",
    };
    println!("{}", format!("⚠️ [{label}]: {message}").bright_yellow());
}

fn report_success() {
    println!("{}", "✅ No issues were found in your schema".bright_green())
}

fn main() {
    let arguments = Interface::parse();

    let exit_code = match try_main(arguments) {
        Ok(()) => 0,
        Err(error) => {
            report_error(error);
            1
        }
    };

    process::exit(exit_code);
}

fn try_main(arguments: Interface) -> Result<(), Error> {
    let schema = fs::read_to_string(arguments.schema)?;
    let diagnostics = lint(&schema)?;

    if diagnostics.is_empty() {
        report_success();
        return Ok(());
    }

    for (message, severity) in diagnostics {
        report_diagnostic(message, severity);
    }

    Ok(())
}

// blah blah blah I am a very important change
