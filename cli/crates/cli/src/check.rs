use crate::{cli_input::CheckCommand, errors::CliError, report};
use backend::api::check;
use std::{
    fs,
    io::{IsTerminal, Read},
    process::Command,
};

const FAILED_CHECK_EXIT_STATUS: i32 = 1;

#[tokio::main]
pub(crate) async fn check(command: CheckCommand) -> Result<(), CliError> {
    let CheckCommand {
        graph_ref,
        subgraph_name,
        schema,
    } = command;

    let git_commit = find_git_commit();

    let schema = match schema {
        Some(schema) => fs::read_to_string(schema).map_err(CliError::SchemaReadError)?,
        None if std::io::stdin().is_terminal() => {
            return Err(CliError::MissingArgument("--schema or a schema piped through stdin"))
        }
        None => {
            let mut schema = String::new();

            std::io::stdin()
                .read_to_string(&mut schema)
                .map_err(CliError::SchemaReadError)?;

            schema
        }
    };

    report::checking();

    let result = check::check(
        graph_ref.account(),
        graph_ref.graph(),
        graph_ref.branch(),
        &subgraph_name,
        &schema,
        git_commit,
    )
    .await
    .map_err(CliError::BackendApiError)?;

    let check::SchemaCheck {
        id: _,
        error_count,
        validation_check_errors,
        composition_check_errors,
        operation_check_errors,
        lint_check_errors,
    } = match result {
        check::SchemaCheckResult::Ok(check) => check,
        check::SchemaCheckResult::SubgraphNameMissingOnFederatedProjectError => {
            report::check_name_missing_on_federated_project();
            std::process::exit(1);
        }
    };

    if validation_check_errors.is_empty()
        && composition_check_errors.is_empty()
        && operation_check_errors.is_empty()
        && lint_check_errors.is_empty()
    {
        report::check_success();
    } else {
        report::check_errors(
            error_count != 0,
            validation_check_errors.iter().map(|err| err.message.as_str()),
            composition_check_errors.iter().map(|err| err.message.as_str()),
            operation_check_errors
                .iter()
                .filter(|err| matches!(err.severity, check::SchemaCheckErrorSeverity::Error))
                .map(|err| err.message.as_str()),
            lint_check_errors
                .iter()
                .filter(|err| matches!(err.severity, check::SchemaCheckErrorSeverity::Error))
                .map(|err| err.message.as_str()),
            operation_check_errors
                .iter()
                .filter(|err| matches!(err.severity, check::SchemaCheckErrorSeverity::Warning))
                .map(|err| err.message.as_str()),
            lint_check_errors
                .iter()
                .filter(|err| matches!(err.severity, check::SchemaCheckErrorSeverity::Warning))
                .map(|err| err.message.as_str()),
        );

        if error_count > 0 {
            std::process::exit(FAILED_CHECK_EXIT_STATUS);
        }
    }

    Ok(())
}

fn find_git_commit() -> Option<check::SchemaCheckGitCommitInput> {
    let git_author = git_author();
    let git_sha = git_sha();
    let git_branch = git_branch();
    let git_message = git_commit_message();

    git_author
        .zip(git_sha)
        .zip(git_branch)
        .zip(git_message)
        .map(|(((author, sha), branch), message)| check::SchemaCheckGitCommitInput {
            author_name: author,
            commit_sha: sha,
            branch,
            message,
        })
}

fn git_author() -> Option<String> {
    let output = Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("user.name")
        .output()
        .ok()?;

    String::from_utf8(output.stdout).ok()
}

fn git_sha() -> Option<String> {
    let output = Command::new("git").arg("rev-parse").arg("HEAD").output().ok()?;

    String::from_utf8(output.stdout).ok()
}

fn git_branch() -> Option<String> {
    let output = Command::new("git").arg("branch").arg("--show-current").output().ok()?;

    String::from_utf8(output.stdout).ok()
}

fn git_commit_message() -> Option<String> {
    let output = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--pretty=format:%s")
        .output()
        .ok()?;

    String::from_utf8(output.stdout).ok()
}
