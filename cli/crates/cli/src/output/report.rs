use crate::{
    errors::CliError,
    watercolor::{self, watercolor},
};
use colored::Colorize;
use common::{
    environment::{PlatformData, Warning},
    trusted_documents::TrustedDocumentsManifest,
};

/// reports to stdout that the server has started
pub fn cli_header() {
    let version = env!("CARGO_PKG_VERSION");
    // TODO: integrate this with watercolor
    println!("{}", format!("Grafbase CLI {version}\n").dimmed());
}

/// reports an error to stderr
pub fn error(error: &CliError) {
    watercolor::output_error!("Error: {error}", @BrightRed);
    if let Some(hint) = error.to_hint() {
        watercolor::output_error!("Hint: {hint}", @BrightBlue);
    }
}

pub fn warnings(warnings: &[Warning]) {
    for warning in warnings {
        let msg = warning.message();

        watercolor::output!("Warning: {msg}", @BrightYellow);

        if let Some(hint) = warning.hint() {
            watercolor::output!("Hint: {hint}", @BrightBlue);
        }

        println!();
    }
}

pub fn login(url: &str) {
    println!(
        "Please continue by opening the following URL:\n{}\n",
        watercolor!("{url}", @BrightBlue)
    );
}

pub fn login_success() {
    watercolor::output!("\n\n✨ Successfully logged in!", @BrightBlue);
}

// TODO: better handling of spinner position to avoid this extra function
pub fn login_error(error: &CliError) {
    watercolor::output!("\n\nError: {error}", @BrightRed);
    if let Some(hint) = error.to_hint() {
        watercolor::output!("Hint: {hint}", @BrightBlue);
    }
}

pub fn logout() {
    watercolor::output!("✨ Successfully logged out!", @BrightBlue);
}

pub fn delete_branch() {
    watercolor::output!("🕒 Branch is being deleted...", @BrightBlue);
}

pub fn delete_branch_success() {
    watercolor::output!("\n✨ The branch was successfully deleted!", @BrightBlue);
}

pub fn create_branch() {
    watercolor::output!("🕒 Branch is being created...", @BrightBlue);
}

pub fn create_branch_success() {
    watercolor::output!("\n✨ The branch was successfully created!", @BrightBlue);
}

// TODO change this to a spinner that is removed on success
pub fn create() {
    watercolor::output!("🕒 Your graph is being created...", @BrightBlue);
}

pub fn create_success(name: &str, urls: &[String], account_slug: &str, project_slug: &str) {
    let platform_data = PlatformData::get();
    watercolor::output!("\n✨ {name} was successfully created!\n", @BrightBlue);
    if let Some(url) = urls.first() {
        watercolor::output!("Endpoint: https://{url}", @BrightBlue)
    }
    watercolor::output!("Dashboard: {}/{account_slug}/{project_slug}/branches/main", platform_data.dashboard_url, @BrightBlue);
}

pub(crate) fn check_name_missing_on_federated_project() {
    watercolor::output!("❌ The project is federated, but you did not provide a subgraph name to check against. Please pass a subgraph name with the --name argument to the check command.", @BrightRed);
}

pub(crate) fn check_success() {
    watercolor::output!("\n✨ Successful check!", @BrightBlue);
}

