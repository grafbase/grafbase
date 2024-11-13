use super::FullGraphRef;
use clap::{Args, Parser};
use std::path::PathBuf;

/// Start a local development server
#[derive(Debug, Parser)]
pub struct DevCommand {
    #[command(flatten)]
    pub(crate) sources: Sources,
    /// The port to listen on for requests
    #[arg(short('p'), long("port"))]
    pub(crate) port: Option<u16>,
}

#[derive(Args, Debug)]
#[group(required = true)]
pub struct Sources {
    #[arg(short('r'), long("graph-ref"), help = FullGraphRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: Option<FullGraphRef>,
    /// The path of the gateway configuration file
    #[arg(short('c'), long("gateway-config"))]
    pub(crate) gateway_config: Option<PathBuf>,
    /// The path of the graph overrides configuration file
    #[arg(short('o'), long("graph-overrides"))]
    pub(crate) graph_overrides: Option<PathBuf>,
}
