#![allow(clippy::panic)] // this is tests, panicking signals failure

use graphql_parser as _;
use std::{fs, path::Path, sync::OnceLock};

fn update_expect() -> bool {
    static UPDATE_EXPECT: OnceLock<bool> = OnceLock::new();
    *UPDATE_EXPECT.get_or_init(|| std::env::var("UPDATE_EXPECT").is_ok())
}

#[allow(clippy::unnecessary_wraps)] // we can't change the signature expected by datatest_stable
fn run_test(path: &Path) -> datatest_stable::Result<()> {
    let graphql = fs::read_to_string(path).unwrap();
    let expected_file_path = path.with_extension("expected.ts");
    let mut expected = fs::read_to_string(&expected_file_path).unwrap_or_default();
    let generated = {
        let mut out = String::with_capacity(graphql.len() / 2);
        typed_resolvers::generate_ts_resolver_types(&graphql, &mut out).unwrap();
        out
    };

    if cfg!(target_os = "windows") {
        expected.retain(|c| c != '\r');
    }

    if generated == expected {
        return Ok(());
    }

    if update_expect() {
        std::fs::write(expected_file_path, &generated).unwrap();
        return Ok(());
    }

    panic!(
        "{}\n\n\n=== Hint: run the tests again with UPDATE_EXPECT=1 to update the snapshot. ===",
        similar::udiff::unified_diff(
            similar::Algorithm::default(),
            &expected,
            &generated,
            5,
            Some(("Expected", "Actual"))
        )
    );
}

datatest_stable::harness! { run_test, "./tests/schema_types", ".*\\.graphql$" }
