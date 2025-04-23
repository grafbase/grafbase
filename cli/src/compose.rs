use crate::{cli_input::ComposeCommand, dev::SubgraphCache, output::report};

#[tokio::main]
pub(crate) async fn compose(args: ComposeCommand) -> anyhow::Result<()> {
    let config = args.config()?;

    if args.graph_ref.is_none() && config.subgraphs.is_empty() {
        return Err(anyhow::anyhow!("No subgraphs found"));
    }

    let subgraph_cache = SubgraphCache::new(args.graph_ref.as_ref(), &config).await?;

    let composed = subgraph_cache.compose().await?.into_result();

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
