use crate::consts::DEFAULT_SCHEMA;
use crate::errors::BackendError;
use async_compression::tokio::bufread::GzipDecoder;
use async_tar::Archive;
use common::consts::{GRAFBASE_DIRECTORY, GRAFBASE_SCHEMA};
use common::environment::Environment;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::iter::Iterator;
use std::path::PathBuf;
use tokio_stream::StreamExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tokio_util::io::StreamReader;
use url::Url;

/// initializes a new project in the current or a new directory, optionally from a template
///
/// # Errors
///
/// ## General
///
/// - returns [`BackendError::ReadCurrentDirectory`] if the current directory cannot be read
///
/// - returns [`BackendError::ProjectDirectoryExists`] if a named is passed and a directory with the same name already exists in the current directory
///
/// - returns [`BackendError::AlreadyAProject`] if there's already a grafbase/schema.graphql in the target
///
/// - returns [`BackendError::CreateGrafbaseDirectory`] if the grafbase directory cannot be created
///
/// - returns [`BackendError::WriteSchema`] if the schema file cannot be written
///
/// ## Templates
///
/// - returns [`BackendError::UnsupportedTemplateURL`] if a template URL is not supported
///
/// - returns [`BackendError::StartDownloadRepoArchive`] if a template URL is not supported (if the request could not be made)
///
/// - returns [`BackendError::DownloadRepoArchive`] if a repo tar could not be downloaded (on a non 200-299 status)
///
/// - returns [`BackendError::TemplateNotFound`] if no files matching the template path were extracted (excluding extraction errors)
///
/// - returns [`BackendError::MoveExtractedFiles`] if the extracted files from the template repository could not be moved
///
/// - returns [`BackendError::ReadArchiveEntries`] if the entries of the template repository archive could not be read
///
/// - returns [`BackendError::ExtractArchiveEntry`] if one of the entries of the template repository archive could not be extracted
///
/// - returns [`BackendError::CleanExtractedFiles`] if the files extracted from the template repository archive could not be cleaned
#[tokio::main]
pub async fn init(name: Option<&str>, template: Option<&str>) -> Result<(), BackendError> {
    let project_path = to_project_path(name)?;
    let grafbase_path = project_path.join(GRAFBASE_DIRECTORY);
    let schema_path = grafbase_path.join(GRAFBASE_SCHEMA);

    if grafbase_path.exists() {
        Err(BackendError::AlreadyAProject(grafbase_path))
    } else if let Some(template) = template {
        // as directory names cannot contain slashes, and URLs with no scheme or path cannot
        // be differentiated from a valid template name,
        // anything with a slash is treated as a URL
        if template.contains('/') {
            if let Ok(repo_url) = Url::parse(template) {
                match repo_url.host_str() {
                    Some("github.com") => handle_github_repo_url(&repo_url).await,
                    _ => Err(BackendError::UnsupportedTemplateURL(template.to_string())),
                }
            } else {
                return Err(BackendError::MalformedTemplateURL(template.to_owned()));
            }
        } else {
            download_github_template(GitHubTemplate::Grafbase(GrafbaseGithubTemplate { path: template })).await
        }
    } else {
        tokio::fs::create_dir_all(&grafbase_path)
            .await
            .map_err(BackendError::CreateGrafbaseDirectory)?;
        let write_result = fs::write(schema_path, DEFAULT_SCHEMA).map_err(BackendError::WriteSchema);

        if write_result.is_err() {
            tokio::fs::remove_dir_all(&grafbase_path)
                .await
                .map_err(BackendError::DeleteGrafbaseDirectory)?;
        }

        write_result
    }
}

async fn handle_github_repo_url(repo_url: &Url) -> Result<(), BackendError> {
    if let Some(mut segments) = repo_url.path_segments().map(Iterator::collect::<Vec<_>>) {
        // remove trailing slashes to prevent extra path parameters
        if segments.last() == Some(&"") {
            segments.pop();
        }

        // disallow empty path paramters other than the last
        if segments.contains(&"") {
            return Err(BackendError::UnsupportedTemplateURL(repo_url.to_string()));
        }

        match segments.len() {
            2 => {
                let org = &segments[0];

                let repo = &segments[1];

                let branch = get_default_branch(org, repo).await?;

                download_github_template(GitHubTemplate::External(ExternalGitHubTemplate {
                    org,
                    repo,
                    branch: &branch,
                    path: None,
                }))
                .await
            }
            4.. if segments[2] == "tree" => {
                let org = &segments[0];

                let repo = &segments[1];

                let branch = &segments[3];

                let path = segments.get(4..).map(|path| path.join("/"));

                download_github_template(GitHubTemplate::External(ExternalGitHubTemplate {
                    org,
                    repo,
                    path,
                    branch,
                }))
                .await
            }
            _ => Err(BackendError::UnsupportedTemplateURL(repo_url.to_string())),
        }
    } else {
        Err(BackendError::UnsupportedTemplateURL(repo_url.to_string()))
    }
}

#[derive(Deserialize)]
struct RepoInfo {
    default_branch: String,
}

