use graphql_schema_validation::Options;
use std::{fs, path::Path, sync::Once};

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

fn run_validation_error_test(graphql_file_path: &Path) {
    if cfg!(windows) {
        return; // newlines
    }

    init_miette();
    let schema = fs::read_to_string(graphql_file_path).unwrap();
    let diagnostics = graphql_schema_validation::validate_with_options(
        &schema,
        Options::FORBID_EXTENDING_UNKNOWN_TYPES | Options::DRAFT_VALIDATIONS,
    );
    let displayed = diagnostics
        .iter()
        .map(|d| format!("{d:?}"))
        .collect::<Vec<_>>()
        .join("\n\n");

    insta::assert_snapshot!("errors", displayed);
}

fn run_valid_schema_test(graphql_file_path: &Path) {
    let schema = fs::read_to_string(graphql_file_path).unwrap();

    let diagnostics = graphql_schema_validation::validate_with_options(
        &schema,
        Options::FORBID_EXTENDING_UNKNOWN_TYPES | Options::DRAFT_VALIDATIONS,
    );

    assert!(
        !diagnostics.has_errors(),
        "Expected no errors, but got:\n{}",
        diagnostics
            .iter()
            .map(|d| format!("{d:?}"))
            .collect::<Vec<_>>()
            .join("\n\n")
    );
}

#[test]
fn validation_error_tests() {
    insta::glob!("validation_errors/**/*.graphql", |graphql_file_path| {
        let snapshot_path = graphql_file_path.parent().unwrap();
        let test_name = graphql_file_path.file_stem().unwrap().to_str().unwrap();
        insta::with_settings!({
            snapshot_path => snapshot_path.to_str().unwrap(),
            prepend_module_to_snapshot => false,
            snapshot_suffix => test_name,
        }, {
            run_validation_error_test(graphql_file_path);
        });
    });
}

#[test]
fn valid_schema_tests() {
    insta::glob!("valid_schemas/**/*.graphql", |graphql_file_path| {
        run_valid_schema_test(graphql_file_path);
    });
}
