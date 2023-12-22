#![allow(unused_crate_dependencies)]

use std::{fs, path::Path, sync::OnceLock};

fn update_expect() -> bool {
    static UPDATE_EXPECT: OnceLock<bool> = OnceLock::new();
    *UPDATE_EXPECT.get_or_init(|| std::env::var("UPDATE_EXPECT").is_ok())
}

fn run_test(case: &Path) -> datatest_stable::Result<()> {
    let schemas = fs::read_to_string(case)?;
    let mut schemas = schemas.split("# --- #\n");
    let source = schemas.next().expect("Can't find first schema in test case.");
    let target = schemas.next().expect("Can't find second schema in test case.");

    let forward_diff = graphql_schema_diff::diff(source, target).unwrap();
    let backward_diff = graphql_schema_diff::diff(target, source).unwrap();

    let diff = serde_json::to_string_pretty(&serde_json::json!({
        "src → target": forward_diff,
        "target → src": backward_diff,
    }))
    .unwrap();

    let snapshot_file_path = case.with_extension("snapshot.json");

    if update_expect() {
        fs::write(&snapshot_file_path, &diff).unwrap();
        return Ok(());
    }

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