async fn get_default_branch(org: &str, repo: &str) -> Result<String, BackendError> {
    let client = Client::new();

    let response = client
        .get(format!("https://api.github.com/repos/{org}/{repo}"))
        .header("User-Agent", "Grafbase-CLI")
        .send()
        .await
        .map_err(|_| BackendError::StartGetRepositoryInformation(format!("{org}/{repo}")))?;

    if !response.status().is_success() {
        return Err(BackendError::GetRepositoryInformation(format!("{org}/{repo}")));
    }

    let repo_info = response
        .json::<RepoInfo>()
        .await
        .map_err(|_| BackendError::ReadRepositoryInformation(format!("{org}/{repo}")))?;

    Ok(repo_info.default_branch)
}

fn to_project_path(name: Option<&str>) -> Result<PathBuf, BackendError> {
    let current_dir = env::current_dir().map_err(|_| BackendError::ReadCurrentDirectory)?;
    match name {
        Some(name) => {
            let project_path = current_dir.join(name);
            if project_path.exists() {
                Err(BackendError::ProjectDirectoryExists(project_path))
            } else {
                Ok(project_path)
            }
        }
        None => Ok(current_dir),
    }
}

#[derive(Clone)]
struct ExternalGitHubTemplate<'a> {
    org: &'a str,
    repo: &'a str,
    path: Option<String>,
    branch: &'a str,
}

struct GrafbaseGithubTemplate<'a> {
    path: &'a str,
}

enum GitHubTemplate<'a> {
    Grafbase(GrafbaseGithubTemplate<'a>),
    External(ExternalGitHubTemplate<'a>),
}

impl<'a> GitHubTemplate<'a> {
    pub fn into_external_github_template(self) -> ExternalGitHubTemplate<'a> {
        match self {
            Self::Grafbase(GrafbaseGithubTemplate { path }) => ExternalGitHubTemplate {
                org: "grafbase",
                repo: "grafbase",
                path: Some(format!("templates/{path}")),
                branch: "main",
            },
            Self::External(template @ ExternalGitHubTemplate { .. }) => template,
        }
    }
}

async fn download_github_template(template: GitHubTemplate<'_>) -> Result<(), BackendError> {
    let ExternalGitHubTemplate {
        org,
        repo,
        path,
        branch,
    } = template.into_external_github_template();

    let org_and_repo = format!("{org}/{repo}");

    let extraction_dir = PathBuf::from(format!("{repo}-{branch}"));

    let mut template_path: PathBuf = PathBuf::from(&extraction_dir);

    if let Some(path) = path {
        template_path.push(path);
    }

    template_path.push("grafbase");

    let extraction_result = stream_github_archive(&org_and_repo, template_path, branch).await;

    if extraction_dir.exists() {
        tokio::fs::remove_dir_all(extraction_dir)
            .await
            .map_err(BackendError::CleanExtractedFiles)?;
    }

    extraction_result
}

async fn stream_github_archive<'a>(
    org_and_repo: &'a str,
    template_path: PathBuf,
    branch: &'a str,
) -> Result<(), BackendError> {
    // not using the common environment since it's not initialized here
    // if the OS does not have a cache path or it is not UTF-8, we don't cache the download
    let cache_directory = dirs::cache_dir().and_then(|path| path.join("grafbase").to_str().map(ToOwned::to_owned));

    let mut client_builder = ClientBuilder::new(Client::new());

    if let Some(cache_directory) = cache_directory {
        client_builder = client_builder.with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager { path: cache_directory },
            options: None,
        }));
    }

    let client = client_builder.build();

    let tar_gz_response = client
        .get(format!("https://codeload.github.com/{org_and_repo}/tar.gz/{branch}"))
        .send()
        .await
        .map_err(|error| BackendError::StartDownloadRepoArchive(org_and_repo.to_owned(), error))?;

    if !tar_gz_response.status().is_success() {
        return Err(BackendError::DownloadRepoArchive(org_and_repo.to_owned()));
    }

    let tar_gz_stream = tar_gz_response
        .bytes_stream()
        .map(|result| result.map_err(|error| IoError::new(IoErrorKind::Other, error)));

    let tar_gz_reader = StreamReader::new(tar_gz_stream);
    let tar = GzipDecoder::new(tar_gz_reader);
    let archive = Archive::new(tar.compat());

    let mut entries = archive.entries().map_err(|_| BackendError::ReadArchiveEntries)?;

    while let Some(entry) = entries.next().await {
        let mut entry = entry.map_err(BackendError::ExtractArchiveEntry)?;

        if entry
            .path()
            .ok()
            .filter(|path| path.starts_with(&template_path))
            .is_some()
        {
            entry.unpack_in(".").await.map_err(BackendError::ExtractArchiveEntry)?;
        }
    }

    if !template_path.exists() {
        return Err(BackendError::TemplateNotFound);
    }

    tokio::fs::rename(template_path, GRAFBASE_DIRECTORY)
        .await
        .map_err(BackendError::MoveExtractedFiles)?;

    Ok(())
}

/// resets the local data for the current project by removing the `.grafbase` directory
///
/// # Errors
///
/// - returns [`BackendError::ReadCurrentDirectory`] if the current directory cannot be read
///
/// - returns [`BackendError::DeleteDotGrafbaseDirectory`] if the `.grafbase` directory cannot be deleted
pub fn reset() -> Result<(), BackendError> {
    let environment = Environment::get();

    fs::remove_dir_all(environment.project_dot_grafbase_path.clone())
        .map_err(BackendError::DeleteDotGrafbaseDirectory)?;

    Ok(())
}
