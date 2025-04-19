use std::collections::HashMap;

use gateway_config::Config;

use crate::{
    backend::dev::{detect_extensions, fetch_remote_subgraphs},
    cli_input::ComposeCommand,
    output::report,
};

#[tokio::main]
pub(crate) async fn compose(ComposeCommand { graph_ref, config_path }: ComposeCommand) -> anyhow::Result<()> {
    let config = Config::load(&config_path).map_err(|err| anyhow::anyhow!(err))?;

    if graph_ref.is_none() && config.subgraphs.is_empty() {
        return Err(anyhow::anyhow!("No subgraphs found"));
    }

    let remote_subgraphs = if let Some(graph_ref) = &graph_ref {
        fetch_remote_subgraphs(graph_ref).await?
    } else {
        Vec::new()
    };

    let mut subgraph_schemas: HashMap<String, (String, Option<String>)> =
        HashMap::with_capacity(remote_subgraphs.len().max(config.subgraphs.len()));

    for subgraph in remote_subgraphs {
        subgraph_schemas.insert(subgraph.name, (subgraph.schema, subgraph.url));
    }

    for (subgraph_name, subgraph) in config.subgraphs {
        let subgraph_url = subgraph.introspection_url.or(subgraph.url);

        if let Some(schema_path) = subgraph.schema_path {
            let schema = std::fs::read_to_string(&schema_path)
                .map_err(|err| anyhow::anyhow!("Failed to read schema file at {}: {}", schema_path.display(), err))?;
            subgraph_schemas.insert(subgraph_name, (schema, subgraph_url.map(|url| url.to_string())));
        } else if let Some(url) = subgraph_url {
            let headers: Vec<(&String, _)> = subgraph
                .introspection_headers
                .as_ref()
                .map(|intropection_headers| intropection_headers.iter().collect())
                .unwrap_or_default();

            // FIXME: do it concurrently
            let sdl = grafbase_graphql_introspection::introspect(url.as_str(), &headers)
                .await
                .map_err(|err| anyhow::anyhow!("Failed to introspect subgraph {subgraph_name}: {err}"))?;

            subgraph_schemas.insert(subgraph_name, (sdl, Some(url.to_string())));
        }
    }

    let mut subgraphs = graphql_composition::Subgraphs::default();

    for (subgraph_name, (schema, url)) in subgraph_schemas {
        let parsed_schema = cynic_parser::parse_type_system_document(&schema)
            .map_err(|err| anyhow::anyhow!("Failed to parse schema for subgraph {subgraph_name}: {err}"))?;

        subgraphs.ingest(&parsed_schema, &subgraph_name, url.as_deref());

        // FIXME: do it concurrently
        let detected_extensions = detect_extensions(&parsed_schema).await;

        subgraphs.ingest_loaded_extensions(
            detected_extensions
                .into_iter()
                .map(|ext| graphql_composition::LoadedExtension::new(ext.url, ext.name)),
        );
    }

    let composed = graphql_composition::compose(&subgraphs).into_result();

    match composed {
        Ok(schema) => {
            let rendered = graphql_composition::render_federated_sdl(&schema).expect("rendering to succeed");

            println!("{rendered}");

            Ok(())
        }
        Err(diagnostics) => {
            report::composition_diagnostics(&diagnostics);
            std::process::exit(1)
        }
    }
}
