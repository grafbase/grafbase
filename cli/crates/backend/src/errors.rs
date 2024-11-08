use common::errors::CommonError;
use federated_graph::DomainError;
use graphql_composition::IngestError;
use std::{io, path::PathBuf};
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

    /// returned if a schema.graphql file could not be created
    #[error("could not create a schema.graphql file\nCaused by: {0}")]
    WriteSchema(io::Error),

    /// returned if a .env file could not be created
    #[error("could not create a .env file\nCaused by: {0}")]
    WriteEnvFile(io::Error),

    /// returned if .grafbase/database could not be deleted
    #[error("could not delete '.grafbase/database'\nCaused by: {0}")]
    DeleteDatabaseDirectory(io::Error),

    /// returned if the grafbase directory for the project could not be deleted
    #[error("could not delete the grafbase directory\nCaused by: {0}")]
    DeleteGrafbaseDirectory(io::Error),

    /// returned if a template URL is not supported
    #[error("'{0}' is not a supported template URL")]
    UnsupportedTemplateURL(String),

    /// returned if a template URL could not be parsed
    #[error("'{0}' is not a valid URL")]
    MalformedTemplateURL(String),

    /// returned if a repo tar could not be downloaded (on a non 200-299 status)
    #[error("could not download the archive for '{0}'\nCaused by: {1}")]
    StartDownloadRepoArchive(String, reqwest_middleware::Error),

    /// returned if a repo tar could not be downloaded
    #[error("could not download the archive for '{0}'")]
    DownloadRepoArchive(String),

    // since this is checked by looking for the extracted files on disk (extraction errors are checked beforehand),
    // may have unlikely false positives if the files were deleted or moved by an external process immediately after extraction.
    //
    // TODO: consider adding an indicator that a file was extracted rather than checking on disk
    // TODO: consider splitting this into internal and external template errors for clarity
    // and change this error to something indicating that the extracted files were not found
    /// returned if no files matching the template path were extracted (excluding extraction errors)
    #[error("could not find the provided template within the template repository")]
    TemplateNotFound,

    /// returned if the extracted files from the template repository could not be moved
    #[error("could not move the extracted files from the template repository\nCaused by: {0}")]
    CopyTemplateFiles(io::Error),

    /// returned if the entries of the template repository archive could not be read
    #[error("could not read the entries of the template repository archive")]
    ReadArchiveEntries,

    /// returned if one of the entries of the template repository archive could not be extracted
    #[error("could not extract an entry from the template repository archive\nCaused by: {0}")]
    ExtractArchiveEntry(io::Error),

    /// returned if the files extracted from the template repository archive could not be cleaned
    #[error("could not clean the files extracted from the repository archive\nCaused by: {0}")]
    CleanExtractedFiles(io::Error),

    /// returned if the request to get the information for a repository could not be sent
    #[error("could not get the repository information for {0}")]
    StartGetRepositoryInformation(String),

    /// returned if the request to get the information for a repository returned a non 200-299 status
    #[error("could not get the repository information for {0}")]
    GetRepositoryInformation(String),

    /// returned if the request to get the information for a repository returned a response that could not be parsed
    #[error("could not read the repository information for {0}")]
    ReadRepositoryInformation(String),

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
    ParseGatewayConfig(String),
    #[error("could not parse the graph overrides\nCaused by: {0}")]
    ParseGraphOverrides(toml::de::Error),
    #[error("could not merge the gateway and graph override configurations")]
    MergeConfigurations,
    #[error(transparent)]
    ApiError(#[from] ApiError),
    #[error("could not read the SDL from {0}\nCaused by: {1}")]
    ReadSdlFromFile(PathBuf, std::io::Error),
    #[error("could not introspect a subgraph URL: {0}")]
    IntrospectSubgraph(String),
    #[error("no url or file were defined for an overridden subgraph: {0}")]
    NoDefinedRouteToSubgraphSdl(String),
    #[error("could not parse a subgraph\nCaused by: {0}")]
    IngestSubgraph(IngestError),
    #[error("could not start the federated gateway\nCaused by: {0}")]
    Serve(federated_server::Error),
    #[error("could not compose subgraphs\nCaused by: {0}")]
    Composition(String),
    #[error("could not convert the composed subgraphs to federated SDL\nCaused by: {0}")]
    ToFederatedSdl(DomainError),
    #[error("could not fetch the specified branch")]
    FetchBranch,
    #[error("the specified branch does not exist")]
    BranchDoesntExist,
}
