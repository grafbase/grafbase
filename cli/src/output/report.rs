use std::collections::BTreeMap;

use crate::{
    api::{
        self,
        graphql::mutations::{SchemaCheckDiagnostic, SchemaCheckStep},
    },
    common::{
        environment::{PlatformData, Warning},
        trusted_documents::TrustedDocumentsManifest,
    },
};
use crate::{
    errors::CliError,
    watercolor::{self, watercolor},
};
use crossterm::style::Stylize;
use extension::Manifest;

/// reports to stdout that the server has started
pub fn cli_header() {
    let version = env!("CARGO_PKG_VERSION");
    println!("{}", format!("Grafbase CLI {version}\n").green().bold());
}

/// reports an error to stderr
pub fn error(error: &CliError) {
    watercolor::output_error!("Error: {error}", @BrightRed);
    if let Some(hint) = error.to_hint() {
        watercolor::output_error!("Hint: {hint}", @BrightBlue);
    }
}

pub(crate) fn composition_diagnostics(diagnostics: &graphql_composition::Diagnostics) {
    for diagnostic in diagnostics.iter_warnings() {
        watercolor::output!("- ⚠️ Warning: {}", diagnostic, @BrightYellow);
        println!();
    }

    for diagnostic in diagnostics.iter_errors() {
        watercolor::output!("- ❌ Error: {}", diagnostic, @BrightRed);
        println!();
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

pub fn create_success(name: &str, urls: &[String], account_slug: &str, graph_slug: &str) {
    let platform_data = PlatformData::get();
    watercolor::output!("\n✨ {name} was successfully created!\n", @BrightBlue);
    if let Some(url) = urls.first() {
        watercolor::output!("Endpoint: https://{url}", @BrightBlue)
    }
    watercolor::output!("Dashboard: {}/{account_slug}/{graph_slug}/branches/main", platform_data.dashboard_url, @BrightBlue);
}

pub(crate) fn check_name_missing_on_federated_graph() {
    watercolor::output!("❌ The graph is federated, but you did not provide a subgraph name to check against. Please pass a subgraph name with the --name argument to the check command.", @BrightRed);
}

pub(crate) fn check_success() {
    watercolor::output!("\n✨ Successful check!", @BrightBlue);
}

pub(crate) fn check_errors(has_errors: bool, diagnostics: &[SchemaCheckDiagnostic]) {
    if has_errors {
        watercolor::output!("\nErrors were found in your schema check:", @BrightRed);
    } else {
        watercolor::output!("\nWarnings were found in your schema check:", @BrightYellow);
    }

    let mut sections: BTreeMap<SchemaCheckStep, Vec<&SchemaCheckDiagnostic>> = BTreeMap::new();

    for diagnostic in diagnostics {
        sections.entry(diagnostic.step).or_default().push(diagnostic);
    }

    for (step, diagnostics) in sections {
        let step_name = match step {
            SchemaCheckStep::Validation => "Validation",
            SchemaCheckStep::Composition => "Composition",
            SchemaCheckStep::Operation => "Operation",
            SchemaCheckStep::Lint => "Lint",
            SchemaCheckStep::Custom => "Custom",
            SchemaCheckStep::Proposal => "Proposal",
        };

        watercolor::output!("\n{step_name}\n", @BrightBlue);

        for diagnostic in diagnostics {
            let error = &diagnostic.message;

            match diagnostic.severity {
                api::check::SchemaCheckErrorSeverity::Error => {
                    watercolor::output!("❌ [Error] {error}", @BrightRed);
                }
                api::check::SchemaCheckErrorSeverity::Warning => {
                    watercolor::output!("⚠️ [Warning] {error}", @BrightYellow);
                }
            }
        }
    }
}

pub(crate) fn subgraph_list_command_success<'a>(branch_name: &str, subgraphs: impl ExactSizeIterator<Item = &'a str>) {
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

pub(crate) fn subgraph_delete_success(subgraph_name: &str) {
    println!("🗑️  Subgraph {subgraph_name} deleted successfully");
}

pub(crate) fn publish_no_change() {
    println!("🟰 The subgraph is already published with this schema and url. Publish skipped.")
}

pub(crate) fn publish_graph_does_not_exist(account_slug: &str, graph_slug: &str) {
    watercolor::output!("❌ Could not publish: there is no graph named {graph_slug} in the account {account_slug}\n", @BrightRed);
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

pub(crate) fn trust_reused_ids(reused: &api::submit_trusted_documents::ReusedIds) {
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

pub(crate) fn extension_build_start() {
    watercolor::output!("🔨 Building extension...", @BrightBlue);
}

pub(crate) fn extension_built(manifest: &Manifest) {
    let name = manifest.name();
    let version = manifest.version();
    let minimum_gateway_version = &manifest.minimum_gateway_version;
    let sdk_version = &manifest.sdk_version;

    watercolor::output!("✨ Extension {name} built successfully", @BrightGreen);
    println!();
    println!("- Extension version: {version}");
    println!("- Minimum Grafbase Gateway version: {minimum_gateway_version}");
    println!("- SDK version: {sdk_version}");
}

pub(crate) fn extension_update_extension_does_not_exist(name: &str) {
    watercolor::output!(r#"❌ Extension "{name}" does not exist"#, @BrightRed);
}

pub(crate) fn extension_update_extension_version_does_not_exist(name: &str, version_req: &semver::VersionReq) {
    watercolor::output!(r#"❌ No published version of extension "{name}" matches "{version_req}""#, @BrightRed);
}

pub(crate) fn extension_version_already_exists() {
    println!("❌ Extension version already exists");
}

pub(crate) fn extension_publish_failed(err: &str) {
    println!("❌ Failed to publish extension: {err}");
}

pub(crate) fn extension_published(name: &str, version: &str) {
    println!("🌟 Extension `{name}@{version}` published successfully");
}

pub(crate) fn extension_install_start() {
    watercolor::output!("Installing extensions...", @BrightWhite);
}

pub(crate) fn no_extension_defined_in_config() {
    watercolor::output!("No extensions defined in the configuration", @BrightGreen);
}
