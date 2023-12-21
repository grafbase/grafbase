#![allow(unused_crate_dependencies)]

use std::{fs, path::Path};

fn run_test(case: &Path) -> datatest_stable::Result<()> {
    let schemas = fs::read_to_string(case)?;
    let mut schemas = schemas.split("# --- #\n");
    let source = schemas.next().expect("Can't find first schema in test case.");
    let target = schemas.next().expect("Can't find second schema in test case.");

    dbg!(source, target);
    let diff = format!("{:#?}", graphql_schema_diff::diff(source, target));

    let snapshot_file_path = case.with_extension("diff.snapshot");
    let snapshot = fs::read_to_string(&snapshot_file_path).unwrap_or_default();

    if snapshot != diff {
        return Err(miette::miette!(
            "{}\n\n\n=== Hint: run the tests again with UPDATE_EXPECT=1 to update the snapshot. ===",
            similar::udiff::unified_diff(
                similar::Algorithm::default(),
                &snapshot,
                &diff,
                5,
                Some(("Snapshot", "Actual"))
            )
        )
        .into());
    }

    Ok(())
}

datatest_stable::harness! {
    run_test, "./tests/diff", r"^.*\.graphql$",
}
