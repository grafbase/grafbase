use std::{fmt::Write as _, fs, path::Path};

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

    // Important: we want to load extensions _after_ ingesting subgraphs, to make sure the subgraph ingestion does not depend on the extensions having been populated.
    subgraphs.ingest_loaded_extensions(extensions.extensions.into_iter().map(|extension| {
        graphql_composition::LoadedExtension {
            url: extension.url.parse().unwrap(),
            link_url: extension.url,
            name: extension.name,
        }
    }));

    let result = graphql_composition::compose(&mut subgraphs);

    let diagnostics = result.diagnostics();
    let mut rendered_diagnostics = String::new();

    for diagnostic in diagnostics.iter() {
        let emoji = match diagnostic.severity() {
            graphql_composition::diagnostics::Severity::Error => "❌",
            graphql_composition::diagnostics::Severity::Warning => "⚠️",
        };

        write!(rendered_diagnostics, "- {emoji} ").unwrap();

        if let Some(error_code) = diagnostic.composite_schemas_error_code() {
            write!(rendered_diagnostics, "{{ {:?} }} ", error_code).unwrap();
        }

        rendered_diagnostics.push_str(diagnostic.message());
        rendered_diagnostics.push('\n');
    }

    let (federated_sdl, api_sdl) = if let Ok(federated_graph) = result.into_result() {
        (
            graphql_composition::render_federated_sdl(&federated_graph).expect("rendering federated SDL"),
            graphql_composition::render_api_sdl(&federated_graph),
        )
    } else {
        (String::new(), String::new())
    };

    let test_description = Some(test_description.as_str().trim()).filter(|desc| !desc.is_empty());

    insta::assert_snapshot!(
        "diagnostics",
        rendered_diagnostics,
        test_description.unwrap_or("Diagnostics")
    );
    insta::assert_snapshot!(
        "federated.graphql",
        federated_sdl,
        test_description.unwrap_or("Federated SDL")
    );
    insta::assert_snapshot!("api.graphql", api_sdl, test_description.unwrap_or("API SDL"));

    check_federated_sdl(&federated_sdl, test_path)
}

#[allow(clippy::panic)]
fn check_federated_sdl(federated_sdl: &str, test_path: &Path) -> anyhow::Result<()> {
    // Exclude tests with an empty schema. This is the case for composition error tests.
    if federated_sdl
        .lines()
        .all(|line| line.is_empty() || line.starts_with('#'))
    {
        return Ok(());
    }

    {
        let diagnostics = graphql_schema_validation::validate(federated_sdl);

        #[allow(clippy::panic)]
        if diagnostics.has_errors() {
            panic!(
                "Validation errors on federated SDL for {}.\n{}",
                test_path.display(),
                diagnostics
                    .iter()
                    .map(|msg| msg.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        }
    }

    let rendered = FederatedGraph::from_sdl(federated_sdl)
        .map_err(|err| anyhow::anyhow!("Error ingesting SDL: {err}\n\nSDL:\n{federated_sdl}"))?;
    let roundtripped = graphql_composition::render_federated_sdl(&rendered)?;

    pretty_assertions::assert_eq!(
        federated_sdl,
        roundtripped,
        "Federated SDL roundtrip failed for {}",
        test_path.display()
    );

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
