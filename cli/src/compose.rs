use cynic_parser::TypeSystemDocument;
use futures::{TryStreamExt as _, stream::FuturesUnordered};

use crate::{
    backend::dev::{detect_extensions, fetch_remote_subgraphs},
    cli_input::ComposeCommand,
    output::report,
};

#[tokio::main]
pub(crate) async fn compose(args: ComposeCommand) -> anyhow::Result<()> {
    let config = args.config()?;

    if args.graph_ref.is_none() && config.subgraphs.is_empty() {
        return Err(anyhow::anyhow!("No subgraphs found"));
    }

    let remote_subgraphs = if let Some(graph_ref) = &args.graph_ref {
        fetch_remote_subgraphs(graph_ref).await?
    } else {
        Vec::new()
    };

    struct IngestionTask {
        name: String,
        url: Option<String>,
        doc: TypeSystemDocument,
        extensions: Vec<graphql_composition::LoadedExtension>,
    }

    let remotes_fut = remote_subgraphs
        .into_iter()
        .map(|subgraph| async move {
            let doc = cynic_parser::parse_type_system_document(&subgraph.schema)
                .map_err(|err| anyhow::anyhow!("Failed to parse schema for subgraph {}: {}", subgraph.name, err))?;
            let extensions = detect_extensions(None, &doc)
                .await
                .into_iter()
                .map(|ext| graphql_composition::LoadedExtension::new(ext.url, ext.name))
                .collect();

            anyhow::Ok(IngestionTask {
                name: subgraph.name.clone(),
                url: subgraph.url,
                doc,
                extensions,
            })
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>();

    let current_dir = std::env::current_dir().ok();
    let current_dir = current_dir.as_deref();
    let locals_fut = config
        .subgraphs
        .into_iter()
        .map(|(name, subgraph)| async move {
            if let Some(schema_path) = subgraph.schema_path {
                let schema = std::fs::read_to_string(&schema_path).map_err(|err| {
                    anyhow::anyhow!("Failed to read schema file at {}: {}", schema_path.display(), err)
                })?;
                let doc = cynic_parser::parse_type_system_document(&schema)
                    .map_err(|err| anyhow::anyhow!("Failed to parse schema for subgraph {}: {}", name, err))?;

                anyhow::Ok(Some(IngestionTask {
                    name: name.clone(),
                    url: subgraph.url.as_ref().map(|url| url.to_string()),
                    extensions: detect_extensions(current_dir, &doc)
                        .await
                        .into_iter()
                        .map(|ext| graphql_composition::LoadedExtension::new(ext.url, ext.name))
                        .collect(),
                    doc,
                }))
            } else if let Some(url) = subgraph.introspection_url.or(subgraph.url.clone()) {
                let headers: Vec<(&String, _)> = subgraph
                    .introspection_headers
                    .as_ref()
                    .map(|intropection_headers| intropection_headers.iter().collect())
                    .unwrap_or_default();

                let schema = grafbase_graphql_introspection::introspect(url.as_str(), &headers)
                    .await
                    .map_err(|err| anyhow::anyhow!("Failed to introspect subgraph {name}: {err}"))?;
                let doc = cynic_parser::parse_type_system_document(&schema)
                    .map_err(|err| anyhow::anyhow!("Failed to parse schema for subgraph {}: {}", name, err))?;

                Ok(Some(IngestionTask {
                    name: name.clone(),
                    url: subgraph.url.as_ref().map(|url| url.to_string()),
                    extensions: detect_extensions(None, &doc)
                        .await
                        .into_iter()
                        .map(|ext| graphql_composition::LoadedExtension::new(ext.url, ext.name))
                        .collect(),
                    doc,
                }))
            } else {
                Ok(None)
            }
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>();

    let (remotes, locals) = tokio::try_join!(remotes_fut, locals_fut)?;

    let mut subgraphs = graphql_composition::Subgraphs::default();
    for subgraph in remotes.into_iter().map(Some).chain(locals) {
        let Some(IngestionTask {
            name,
            url,
            doc,
            extensions,
        }) = subgraph
        else {
            continue;
        };
        subgraphs.ingest(&doc, &name, url.as_deref());
        subgraphs.ingest_loaded_extensions(extensions);
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
