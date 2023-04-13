use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::mpsc::Sender;

use common::environment::Environment;
use futures_util::pin_mut;
use itertools::Itertools;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::errors::ServerError;
use crate::servers::DetectedResolver;
use crate::types::ServerMessage;

#[derive(strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum CommandType {
    Npm,
    Npx,
}

async fn run_npm_command<P: AsRef<Path>>(
    command_type: CommandType,
    artifact_directory_path: P,
    extra_arguments: &[&str],
    tracing: bool,
    environment: &[(&'static str, &'static str)],
) -> Result<(), ServerError> {
    let artifact_directory_path_string = artifact_directory_path
        .as_ref()
        .to_str()
        .ok_or(ServerError::CachePath)?;

    let mut arguments = vec!["--prefix", artifact_directory_path_string];
    arguments.extend(extra_arguments);

    trace!("running '{command_type} {}'", arguments.iter().format(" "));

    let npm_command = Command::new(command_type.as_ref())
        .envs(environment.iter().copied())
        .args(arguments)
        .stdout(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .stderr(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .current_dir(artifact_directory_path)
        .spawn()
        .map_err(ServerError::NpmCommandError)?;

    let output = npm_command
        .wait_with_output()
        .await
        .map_err(ServerError::NpmCommandError)?;

    if output.status.success() {
        Ok(())
    } else {
        Err(ServerError::NpmCommand(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}

#[allow(clippy::too_many_lines)]
async fn build_resolver(
    environment: &Environment,
    environment_variables: &std::collections::HashMap<String, String>,
    resolver_name: &str,
    resolver_wrapper_worker_contents: &str,
    resolver_build_artifact_directory_path: &Path,
    tracing: bool,
) -> Result<(), ServerError> {
    const EXTENSIONS: [&str; 2] = ["js", "ts"];

    use futures_util::StreamExt;

    trace!("building resolver {resolver_name}");

    let resolvers_build_artifact_directory_path = environment.resolvers_build_artifact_path.as_path();
    let resolver_input_file_path_without_extension = environment.resolvers_source_path.join(resolver_name);

    let resolver_paths = futures_util::stream::iter(
        EXTENSIONS
            .iter()
            .map(|extension| resolver_input_file_path_without_extension.with_extension(*extension)),
    )
    .filter(|path| {
        let path = path.as_path().to_owned();
        async move { tokio::fs::try_exists(path).await.expect("must succeed") }
    });
    futures_util::pin_mut!(resolver_paths);
    let resolver_input_file_path = resolver_paths
        .next()
        .await
        .ok_or_else(|| ServerError::ResolverDoesNotExist(resolver_input_file_path_without_extension.clone()))?;

    trace!("locating package.json…");

    let package_json_file_path = {
        let paths = futures_util::stream::iter(
            resolver_input_file_path
                .ancestors()
                .skip(1)
                .take_while(|path| path.starts_with(&environment.project_path))
                .map(|directory_path| directory_path.join("package.json")),
        )
        .filter_map(|path| async {
            if tokio::fs::metadata(&path).await.is_ok() {
                Some(path)
            } else {
                None
            }
        });
        pin_mut!(paths);
        paths.next().await
    };

    tokio::fs::create_dir_all(&resolver_build_artifact_directory_path)
        .await
        .map_err(ServerError::CreateTemporaryFile)?;
    let resolver_build_entrypoint_path = resolver_build_artifact_directory_path.join("entrypoint.js");

    if let Some(package_json_file_path) = package_json_file_path {
        trace!("copying package.json from {}", package_json_file_path.display());
        tokio::fs::copy(
            package_json_file_path,
            resolver_build_artifact_directory_path.join("package.json"),
        )
        .await
        .map_err(ServerError::NpmCommandError)?;
    }

    // FIXME: Drop the dependency on wrangler.
    run_npm_command(
        CommandType::Npm,
        resolver_build_artifact_directory_path,
        &["add", "--save-dev", "wrangler"],
        tracing,
        &[],
    )
    .await?;
    run_npm_command(
        CommandType::Npm,
        resolver_build_artifact_directory_path,
        &["install"],
        tracing,
        &[],
    )
    .await?;

    let entrypoint_contents = resolver_wrapper_worker_contents.replace(
        "${RESOLVER_MAIN_FILE_PATH}",
        resolver_input_file_path.to_str().expect("must be valid utf-8"),
    );
    tokio::fs::write(&resolver_build_entrypoint_path, entrypoint_contents)
        .await
        .map_err(ServerError::CreateResolverArtifactFile)?;

    let wrangler_output_directory_path = resolver_build_artifact_directory_path.join("wrangler");
    let outdir_argument = format!(
        "--outdir={}",
        wrangler_output_directory_path.to_str().expect("must be valid utf-8"),
    );

    trace!("writing the package.json file for '{resolver_name}' used by wrangler");

    tokio::fs::write(
        resolver_build_artifact_directory_path.join("package.json"),
        r#"{ "module": "wrangler/entrypoint.js" }"#,
    )
    .await
    .map_err(ServerError::CreateResolverArtifactFile)?;

    let wrangler_toml_file_path = resolver_build_artifact_directory_path.join("wrangler.toml");

    let _ = tokio::fs::remove_file(&wrangler_toml_file_path).await;

    // Not great. We use wrangler to produce the JS file that is then used as the input for the resolver-specific worker.
    // FIXME: Swap out for the internal logic that wrangler effectively uses under the hood.
    run_npm_command(
        CommandType::Npx,
        resolvers_build_artifact_directory_path,
        &[
            "wrangler",
            "publish",
            "--dry-run",
            &outdir_argument,
            "--compatibility-date",
            "2023-02-08",
            "--name",
            "STUB",
            resolver_build_entrypoint_path.to_str().expect("must be valid utf-8"),
        ],
        tracing,
        &[
            ("CLOUDFLARE_API_TOKEN", "STUB"),
            ("FORCE_COLOR", "0"),
            ("WRANGLER_SEND_METRICS", "false"),
        ],
    )
    .await
    .map_err(|err| match err {
        ServerError::NpmCommand(output) => ServerError::ResolverBuild(resolver_name.to_owned(), output),
        other => other,
    })?;

    let process_env_prelude = format!(
        "globalThis.process = {{ env: {} }};",
        serde_json::to_string(&environment_variables).expect("must be valid JSON")
    );

    let (temp_file, temp_file_path) = tokio::task::spawn_blocking(tempfile::NamedTempFile::new)
        .await?
        .map_err(ServerError::CreateResolverArtifactFile)?
        .into_parts();
    {
        let mut temp_file: tokio::fs::File = temp_file.into();
        temp_file
            .write_all(process_env_prelude.as_bytes())
            .await
            .map_err(ServerError::CreateResolverArtifactFile)?;
        temp_file
            .write_all(
                &tokio::fs::read(wrangler_output_directory_path.join("entrypoint.js"))
                    .await
                    .expect("must succeed"),
            )
            .await
            .map_err(ServerError::CreateResolverArtifactFile)?;
    }
    tokio::fs::copy(temp_file_path, wrangler_output_directory_path.join("entrypoint.js"))
        .await
        .map_err(ServerError::CreateResolverArtifactFile)?;

    let slugified_resolver_name = slug::slugify(resolver_name);
    tokio::fs::write(
        wrangler_toml_file_path,
        format!(
            r#"
                name = "{slugified_resolver_name}"
                [build.upload]
                format = "modules"
                [miniflare]
                routes = ["127.0.0.1/resolver/{resolver_name}/invoke"]
            "#,
        ),
    )
    .await
    .map_err(ServerError::CreateTemporaryFile)?;

    Ok(())
}

async fn extract_resolver_wrapper_worker_contents() -> Result<String, ServerError> {
    trace!("extracting resolver wrapper worker contents");
    let environment = Environment::get();
    tokio::fs::read_to_string(
        environment
            .user_dot_grafbase_path
            .join("custom-resolvers/wrapper-worker.js"),
    )
    .await
    .map_err(ServerError::SchemaParserResultRead)
}

pub async fn build_resolvers(
    sender: &Sender<ServerMessage>,
    environment: &Environment,
    environment_variables: &std::collections::HashMap<String, String>,
    resolvers: impl IntoIterator<Item = crate::servers::DetectedResolver>,
    tracing: bool,
) -> Result<HashMap<String, PathBuf>, ServerError> {
    use futures_util::{StreamExt, TryStreamExt};

    const RESOLVER_BUILD_CONCURRENCY: usize = 8;

    let mut resolvers_iterator = resolvers.into_iter().peekable();
    if resolvers_iterator.peek().is_none() {
        return Ok(HashMap::new());
    }

    let resolver_wrapper_worker_contents = extract_resolver_wrapper_worker_contents().await?;

    let resolvers_build_artifact_directory_path = environment.resolvers_build_artifact_path.as_path();

    futures_util::stream::iter(resolvers_iterator)
        .map(Ok)
        .map_ok(|DetectedResolver { resolver_name, fresh }| {
            let resolver_wrapper_worker_contents = resolver_wrapper_worker_contents.as_str();

            async move {
                let resolver_build_artifact_directory_path =
                    resolvers_build_artifact_directory_path.join(&resolver_name);
                if fresh {
                    let wrangler_toml_path = resolver_build_artifact_directory_path.join("wrangler.toml");
                    // Only touch the file to ensure the modified time relation is preserved.
                    tokio::task::spawn_blocking(|| {
                        filetime::set_file_mtime(wrangler_toml_path, filetime::FileTime::now()).expect("must succeed");
                    })
                    .await
                    .expect("must succeed");
                } else {
                    let start = std::time::Instant::now();
                    let _ = sender.send(ServerMessage::StartResolverBuild(resolver_name.clone()));
                    build_resolver(
                        environment,
                        environment_variables,
                        resolver_name.as_str(),
                        resolver_wrapper_worker_contents,
                        &resolver_build_artifact_directory_path,
                        tracing,
                    )
                    .await?;
                    let _ = sender.send(ServerMessage::CompleteResolverBuild {
                        name: resolver_name.clone(),
                        duration: start.elapsed(),
                    });
                }
                Ok((resolver_name, resolver_build_artifact_directory_path))
            }
        })
        .try_buffer_unordered(RESOLVER_BUILD_CONCURRENCY)
        .try_collect()
        .await
}
