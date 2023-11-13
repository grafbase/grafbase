#![allow(unused_crate_dependencies)]

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

fn run_test(graphql_file_path: &Path) -> datatest_stable::Result<()> {
    init_miette();
    let schema = fs::read_to_string(graphql_file_path)?;
    let diagnostics = grafbase_validation::validate_with_options(
        &schema,
        grafbase_validation::Options::FORBID_EXTENDING_UNKNOWN_TYPES | grafbase_validation::Options::DRAFT_VALIDATIONS,
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

datatest_stable::harness! { run_test, "./tests/validation_errors", r"^.*\.graphql$" }
