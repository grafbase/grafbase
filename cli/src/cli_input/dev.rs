use super::FullGraphRef;
use clap::Parser;
use std::path::PathBuf;

/// Start a local development server
#[derive(Debug, Parser)]
pub struct DevCommand {
    #[arg(short('g'), long("graph-ref"), help = FullGraphRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: Option<FullGraphRef>,
    /// The path of the configuration file
    #[arg(short('c'), long("config"), default_value("grafbase.toml"))]
    pub(crate) config: Option<PathBuf>,
    /// The path of the graph overrides configuration file
    #[arg(short('o'), long("graph-overrides"))]
    pub(crate) graph_overrides: Option<PathBuf>,
    /// The port to listen on
    #[arg(short('p'), long("port"))]
    pub(crate) port: Option<u16>,
}
