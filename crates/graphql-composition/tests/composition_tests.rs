use std::{fs, path::Path};

use graphql_composition::FederatedGraph;

fn run_test(test_path: &Path) -> anyhow::Result<()> {
    if cfg!(windows) {
        return Ok(()); // newlines
    }

    let test_description = fs::read_to_string(test_path)?;
    let subgraphs_dir = test_path.with_file_name("").join("subgraphs");
    let extensions_path = test_path.with_file_name("extensions.toml");

    if !subgraphs_dir.is_dir() {
        return Err(anyhow::anyhow!("{} is not a directory.", subgraphs_dir.display()));
    }

    let extensions: TestExtensions = fs::read_to_string(extensions_path)
        .ok()
        .map(|file| toml::from_str(&file).unwrap())
        .unwrap_or_default();

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
        let name = path.file_stem().unwrap().to_str().unwrap().replace('_', "-");

        subgraphs
            .ingest_str(&sdl, &name, Some(&format!("http://example.com/{name}")))
            .map_err(|err| anyhow::anyhow!("Error parsing {}: \n{err:#}", path.display()))?;
    }

    subgraphs.ingest_loaded_extensions(
        extensions
            .extensions
            .into_iter()
            .map(|extension| graphql_composition::LoadedExtension::new(extension.url, extension.name)),
    );

    let (federated_sdl, api_sdl) = match graphql_composition::compose(&subgraphs).into_result() {
        Ok(federated_graph) => (
            graphql_federated_graph::render_federated_sdl(&federated_graph).expect("rendering federated SDL"),
            Some(graphql_federated_graph::render_api_sdl(&federated_graph)),
        ),
        Err(diagnostics) => (
            format!(
                "{}\n",
                diagnostics
                    .iter_messages()
                    .map(|msg| format!("# {}", msg.lines().collect::<Vec<_>>().join("\\n")))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            None,
        ),
    };

    let test_description = Some(test_description.as_str().trim())
        .filter(|desc| !desc.is_empty())
        .unwrap_or("Federated SDL");

    insta::assert_snapshot!("federated.graphql", federated_sdl, test_description);

    if let Some(actual_api_sdl) = api_sdl {
        insta::assert_snapshot!("api.graphql", actual_api_sdl);
    }

    test_sdl_roundtrip(&federated_sdl)
}

fn test_sdl_roundtrip(sdl: &str) -> anyhow::Result<()> {
    // Exclude tests with an empty schema. This is the case for composition error tests.
    if sdl.lines().all(|line| line.is_empty() || line.starts_with('#')) {
        return Ok(());
    }

    let roundtripped = graphql_federated_graph::render_federated_sdl(
        &FederatedGraph::from_sdl(sdl).map_err(|err| anyhow::anyhow!("Error ingesting SDL: {err}\n\nSDL:\n{sdl}"))?,
    )?;

    if roundtripped == sdl {
        return Ok(());
    }

    pretty_assertions::assert_eq!(sdl, roundtripped, "SDL roundtrip failed");
    Ok(())
}

#[test]
fn composition_tests() {
    insta::glob!("composition/**/test.md", |test_path| {
        let snapshot_path = test_path.parent().unwrap();
        insta::with_settings!({
            snapshot_path => snapshot_path.to_str().unwrap(),
            prepend_module_to_snapshot => false,
            snapshot_suffix => "",
        }, {
            run_test(test_path).unwrap();
        });
    });
}

#[derive(Debug, serde::Deserialize, Default)]
struct TestExtensions {
    extensions: Vec<TestExtension>,
}

#[derive(Debug, serde::Deserialize)]
struct TestExtension {
    url: String,
    name: String,
}
