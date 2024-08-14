#![allow(unused_crate_dependencies)]

use std::{fs, path::Path, sync::LazyLock};

fn run_test(case: &Path) -> datatest_stable::Result<()> {
    let schemas = fs::read_to_string(case)?;
    let mut schemas = schemas.split("# --- #");
    let source = schemas.next().expect("Can't find first schema in test case.");
    let target = schemas.next().expect("Can't find second schema in test case.");

    let forward_diff = graphql_schema_diff::diff(source, target).unwrap();
    let backward_diff = graphql_schema_diff::diff(target, source).unwrap();

    // Applying the forward diff to source should give target.
    {
        let resolved_spans: Vec<_> = graphql_schema_diff::resolve_spans(source, target, &forward_diff).collect();
        graphql_schema_diff::patch(source, &forward_diff, &resolved_spans).unwrap();
    }

    todo!();
    // test that applying forward diff to source gives target, and then backwards we're back to source

    Ok(())
}
