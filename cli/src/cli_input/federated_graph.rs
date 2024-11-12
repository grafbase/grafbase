use clap::Parser;

#[derive(Debug, Parser)]
pub enum FederatedGraphCommandKind {
    /// Fetch the GraphQL schema of the federated graph
    Fetch,
}

#[derive(Debug, Parser)]
struct FederatedGraphCommand {
    #[command(subcommand)]
    kind: FederatedGraphCommandKind,
}
