use std::{fs, path::Path, sync::OnceLock};

fn update_expect() -> bool {
    static UPDATE_EXPECT: OnceLock<bool> = OnceLock::new();
    *UPDATE_EXPECT.get_or_init(|| std::env::var("UPDATE_EXPECT").is_ok())
}

#[allow(clippy::unnecessary_wraps)] // we can't change the signature expected by datatest_stable
fn run_test(federated_graph_path: &Path) -> datatest_stable::Result<()> {
    let subgraphs_dir = federated_graph_path.with_file_name("").join("subgraphs");

    if !subgraphs_dir.is_dir() {
        return Err(miette::miette!("{} is not a directory.", subgraphs_dir.display()).into());
    }

    let mut subgraphs_sdl = fs::read_dir(subgraphs_dir)?
        .filter_map(Result::ok)
        .map(|file| fs::read_to_string(file.path()).map(|contents| (contents, file.path())))
        .collect::<Result<Vec<_>, _>>()?;

    // [fs::read_dir()] doesn't guarantee ordering. Sort to make tests deterministic
    // (inconsistencies observed in CI).
    subgraphs_sdl.sort_by_key(|(_, path)| path.file_name().unwrap().to_owned());

    let mut subgraphs = grafbase_composition::Subgraphs::default();

    for (sdl, path) in subgraphs_sdl {
        let parsed = async_graphql_parser::parse_schema(&sdl)
            .map_err(|err| miette::miette!("Error parsing {}: {err}", path.display()))?;
        subgraphs.ingest(&parsed, path.file_stem().unwrap().to_str().unwrap());
    }

    let expected = fs::read_to_string(federated_graph_path)
        .map_err(|err| miette::miette!("Error trying to read federated.graphql: {}", err))?;
    let actual = match grafbase_composition::compose(&subgraphs).into_result() {
        Ok(sdl) => grafbase_federated_graph::render_sdl(&sdl).unwrap(),
        Err(diagnostics) => format!(
            "{}\n",
            diagnostics
                .iter_messages()
                .map(|msg| format!("# {msg}"))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
    };

    if expected == actual {
        return Ok(());
    }

    if update_expect() {
        return fs::write(federated_graph_path, actual).map_err(From::from);
    }

    Err(miette::miette!(
        "{}\n\n\n=== Hint: run the tests again with UPDATE_EXPECT=1 to update the snapshot. ===",
        similar::udiff::unified_diff(
            similar::Algorithm::default(),
            &expected,
            &actual,
            5,
            Some(("Expected", "Actual"))
        )
    )
    .into())
}

datatest_stable::harness! { run_test, "./tests/composition", r"^.*federated.graphql$" }
