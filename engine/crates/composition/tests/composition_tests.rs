use async_graphql_value as _;
use indexmap as _;
use itertools as _;
use std::{fs, path::Path, sync::OnceLock};

fn update_expect() -> bool {
    static UPDATE_EXPECT: OnceLock<bool> = OnceLock::new();
    *UPDATE_EXPECT.get_or_init(|| std::env::var("UPDATE_EXPECT").is_ok())
}

fn run_test(federated_graph_path: &Path) -> datatest_stable::Result<()> {
    if cfg!(windows) {
        return Ok(()); // newlines
    }

    let subgraphs_dir = federated_graph_path.with_file_name("").join("subgraphs");
    let api_sdl_path = federated_graph_path.with_file_name("api.graphql");

    if !subgraphs_dir.is_dir() {
        return Err(miette::miette!("{} is not a directory.", subgraphs_dir.display()).into());
    }

    let mut subgraphs_sdl = fs::read_dir(subgraphs_dir)?
        .filter_map(Result::ok)
        .filter(|file| file.file_name() != ".gitignore")
        .map(|file| fs::read_to_string(file.path()).map(|contents| (contents, file.path())))
        .collect::<Result<Vec<_>, _>>()?;

    // [fs::read_dir()] doesn't guarantee ordering. Sort to make tests deterministic
    // (inconsistencies observed in CI).
    subgraphs_sdl.sort_by_key(|(_, path)| path.file_name().unwrap().to_owned());

    let mut subgraphs = graphql_composition::Subgraphs::default();

    for (sdl, path) in subgraphs_sdl {
        let parsed = async_graphql_parser::parse_schema(&sdl)
            .map_err(|err| miette::miette!("Error parsing {}: {err}", path.display()))?;

        let name = path.file_stem().unwrap().to_str().unwrap();

        subgraphs.ingest(&parsed, name, &format!("http://example.com/{name}"));
    }

    let expected_federated_sdl = fs::read_to_string(federated_graph_path)
        .map_err(|err| miette::miette!("Error trying to read federated.graphql: {}", err))?;
    let expected_api_sdl = fs::read_to_string(&api_sdl_path)
        .map_err(|err| miette::miette!("Error trying to read api.graphql: {}", err))
        .ok();
    let (actual_federated_sdl, actual_api_sdl) = match graphql_composition::compose(&subgraphs).into_result() {
        Ok(federated_graph) => {
            let federated_graph = federated_graph.into_latest();
            (
                graphql_federated_graph::render_federated_sdl(&federated_graph).unwrap(),
                Some(graphql_federated_graph::render_api_sdl(&federated_graph).unwrap()),
            )
        }
        Err(diagnostics) => (
            format!(
                "{}\n",
                diagnostics
                    .iter_messages()
                    .map(|msg| format!("# {msg}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            None,
        ),
    };

    if expected_federated_sdl == actual_federated_sdl && expected_api_sdl == actual_api_sdl {
        return Ok(());
    }

    if update_expect() {
        if let Some(sdl) = expected_api_sdl {
            fs::write(api_sdl_path, sdl)?;
        }

        return fs::write(federated_graph_path, actual_federated_sdl).map_err(From::from);
    }

    Err(miette::miette!(
        "{}\n\n\n=== Hint: run the tests again with UPDATE_EXPECT=1 to update the snapshot. ===",
        similar::udiff::unified_diff(
            similar::Algorithm::default(),
            &expected_federated_sdl,
            &actual_federated_sdl,
            5,
            Some(("Expected", "Actual"))
        )
    )
    .into())
}

fn test_sdl_roundtrip(federated_graph_path: &Path) -> datatest_stable::Result<()> {
    if cfg!(windows) {
        return Ok(()); // newlines
    }

    let sdl = fs::read_to_string(federated_graph_path)
        .map_err(|err| miette::miette!("Error trying to read federated.graphql: {}", err))?;

    // Exclude tests with an empty schema. This is the case for composition error tests.
    if sdl.lines().all(|line| line.is_empty() || line.starts_with('#')) {
        return Ok(());
    }

    let roundtripped = graphql_federated_graph::render_federated_sdl(
        &graphql_federated_graph::from_sdl(&sdl)
            .map_err(|err| miette::miette!("Error ingesting SDL: {err}\n\nSDL:\n{sdl}"))?
            .into_latest(),
    )?;

    if roundtripped == sdl {
        return Ok(());
    }

    Err(miette::miette!(
        "{}\n\n\n=== Hint: run the tests again with UPDATE_EXPECT=1 to update the snapshot. ===",
        similar::udiff::unified_diff(
            similar::Algorithm::default(),
            &sdl,
            &roundtripped,
            5,
            Some(("Expected", "Actual"))
        )
    )
    .into())
}

datatest_stable::harness! {
    run_test, "./tests/composition", r"^.*federated.graphql$",
    test_sdl_roundtrip, "./tests/composition", r"^.*federated.graphql$",
}
