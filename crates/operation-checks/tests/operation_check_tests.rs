#![allow(unused_crate_dependencies)]

use operation_checks::CheckParams;
use std::{fs, path::Path, sync::OnceLock};

fn update_expect() -> bool {
    static UPDATE_EXPECT: OnceLock<bool> = OnceLock::new();
    *UPDATE_EXPECT.get_or_init(|| std::env::var("UPDATE_EXPECT").is_ok())
}

fn run_test(case_path: &Path) -> datatest_stable::Result<()> {
    let case = fs::read_to_string(case_path)?;
    let mut sections = case.split("# --- #");

    let source_schema_string = sections.next().expect("source schema section missing");
    let source_schema: operation_checks::Schema = async_graphql_parser::parse_schema(source_schema_string)?.into();

    let target_schema_string = sections.next().expect("target schema section missing");
    let target_schema: operation_checks::Schema = async_graphql_parser::parse_schema(target_schema_string)?.into();

    let mut field_usage = operation_checks::FieldUsage::default();

    for operation in sections {
        let parsed_query = async_graphql_parser::parse_query(operation)?;
        let operation = operation_checks::Operation::from(parsed_query);
        operation_checks::aggregate_field_usage(&operation, &source_schema, &mut field_usage);
    }

    let [result_forward, result_backward] = [
        (
            &source_schema_string,
            &source_schema,
            &target_schema_string,
            &target_schema,
        ),
        (
            &target_schema_string,
            &target_schema,
            &source_schema_string,
            &source_schema,
        ),
    ]
    .map(|(source_str, source, target_str, target)| {
        let diff = graphql_schema_diff::diff(source_str, target_str).unwrap();
        let params = CheckParams {
            source,
            target,
            diff: &diff,
            field_usage: &field_usage,
        };
        operation_checks::check(&params)
    });

    let snapshot_path = case_path.with_extension("snapshot");

    let mut result = format!("Forward:\n{result_forward:#?}\n\nBackward:\n{result_backward:#?}\n",);

    if cfg!(windows) {
        result = result.replace("\r\n", "\n");
    }

    if update_expect() {
        fs::write(&snapshot_path, result)?;
        return Ok(());
    }

    let mut snapshot = fs::read_to_string(&snapshot_path).unwrap_or_default();

    if cfg!(windows) {
        snapshot = snapshot.replace("\r\n", "\n");
    }

    if snapshot != result {
        return Err(miette::miette!(
            "{}\n\n\n=== Hint: run the tests again with UPDATE_EXPECT=1 to update the snapshot. ===",
            similar::udiff::unified_diff(
                similar::Algorithm::default(),
                &snapshot,
                &result,
                5,
                Some(("Snapshot", "Actual"))
            )
        )
        .into());
    }

    Ok(())
}

datatest_stable::harness! {
    run_test, "./tests/cases", r"^.*\.graphql$",
}
