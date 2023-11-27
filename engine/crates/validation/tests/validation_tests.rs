#![allow(unused_crate_dependencies)]

use graphql_schema_validation::Options;
use std::{
    fs,
    path::Path,
    sync::{Once, OnceLock},
};

fn update_expect() -> bool {
    static UPDATE_EXPECT: OnceLock<bool> = OnceLock::new();
    *UPDATE_EXPECT.get_or_init(|| std::env::var("UPDATE_EXPECT").is_ok())
}

fn init_miette() {
    static MIETTE_SETUP: Once = Once::new();
    MIETTE_SETUP.call_once(|| {
        miette::set_hook(Box::new(|_| {
            Box::new(
                miette::GraphicalReportHandler::new()
                    .with_theme(miette::GraphicalTheme::unicode_nocolor())
                    .with_links(false)
                    .with_urls(true),
            )
        }))
        .unwrap();
    });
}

fn run_validation_error_test(graphql_file_path: &Path) -> datatest_stable::Result<()> {
    if cfg!(windows) {
        return Ok(()); // newlines
    }

    init_miette();
    let schema = fs::read_to_string(graphql_file_path)?;
    let diagnostics = graphql_schema_validation::validate_with_options(
        &schema,
        Options::FORBID_EXTENDING_UNKNOWN_TYPES | Options::DRAFT_VALIDATIONS,
    );
    let displayed = diagnostics
        .iter()
        .map(|d| format!("{d:?}"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let snapshot_path = graphql_file_path.with_extension("errors.txt");

    if update_expect() {
        fs::write(snapshot_path, displayed)?;
        return Ok(());
    }

    let snapshot = fs::read_to_string(snapshot_path).map_err(|_| {
        miette::miette!(
            "No snapshot found for {}\n\nErrors:\n\n{displayed}",
            graphql_file_path.display()
        )
    })?;

    if snapshot == displayed {
        return Ok(());
    }

    Err(miette::miette! {
        "The errors do not match the snapshot.\n\nExpected:\n{snapshot}\nGot:\n{displayed}\n\nhint: re-run the test with UPDATE_EXPECT=1 in the environment to update the snapshot"
    }
    .into())
}

fn run_valid_schema_test(graphql_file_path: &Path) -> datatest_stable::Result<()> {
    let schema = fs::read_to_string(graphql_file_path)?;

    let diagnostics = graphql_schema_validation::validate_with_options(
        &schema,
        Options::FORBID_EXTENDING_UNKNOWN_TYPES | Options::DRAFT_VALIDATIONS,
    );

    if diagnostics.has_errors() {
        let displayed = diagnostics
            .iter()
            .map(|d| format!("{d:?}"))
            .collect::<Vec<_>>()
            .join("\n\n");

        return Err(miette::miette!("{displayed}").into());
    }

    Ok(())
}

datatest_stable::harness! {
    run_validation_error_test, "./tests/validation_errors", r"^.*\.graphql$",
    run_valid_schema_test, "./tests/valid_schemas", r"^.*\.graphql$",
}