pub(crate) fn check_errors<'a>(
    has_errors: bool,
    validation_errors: impl ExactSizeIterator<Item = &'a str>,
    composition_errors: impl ExactSizeIterator<Item = &'a str>,
    operation_errors: impl Iterator<Item = &'a str>,
    lint_errors: impl Iterator<Item = &'a str>,
    operation_warnings: impl Iterator<Item = &'a str>,
    lint_warnings: impl Iterator<Item = &'a str>,
) {
    if has_errors {
        watercolor::output!("\nErrors were found in your schema check:", @BrightRed);
    } else {
        watercolor::output!("\nWarnings were found in your schema check:", @BrightYellow);
    }

    if validation_errors.len() > 0 {
        watercolor::output!("\nValidation\n", @BrightBlue);
        for error in validation_errors {
            watercolor::output!("❌ [Error] {error}", @BrightRed);
        }
    }

    let mut lint_errors = lint_errors.peekable();
    let mut lint_warnings = lint_warnings.peekable();
    if lint_errors.peek().is_some() || lint_warnings.peek().is_some() {
        watercolor::output!("\nLint\n", @BrightBlue);
        for warning in lint_warnings {
            watercolor::output!("⚠️ [Warning] {warning}", @BrightYellow);
        }
        for error in lint_errors {
            watercolor::output!("❌ [Error] {error}", @BrightRed);
        }
    }

    if composition_errors.len() > 0 {
        watercolor::output!("\nComposition\n", @BrightBlue);
        for error in composition_errors {
            watercolor::output!("❌ [Error] {error}", @BrightRed);
        }
    }

    let mut operation_errors = operation_errors.peekable();
    let mut operation_warnings = operation_warnings.peekable();
    if operation_errors.peek().is_some() || operation_warnings.peek().is_some() {
        watercolor::output!("\nOperation\n", @BrightBlue);
        for warning in operation_warnings {
            watercolor::output!("⚠️ [Warning] {warning}", @BrightYellow);
        }
        for error in operation_errors {
            watercolor::output!("❌ [Error] {error}", @BrightRed);
        }
    }
}

pub(crate) fn subgraphs_command_success<'a>(branch_name: &str, subgraphs: impl ExactSizeIterator<Item = &'a str>) {
    if subgraphs.len() == 0 {
        println!("🈳 There are no published subgraphs in the {branch_name} branch\n");
        return;
    }

    println!("Subgraphs in branch \"{branch_name}\":\n");

    for name in subgraphs {
        println!("-  {name}");
    }

    println!();
}

pub(crate) fn schema_command_success(schema: Option<&str>) {
    if let Some(schema) = schema {
        print!("{schema}");
    } else {
        eprintln!("🤲 Found no schema");
    }
}

pub(crate) fn checking() {
    println!("⏳ Checking...");
}

pub(crate) fn publishing() {
    println!("⏳ Publishing...");
}

pub(crate) fn publish_command_success(subgraph_name: &str) {
    println!("🧩 {subgraph_name} published successfully");
}

pub(crate) fn publish_graph_does_not_exist(account_slug: &str, project_slug: &str) {
    watercolor::output!("❌ Could not publish: there is no project named {project_slug} in the account {account_slug}\n", @BrightRed);
}

pub(crate) fn publish_command_composition_failure(messages: &[String]) {
    assert_matches::assert_matches!(messages, [_, ..]);

    let with_what = if messages.len() == 1 {
        "a composition error"
    } else {
        "composition errors"
    };
    watercolor::output!("🔴 Published with {with_what}.\n", @BrightRed);

    watercolor::output!("Composition errors:", @BrightRed);
    for error in messages {
        watercolor::output!("- {error}", @BrightRed);
    }
}

pub(crate) fn trust_start(manifest: &TrustedDocumentsManifest) {
    let format = match manifest {
        TrustedDocumentsManifest::Apollo(_) => "apollo",
        TrustedDocumentsManifest::Relay(_) => "relay",
    };
    watercolor::output!("📡 Submitting trusted documents manifest (format: {format})...", @BrightBlue);
}

pub(crate) fn trust_success(count: i32) {
    watercolor::output!("✨ Successfully submitted {count} documents", @BrightGreen)
}

pub(crate) fn trust_failed() {
    watercolor::output!("❌ Trusted document submission failed", @BrightRed)
}

pub(crate) fn old_access_token() {
    watercolor::output!("❌ You must pass a graph reference of the form <account>/<graph>@<branch> (missing account)", @BrightRed)
}

pub(crate) fn trust_reused_ids(reused: &backend::api::submit_trusted_documents::ReusedIds) {
    watercolor::output!("Error: there already exist trusted documents with the same ids, but a different body:", @BrightRed);

    for reused_id in &reused.reused {
        let id = &reused_id.document_id;
        watercolor::output!("- {id}", @BrightRed);
    }
}

pub(crate) fn upgrade_up_to_date(version: &str) {
    watercolor::output!("✅ The locally installed version ({version}) is already up to date", @BrightGreen)
}

pub(crate) fn lint_success() {
    watercolor::output!("✅ No issues found in your schema", @BrightGreen)
}

pub(crate) fn lint_warning(warning: String) {
    watercolor::output!("⚠️ [Warning] {warning}", @BrightYellow);
}
