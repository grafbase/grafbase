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
use std::path::PathBuf;
use tokio_stream::StreamExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tokio_util::io::StreamReader;
use url::Url;

/// initializes a new project in the current or a new directory, optionally from a template
///
/// # Errors
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
pub fn init(name: Option<&str>, template: Option<&str>) -> Result<(), BackendError> {
    let project_path = to_project_path(name)?;
    let grafbase_path = project_path.join(GRAFBASE_DIRECTORY);
    let schema_path = grafbase_path.join(GRAFBASE_SCHEMA);

    if schema_path.exists() {
        Err(BackendError::AlreadyAProject(schema_path))
    } else {
        fs::create_dir_all(&grafbase_path).map_err(BackendError::CreateGrafbaseDirectory)?;

        let write_result = if let Some(template) = template {
            match Url::parse(template) {
                Ok(repo_url) => match repo_url.host_str() {
                    Some("github.com") => handle_github_repo_url(&repo_url),
                    _ => Err(BackendError::UnsupportedTemplateURL(template.to_string())),
                },
                Err(_) => download_github_template(&TemplateInfo::Grafbase { path: template }),
            }
        } else {
            fs::write(schema_path, DEFAULT_SCHEMA).map_err(BackendError::WriteSchema)
        };

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

    // not using the common environment since it's not initialized here
    // if the OS does not have a cache path or it is not utf-8, we don't cache the download
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

    template_path.push(&format!("{repo_without_org}-{branch}"));

    if let Some(path) = path {
        template_path.push(path);
    }

    template_path.push("grafbase");

    let mut entries = archive.entries().map_err(|_| BackendError::ReadArchiveEntries)?;

    while let Some(mut entry) = entries.next().await.and_then(Result::ok) {
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

    fs::rename(template_path, "grafbase").map_err(BackendError::MoveExtractedFiles)?;

    // FIXME: do this regardless of success
    fs::remove_dir_all(format!("{repo_without_org}-{branch}")).map_err(BackendError::CleanExtractedFiles)?;

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
