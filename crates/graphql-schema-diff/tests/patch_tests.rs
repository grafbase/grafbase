use std::{fmt, fs, path::Path};

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

fn read_schemas(case: &Path) -> datatest_stable::Result<(String, String)> {
    let schemas = fs::read_to_string(case)?;
    let mut schemas = schemas.split("# --- #");
    let source = schemas.next().expect("Can't find first schema in test case.");
    let target = schemas.next().expect("Can't find second schema in test case.");

    Ok((source.to_owned(), target.to_owned()))
}

fn run_test_backwards(case: &Path) -> datatest_stable::Result<()> {
    let (source, target) = read_schemas(case)?;
    run_test_impl(target, source)
}

fn run_test(case: &Path) -> datatest_stable::Result<()> {
    let (source, target) = read_schemas(case)?;
    run_test_impl(source, target)
}

fn run_test_impl(source: String, target: String) -> datatest_stable::Result<()> {
    if cfg!(windows) {
        return Ok(()); // windows line endings make things complicated
    }

    let diff = graphql_schema_diff::diff(&source, &target).unwrap();

    // Applying the diff to source should give target.
    {
        let resolved_spans: Vec<_> = graphql_schema_diff::resolve_spans(&source, &target, &diff).collect();
        let patched = graphql_schema_diff::patch(&source, &diff, &resolved_spans).unwrap();

        if patched.schema().trim() != target.trim() {
            return Err(DiffError(
                similar::udiff::unified_diff(
                    similar::Algorithm::default(),
                    &target,
                    patched.schema(),
                    5,
                    Some(("Original target", "Patched")),
                )
                .to_string(),
            )
            .into());
        }
    }

    Ok(())
}

datatest_stable::harness! {
    { test = run_test, root = "./tests/patch", pattern = r"^.*\.graphql$" },
    { test = run_test_backwards, root = "./tests/patch", pattern = r"^.*\.graphql$" },
}
