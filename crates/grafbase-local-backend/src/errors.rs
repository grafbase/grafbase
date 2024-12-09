use common::errors::CommonError;
use graphql_composition::IngestError;
use std::{fmt, io, path::PathBuf};
use thiserror::Error;

use crate::api::errors::ApiError;

#[derive(Error, Debug)]
pub enum BackendError {
    /// returned when trying to initialize a project that conflicts with an existing project
    #[error("{0} already exists")]
    AlreadyAProject(PathBuf),

    /// returned when trying to initialize a project that conflicts with an existing directory or file
    #[error("{0} already exists")]
    ProjectDirectoryExists(PathBuf),

    /// returned if the current directory path could not be read
    #[error("could not read the current path")]
    ReadCurrentDirectory,

    /// returned if the grafbase directory could not be created
    #[error("could not create the 'grafbase' directory\nCaused by: {0}")]
    CreateGrafbaseDirectory(io::Error),

    /// returned if the project directory could not be created
    #[error("could not create the project directory\nCaused by: {0}")]
    CreateProjectDirectory(io::Error),

    /// returned if a .env file could not be created
    #[error("could not create a .env file\nCaused by: {0}")]
    WriteEnvFile(io::Error),

    /// returned if creating a temporary directory for the template archive fails
    #[error("could not create a temporary directory to download the template archive: {0}")]
    CouldNotCreateTemporaryFile(std::io::Error),

    // wraps a [`CommonError`]
    #[error(transparent)]
    CommonError(#[from] CommonError),
    #[error("could not read the gateway configuration\nCaused by: {0}")]
    ReadGatewayConfig(std::io::Error),
    #[error("could not read the graph overrides\nCaused by: {0}")]
    ReadGraphOverrides(std::io::Error),
    #[error("could not parse the gateway configuration\nCaused by: {0}")]
    ParseGatewayConfig(toml::de::Error),
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
    IngestSubgraph(IngestError),
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
