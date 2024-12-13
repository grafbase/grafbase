use std::{fmt, fs, path::Path, sync::LazyLock};

static UPDATE_EXPECT: LazyLock<bool> = LazyLock::new(|| std::env::var("UPDATE_EXPECT").is_ok());

struct DiffError(String);

impl fmt::Debug for DiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for DiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for DiffError {}

fn run_test(case: &Path) -> datatest_stable::Result<()> {
    if cfg!(windows) {
        return Ok(()); // windows line endings make the spans in the snapshots different
    }

    let schemas = fs::read_to_string(case)?;
    let mut schemas = schemas.split("# --- #");
    let source = schemas.next().expect("Can't find first schema in test case.");
    let target = schemas.next().expect("Can't find second schema in test case.");

    let forward_diff = graphql_schema_diff::diff(source, target).unwrap();
    let backward_diff = graphql_schema_diff::diff(target, source).unwrap();

    let mut diff = serde_json::to_string_pretty(&serde_json::json!({
        "src → target": forward_diff,
        "target → src": backward_diff,
    }))
    .unwrap();

    if cfg!(windows) {
        diff = diff.replace("\r\n", "\n");
    }

    let snapshot_file_path = case.with_extension("snapshot.json");

    if *UPDATE_EXPECT {
        fs::write(&snapshot_file_path, &diff).unwrap();
        return Ok(());
    }

    let mut snapshot = fs::read_to_string(&snapshot_file_path).unwrap_or_default();

    if cfg!(windows) {
        snapshot = snapshot.replace("\r\n", "\n");
    }

    if snapshot != diff {
        return Err(DiffError(format!(
            "{}\n\n\n=== Hint: run the tests again with UPDATE_EXPECT=1 to update the snapshot. ===",
            similar::udiff::unified_diff(
                similar::Algorithm::default(),
                &snapshot,
                &diff,
                5,
                Some(("Snapshot", "Actual"))
            )
        ))
        .into());
    }

    Ok(())
}

datatest_stable::harness! {
    run_test, "./tests/diff", r"^.*\.graphql$",
}
