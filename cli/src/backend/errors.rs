use crate::common::errors::CommonError;
use notify_debouncer_full::notify;
use std::{fmt, path::PathBuf};
use thiserror::Error;

use super::api::errors::ApiError;

#[derive(Error, Debug)]
pub(crate) enum BackendError {
    // wraps a [`CommonError`]
    #[error(transparent)]
    CommonError(#[from] CommonError),
    #[error("could not read the configuration file\nCaused by: {0}")]
    ReadConfig(std::io::Error),
    #[error("could not read the graph overrides\nCaused by: {0}")]
    ReadGraphOverrides(std::io::Error),
    #[error("could not parse the configuration file\nCaused by: {0}")]
    ParseConfig(toml::de::Error),
    #[error("could not parse the graph overrides configuration\nCaused by: {0}")]
    ParseGraphOverrides(toml::de::Error),
    #[error("could not merge the gateway and graph override configurations")]
    MergeConfigurations,
    #[error(transparent)]
    ApiError(#[from] ApiError),
    #[error("could not read the SDL from {0}\nCaused by: {1}")]
    ReadSdlFromFile(PathBuf, std::io::Error),
    #[error("could not set the current directory\nCaused by: {0}")]
    SetCurrentDirectory(std::io::Error),
    #[error("could not introspect a subgraph URL: {0}")]
    IntrospectSubgraph(String),
    #[error("no url or schema_path were defined for an overridden subgraph: {0}")]
    NoDefinedRouteToSubgraphSdl(String),
    #[error("could not parse a subgraph:\n{0:#}")]
    ParseSubgraphSdl(cynic_parser::Error),
    #[error("could not start the federated gateway\nCaused by: {0}")]
    Serve(federated_server::Error),
    #[error("could not compose subgraphs\nCaused by: {0}")]
    Composition(String),
    #[error("could not convert the composed subgraphs to federated SDL\nCaused by: {0}")]
    ToFederatedSdl(fmt::Error),
    #[error("could not fetch the specified branch")]
    FetchBranch,
    #[error("the specified branch does not exist")]
    BranchDoesntExist,
    #[error("the gateway configuration contains a field reserved for the graph overrides configuration: {0}")]
    DevOptionsInGatewayConfig(&'static str),
    #[error("could not set up a file watcher\nCaused by: {0}")]
    SetUpWatcher(notify::Error),
    #[error("could not determine the path of the home directory")]
    HomeDirectory,
    #[error("could not unpack Pathfinder\nCaused by: {0}")]
    UnpackPathfinderArchive(std::io::Error),
    #[error("could not write the current version of the unpacked Pathfinder assets\nCaused by: {0}")]
    WriteAssetVersion(std::io::Error),
    #[error("could not read the current version of the unpacked Pathfinder assets\nCaused by: {0}")]
    ReadAssetVersion(std::io::Error),
    #[error("could not create ~/.grafbase\nCaused by: {0}")]
    CreateDotGrafbaseDirectory(std::io::Error),
    #[error("could not access ~/.grafbase\nCaused by: {0}")]
    AccessDotGrafbaseDirectory(std::io::Error),
}

impl From<cynic_parser::Error> for BackendError {
    fn from(v: cynic_parser::Error) -> Self {
        Self::ParseSubgraphSdl(v)
    }
}
