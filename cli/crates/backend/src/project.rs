use crate::consts::DEFAULT_SCHEMA;
use crate::errors::BackendError;
use async_compression::tokio::bufread::GzipDecoder;
use async_tar::Archive;
use common::consts::{GRAFBASE_DIRECTORY, GRAFBASE_SCHEMA};
use common::environment::Environment;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use std::env;
use std::fs;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::str::FromStr;
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
/// - returns [`BackendError::StoreRepoArchive`] if a repo tar could not be stored
///
/// - returns [`BackendError::NoFilesExtracted`] if no files were extracted from the repo archive
///
/// - returns [`BackendError::MoveExtractedFiles`] if the extracted files from the template repository could not be moved
///
/// - returns [`BackendError::ReadArchiveEntries`] if the entries of the template repository archive could not be read
///
/// - returns [`BackendError::ExtractArchiveEntry`] if one of the entries of the template repository archive could not be extracted
///
/// - returns [`BackendError::CleanExtractedFiles`] if the files extracted from the template repository archive could not be cleaned
pub fn init(name: Option<&str>, template: Option<&str>) -> Result<(), BackendError> {
    let project_path = to_project_path(name)?;
    let grafbase_path = project_path.join(GRAFBASE_DIRECTORY);
    let schema_path = grafbase_path.join(GRAFBASE_SCHEMA);

    if grafbase_path.exists() {
        Err(BackendError::AlreadyAProject(grafbase_path))
    } else if let Some(template) = template {
        match Url::parse(template) {
            Ok(repo_url) => match repo_url.host_str() {
                Some("github.com") => handle_github_repo_url(&repo_url),
                _ => Err(BackendError::UnsupportedTemplateURL(template.to_string())),
            },
            Err(_) => download_github_template(&TemplateInfo::Grafbase { path: template }),
        }
    } else {
        fs::create_dir_all(&grafbase_path).map_err(BackendError::CreateGrafbaseDirectory)?;
        let write_result = fs::write(schema_path, DEFAULT_SCHEMA).map_err(BackendError::WriteSchema);

        if write_result.is_err() {
            fs::remove_dir_all(&grafbase_path).map_err(BackendError::DeleteGrafbaseDirectory)?;
        }

        write_result
    }
}

fn handle_github_repo_url(repo_url: &Url) -> Result<(), BackendError> {
    if let Some(segments) = repo_url
        .path_segments()
        .map(Iterator::collect::<Vec<_>>)
        // TODO: allow URLs without 'tree/branch' by checking the default branch via the API
        .filter(|segments| segments.len() >= 4 && segments[2] == "tree")
    {
        let repo = &segments[..=1].join("/");

        let branch = segments[3];

        let path = segments.get(4..).map(|path| path.join("/"));

        download_github_template(&TemplateInfo::External {
            repo,
            path: path.as_deref(),
            branch,
        })
    } else {
        Err(BackendError::UnsupportedTemplateURL(repo_url.to_string()))
    }
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

enum TemplateInfo<'a> {
    Grafbase {
        path: &'a str,
    },
    External {
        repo: &'a str,
        path: Option<&'a str>,
        branch: &'a str,
    },
}

#[tokio::main]
async fn download_github_template(template_info: &TemplateInfo<'_>) -> Result<(), BackendError> {
    let (repo, path, branch) = match template_info {
        TemplateInfo::Grafbase { path } => ("grafbase/grafbase", Some(PathBuf::from("templates").join(path)), "main"),
        TemplateInfo::External { repo, path, branch } => (*repo, path.map(PathBuf::from), *branch),
    };

    let (_, repo_without_org) = repo.split_once('/').expect("must have a slash");

    let extraction_dir = PathBuf::from_str(&format!("{repo_without_org}-{branch}")).expect("must succeed");

    let extraction_result = stream_github_archive(repo, path, branch, extraction_dir.as_path()).await;

    if extraction_dir.exists() {
        tokio::fs::remove_dir_all(extraction_dir)
            .await
            .map_err(BackendError::CleanExtractedFiles)?;
    }

    extraction_result
}

async fn stream_github_archive<'a>(
    repo: &'a str,
    path: Option<PathBuf>,
    branch: &'a str,
    extraction_dir: &'a Path,
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
        .get(format!("https://codeload.github.com/{repo}/tar.gz/{branch}"))
        .send()
        .await
        .map_err(|error| BackendError::StartDownloadRepoArchive(repo.to_owned(), error))?;

    if !tar_gz_response.status().is_success() {
        return Err(BackendError::DownloadRepoArchive(repo.to_owned()));
    }

    let tar_gz_stream = tar_gz_response
        .bytes_stream()
        .map(|result| result.map_err(|error| IoError::new(IoErrorKind::Other, error)));

    let tar_gz_reader = StreamReader::new(tar_gz_stream);
    let tar = GzipDecoder::new(tar_gz_reader);
    let archive = Archive::new(tar.compat());

    let mut template_path = PathBuf::new();

    template_path.push(extraction_dir);

    if let Some(path) = path {
        template_path.push(path);
    }

    template_path.push("grafbase");

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

    // FIXME: an incorrect template name errors here rather than initially
    if !template_path.exists() {
        return Err(BackendError::NoFilesExtracted);
    }

    tokio::fs::rename(template_path, "grafbase")
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
