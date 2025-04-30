use crate::{cli_input::ComposeCommand, dev::SubgraphCache, output::report};

#[tokio::main]
pub(crate) async fn compose(args: ComposeCommand) -> anyhow::Result<()> {
    let config = args.config()?;

    if args.graph_ref.is_none() && config.subgraphs.is_empty() {
        return Err(anyhow::anyhow!("No subgraphs found"));
    }

    let (warnings_sender, _warnings_receiver) = tokio::sync::mpsc::channel(1);

    let subgraph_cache = SubgraphCache::new(args.graph_ref.as_ref(), &config, warnings_sender).await?;

    let result = subgraph_cache.compose(&config).await?;

    match result {
        Ok(schema) => {
            println!("{schema}");

            Ok(())
        }
        Err(diagnostics) => {
            report::composition_diagnostics(&diagnostics);
            std::process::exit(1)
        }
    }
}
